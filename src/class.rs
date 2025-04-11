use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::{self, OpenOptions},
    io::Write as IoWrite,
    path::Path,
};

use crate::Index;

#[derive(Debug)]
pub struct Class {
    pub obfuscated: Vec<String>,
    pub real_name: Vec<String>,
    comment: Vec<String>,
    // entry, comment
    entries: Vec<(Entry, Vec<String>)>,
}

impl Class {
    pub fn write(
        &self,
        root: &Path,
        index: &Index,
        package: &str,
        remap: &HashMap<String, String>,
    ) {
        let mut path = format!("{}.{}", package, self.real_name.join("."));

        for (from, to) in remap.iter() {
            path = path.replace(from, to);
        }

        let pathbuf = root.join(path.replace(".", "/")).with_extension("java");

        if !pathbuf.parent().unwrap().exists() {
            fs::create_dir_all(pathbuf.parent().unwrap()).unwrap();
        }

        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(pathbuf)
            .unwrap()
            .write_all(self.to_string(index, package, remap, self.real_name.last().unwrap()).as_bytes())
            .unwrap();
    }

    pub fn to_string(
        &self,
        index: &Index,
        package: &str,
        remap: &HashMap<String, String>,
        path: &str
    ) -> String {
        let package_name = self.real_name[0..self.real_name.len() - 1].join(".");
        let class_name = self.real_name.last().unwrap();
        let original_name = self.real_name.join(".");
        let mut remapped = format!("{package}.{package_name}");

        let entries = self
            .entries
            .iter()
            .cloned()
            .fold(String::new(), |mut acc, entry| {
                writeln!(
                    acc,
                    "{}",
                    entry
                        .0
                        .to_string(index, package, remap, &original_name, path)
                )
                .unwrap();
                acc
            });
        for (from, to) in remap.iter() {
            remapped = remapped.replace(from, to);
        }

        format!(
            r#"package {remapped};
public class {class_name} {{ public {original_name} wrapperContained; public {class_name}({original_name} wrapperContained) {{ this.wrapperContained = wrapperContained; }}

{entries}
}}"#,
        )
    }
}

impl Class {
    pub fn from_str(s: &str) -> Self {
        let (obfuscated, real_name) = Self::class_sig(s.lines().next().unwrap());
        let mut comment = None;

        // buffers
        let (mut comments, mut entries) = (Vec::new(), Vec::new());

        for line in s.lines().skip(1) {
            match line.trim().splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                ["COMMENT", content] => {
                    comments.push(content.to_string());
                    continue;
                }
                ["COMMENT"] => {
                    comments.push(String::new());
                    continue;
                }
                ["METHOD", sig] => entries.push((Entry::method(sig), Vec::new())),
                ["ARG", sig] if entries.last().unwrap().0.is_empty() => {
                    entries.last_mut().unwrap().1 = comments;
                    entries.last_mut().unwrap().0.push_arg(sig.to_string());
                    comments = Vec::new();
                    continue;
                }
                ["ARG", sig] => {
                    entries.last_mut().unwrap().0.insert_comment(comments);
                    entries.last_mut().unwrap().0.push_arg(sig.to_string());
                    comments = Vec::new();
                    continue;
                }
                ["FIELD", sig] => {
                    entries.push((Entry::field(sig), Vec::new()));
                }
                ["CLASS", _] => break, // TODO??? may not be needed
                x => panic!("unknown variant {x:?}"),
            }

            // push comment
            if comment.is_none() {
                comment = Some(comments);
            } else if entries.last().unwrap().0.is_empty() {
                entries.last_mut().unwrap().1 = comments;
            } else {
                entries.last_mut().unwrap().0.insert_comment(comments);
            }

            comments = Vec::new();
        }

