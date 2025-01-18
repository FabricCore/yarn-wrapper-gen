use std::{collections::HashMap, path::PathBuf};

use crate::{Class, FileTree};

// obfuscated, entry
pub struct Index(HashMap<Vec<String>, Class>);

impl Index {
    pub fn new(ftree: &FileTree) -> Self {
        let map = HashMap::from_iter(
            ftree
                .0
                .values()
                .map(String::as_str)
                .map(Class::from_str)
                .map(|entry| (entry.obfuscated.clone(), entry)),
        );
        Self(map)
    }

    pub fn get(&self, ident: &[String]) -> Option<&Class> {
        self.0.get(ident)
    }

    pub fn get_str(&self, ident: &str) -> Option<&Class> {
        self.0
            .get(&ident.split('/').map(str::to_string).collect::<Vec<_>>())
    }

    pub fn write(&self, root: impl Into<PathBuf>, index: &Index, package: &str) {
        let root: PathBuf = root.into();
        for entry in self.0.values() {
            entry.write(root.as_path(), index, package);
        }
    }
}
