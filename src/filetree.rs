use std::{
    collections::HashMap,
    fs, iter,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileTree(pub HashMap<Vec<String>, String>);

impl FileTree {
    pub fn new(root: impl Into<PathBuf> + Clone) -> Self {
        let map = Self::scan_dir(root.clone().into().as_path());
        Self(
            map.into_iter()
                .zip(iter::repeat(root.into().as_path()))
                .map(Self::prepend)
                .collect(),
        )
    }

    fn prepend(
        ((mut chunks, content), prepend): ((Vec<String>, String), &Path),
    ) -> (Vec<String>, String) {
        chunks.insert(
            0,
            prepend.file_name().unwrap().to_string_lossy().to_string(),
        );
        (chunks, content)
    }

    fn scan_dir(root: &Path) -> HashMap<Vec<String>, String> {
        fs::read_dir(root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| (entry.path(), entry.metadata().unwrap()))
            .flat_map(|(path, meta)| {
                if meta.is_file() {
                    fs::read_to_string(&path)
                        .map(|content| {
                            vec![(
                                vec![path.file_name().unwrap().to_string_lossy().to_string()],
                                content,
                            )]
                        })
                        .unwrap_or_default()
                } else {
                    Self::scan_dir(&path)
                        .into_iter()
                        .zip(iter::repeat(path.as_path()))
                        .map(Self::prepend)
                        .collect()
                }
            })
            .collect()
    }
}
