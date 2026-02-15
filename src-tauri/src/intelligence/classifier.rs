use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: String,
    pub lines: usize,
    pub size: u64,
}

pub struct FileClassifier;

impl FileClassifier {
    const SOURCE_EXTS: &'static [(&'static str, &'static str)] = &[
        ("rs", "Rust"),
        ("ts", "TypeScript"),
        ("tsx", "TypeScript"),
        ("js", "JavaScript"),
        ("jsx", "JavaScript"),
        ("svelte", "Svelte"),
        ("vue", "Vue"),
        ("py", "Python"),
        ("go", "Go"),
        ("java", "Java"),
        ("kt", "Kotlin"),
        ("c", "C"),
        ("cpp", "C++"),
        ("h", "C/C++"),
        ("cs", "C#"),
        ("rb", "Ruby"),
        ("php", "PHP"),
    ];

    pub fn is_source_file(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| {
                Self::SOURCE_EXTS
                    .iter()
                    .find(|(e, _)| *e == ext)
                    .map(|(_, lang)| lang.to_string())
            })
    }

    pub fn scan_project(root: &Path) -> Vec<SourceFile> {
        WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build()
            .filter_map(|entry| entry.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|entry| {
                let path = entry.path();
                Self::is_source_file(path).and_then(|lang| {
                    let metadata = std::fs::metadata(path).ok()?;
                    let content = std::fs::read_to_string(path).ok()?;
                    let lines = content.lines().count();
                    
                    Some(SourceFile {
                        path: path.to_path_buf(),
                        language: lang,
                        lines,
                        size: metadata.len(),
                    })
                })
            })
            .collect()
    }
}