        Self {
            obfuscated,
            real_name,
            comment: comment.unwrap_or_default(),
            entries: entries
                .into_iter()
                .filter(Entry::not_dummy)
                .collect::<Vec<_>>(),
        }
    }

    fn class_sig(s: &str) -> (Vec<String>, Vec<String>) {
        let mut iter = s.splitn(3, ' ').skip(1);
        (
            iter.next()
                .unwrap()
                .split('/')
                .map(str::to_string)
                .collect(),
            iter.next()
                .filter(|s| !s.is_empty())
                .unwrap_or(s.split(' ').nth(1).unwrap())
                .split('/')
                .map(str::to_string)
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub enum Entry {
    Field {
        label: String,
        r#type: String,
    },
    // params: (param, comment)
    Method {
        label: String,
        params: String,
        param_declr: Vec<(String, Vec<String>)>,
        output: String,
    },
    // Class (Class)
}

impl Entry {
    fn type_preproc(s: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut buf = String::new();
        let mut obj = false;

        s.chars().for_each(|c| {
            if c == 'L' {
                obj = true
            }

            if matches!(c, 'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 'V') && !obj {
                out.push(c.to_string());
            } else if c == ';' {
                obj = false;
                out.push(buf.clone());
                buf.clear();
            } else {
                buf.push(c);
            }
        });

        out
    }

    fn type_string(
        s: &str,
        index: &Index,
        package: &str,
        wrap: bool,
        remap: &HashMap<String, String>,
    ) -> String {
        if !s.starts_with("[") && s.contains("$") {
            return "Object".to_string();
        }

        match s {
            "B" => "byte".to_string(),
            "C" => "char".to_string(),
            "D" => "double".to_string(),
            "F" => "float".to_string(),
            "I" => "int".to_string(),
            "J" => "long".to_string(),
            "S" => "short".to_string(),
            "Z" => "boolean".to_string(),
            "V" => "void".to_string(),
            s if s.starts_with("L") => {
                if let Some(class) = index.get_str(&s[1..]) {
                    if class
                        .real_name
                        .starts_with(&["net".to_string(), "minecraft".to_string()])
                        && wrap
                    {
                        let mut out = format!("{package}.{}", class.real_name.join("."));

                        for (from, to) in remap.iter() {
                            out = out.replace(from, to);
                        }

                        out
                    } else {
                        class.real_name.join(".")
                    }
                } else {
                    s[1..].to_string().replace("/", ".")
                }
            }
            s if s.starts_with("[") => {
                format!(
                    "{}[]",
                    Self::type_string(&s[1..], index, package, false, remap)
                )
            }
            "" => String::new(),
            x => panic!("unknown type string \"{x}\""),
        }
    }

    pub fn to_string(
        self,
        index: &Index,
        package: &str,
        remap: &HashMap<String, String>,
        original_name: &str,
        remapped_name: &str,
    ) -> String {
        match self {
            Self::Field { label, r#type } => {
                let r#type = if r#type.contains('$') {
                    "LObject".to_string()
                } else {
                    r#type.to_string()
                };

                let class = Self::type_string(&r#type, index, package, true, remap);

                if class.starts_with(package) {
                    format!(
                        r#"public {class} {label}() {{ return new {class}(wrapperContained.{label}); }}
public void {label}({class} value) {{ wrapperContained.{label} = value.wrapperContained; }}"#
                    )
                } else {
                    format!(
                        r#"public {class} {label}() {{ return wrapperContained.{label}; }}
public void {label}({class} value) {{ wrapperContained.{label} = value; }}"#
                    )
                }
            }
            Self::Method {
                label,
                params,
                param_declr,
                output,
            } => {
                let params = Self::type_preproc(&params);
                let output = if output.contains('$') {
                    "LObject".to_string()
                } else {
                    output.to_string()
                };

                let mut wrapped = HashSet::new();
                let class = Self::type_string(&output, index, package, true, remap);
                let params = params
                    .iter()
                    .zip(param_declr.iter().map(|item| &item.0))
                    .enumerate()
                    .fold(String::new(), |mut acc, (i, (adt, name))| {
                        let class = Self::type_string(adt, index, package, true, remap);
                        if class.starts_with(package) {
                            wrapped.insert(i);
                        }
                        write!(acc, "{class} {name},").unwrap();
                        acc
                    })
                    .trim_end_matches(',')
                    .to_string();

                let args = param_declr
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut acc, (i, (name, _comment))| {
                        if wrapped.contains(&i) {
                            write!(acc, "{name}.wrapperContained,").unwrap();
                        } else {
                            write!(acc, "{name},").unwrap();
                        }
                        acc
                    })
                    .trim_end_matches(',')
                    .to_string();

                if label.as_str() == "<init>" {
                    return format!("public {remapped_name}({params}) {{ this.wrapperContained = new {original_name}({args}); }}");
                }

                format!(
                    "public {class} {label}({params}) {{ {} }}",
                    if class == "void" {
                        format!("wrapperContained.{label}({args});",)
                    } else if class.starts_with(package) {
                        format!("return new {class}(wrapperContained.{label}({args}));",)
                    } else {
                        format!("return wrapperContained.{label}({args});",)
                    }
                )
            }
        }
    }
}

impl Entry {
    pub fn method(sig: &str) -> Self {
        match sig.splitn(3, ' ').collect::<Vec<_>>().as_slice() {
            [_obfuscated, real_name, signature] => Self::Method {
                label: real_name.to_string(),
                params: signature.split_once(')').unwrap().0[1..].to_string(),
                param_declr: Vec::new(),
                output: signature
                    .split_once(')')
                    .unwrap()
                    .1
                    .trim_end_matches(';')
                    .to_string(),
            },
            /*
            ["<init>", sig] => {
                todo!()
            }*/
            [name, sig] => Self::Method {
                label: name.to_string(),
                params: sig.split_once(')').unwrap().0[1..].to_string(),
                param_declr: Vec::new(),
                output: sig
                    .split_once(')')
                    .unwrap()
                    .1
                    .trim_end_matches(';')
                    .to_string(),
            },
            _ => Self::dummy_method(),
        }
    }

    fn dummy_method() -> Self {
        Self::Method {
            label: String::new(),
            params: String::new(),
            param_declr: Vec::new(),
            output: String::new(),
        }
    }

    pub fn field(sig: &str) -> Self {
        match sig.splitn(3, ' ').collect::<Vec<_>>().as_slice() {
            [_obfuscated, label, r#type] => Self::Field {
                label: label.to_string(),
                r#type: r#type.trim_matches(';').to_string(),
            },
            _ => Self::dummy_field(),
        }
    }

    fn dummy_field() -> Self {
        Self::Field {
            label: String::new(),
            r#type: String::new(),
        }
    }

    pub fn not_dummy((val, _comment): &(Self, Vec<String>)) -> bool {
        match val {
            Self::Field { label, .. } | Self::Method { label, .. } => !label.is_empty(),
        }
    }

    pub fn push_arg(&mut self, arg: String) {
        match self {
            Self::Method { param_declr, .. } => {
                param_declr.push((arg.split_once(' ').unwrap().1.to_string(), Vec::new()))
            }
            _ => panic!("pushing arg on not a method"),
        }
    }

    pub fn insert_comment(&mut self, comments: Vec<String>) {
        match self {
            Self::Method { param_declr, .. } => param_declr.last_mut().unwrap().1 = comments,
            Self::Field { .. } => panic!("inserting comment for field"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Field { .. } => true,
            Self::Method { param_declr, .. } => param_declr.is_empty(),
        }
    }
}
