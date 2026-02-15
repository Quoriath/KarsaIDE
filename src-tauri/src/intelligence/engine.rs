use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use notify::{Watcher, RecursiveMode, Event};
use serde::{Deserialize, Serialize};
use super::classifier::{FileClassifier, SourceFile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, usize>,
    pub indexed_files: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectMap {
    pub tree: String,
    pub total_files: usize,
}

pub struct IndexEngine {
    root: PathBuf,
    files: Arc<Mutex<HashMap<PathBuf, SourceFile>>>,
    stats: Arc<Mutex<CodebaseStats>>,
}

impl IndexEngine {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(CodebaseStats {
                total_files: 0,
                total_lines: 0,
                languages: HashMap::new(),
                indexed_files: 0,
                status: "Idle".to_string(),
            })),
        }
    }

    pub fn start_indexing(&self) {
        let root = self.root.clone();
        let files = self.files.clone();
        let stats = self.stats.clone();

        std::thread::spawn(move || {
            // Update status
            {
                let mut s = stats.lock().unwrap();
                s.status = "Indexing".to_string();
            }

            // Scan files
            let source_files = FileClassifier::scan_project(&root);
            
            // Build stats
            let mut total_lines = 0;
            let mut languages: HashMap<String, usize> = HashMap::new();
            
            for file in &source_files {
                total_lines += file.lines;
                *languages.entry(file.language.clone()).or_insert(0) += 1;
            }

            // Store files
            {
                let mut f = files.lock().unwrap();
                f.clear();
                for file in source_files.iter() {
                    f.insert(file.path.clone(), file.clone());
                }
            }

            // Update stats
            {
                let mut s = stats.lock().unwrap();
                s.total_files = source_files.len();
                s.total_lines = total_lines;
                s.languages = languages;
                s.indexed_files = source_files.len();
                s.status = "Ready".to_string();
            }
        });
    }

    pub fn get_stats(&self) -> CodebaseStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn get_project_map(&self) -> ProjectMap {
        let files = self.files.lock().unwrap();
        let mut tree = String::new();
        
        // Build tree structure
        let mut paths: Vec<_> = files.keys().collect();
        paths.sort();
        
        for path in paths {
            if let Some(rel) = path.strip_prefix(&self.root).ok() {
                let depth = rel.components().count() - 1;
                let indent = "  ".repeat(depth);
                let name = rel.file_name().unwrap().to_string_lossy();
                tree.push_str(&format!("{}{}\n", indent, name));
            }
        }
        
        ProjectMap {
            tree,
            total_files: files.len(),
        }
    }

    pub fn query_codebase(&self, query: &str) -> Vec<String> {
        let files = self.files.lock().unwrap();
        let query_lower = query.to_lowercase();
        
        // Simple keyword search for now
        files
            .iter()
            .filter(|(path, _)| {
                path.to_string_lossy().to_lowercase().contains(&query_lower)
            })
            .take(10)
            .map(|(path, file)| {
                format!("{} ({})", path.display(), file.language)
            })
            .collect()
    }

    pub fn update_file(&self, path: &Path) {
        if let Some(lang) = FileClassifier::is_source_file(path) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let lines = content.lines().count();
                    let file = SourceFile {
                        path: path.to_path_buf(),
                        language: lang,
                        lines,
                        size: metadata.len(),
                    };
                    
                    let mut files = self.files.lock().unwrap();
                    files.insert(path.to_path_buf(), file);
                }
            }
        }
    }

    pub fn remove_file(&self, path: &Path) {
        let mut files = self.files.lock().unwrap();
        files.remove(path);
    }
}
