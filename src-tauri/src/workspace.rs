use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

const IGNORED_DIRS: &[&str] = &[
    "node_modules", ".git", "target", "dist", "build", ".next", 
    ".cache", "coverage", ".vscode", ".idea"
];

pub fn scan_directory(path: &str, max_depth: usize) -> Result<FileNode> {
    let root = Path::new(path);
    
    if !root.exists() {
        return Err(anyhow::anyhow!("Path does not exist"));
    }
    
    let name = root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    
    if !root.is_dir() {
        return Ok(FileNode {
            name,
            path: path.to_string(),
            is_dir: false,
            children: None,
        });
    }
    
    let mut children = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().to_string();
            
            // Skip ignored directories
            if entry_path.is_dir() && IGNORED_DIRS.contains(&entry_name.as_str()) {
                continue;
            }
            
            // Skip hidden files
            if entry_name.starts_with('.') && entry_name != ".env" {
                continue;
            }
            
            let is_dir = entry_path.is_dir();
            let path_str = entry_path.to_string_lossy().to_string();
            
            let node = if is_dir && max_depth > 0 {
                scan_directory(&path_str, max_depth - 1).unwrap_or(FileNode {
                    name: entry_name,
                    path: path_str,
                    is_dir: true,
                    children: Some(Vec::new()),
                })
            } else {
                FileNode {
                    name: entry_name,
                    path: path_str,
                    is_dir,
                    children: if is_dir { Some(Vec::new()) } else { None },
                }
            };
            
            children.push(node);
        }
    }
    
    // Sort: directories first, then files, alphabetically
    children.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(FileNode {
        name,
        path: path.to_string(),
        is_dir: true,
        children: Some(children),
    })
}

pub fn quick_scan(path: &str) -> Result<Vec<FileNode>> {
    let root = Path::new(path);
    let mut nodes = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().to_string();
            
            if entry_path.is_dir() && IGNORED_DIRS.contains(&entry_name.as_str()) {
                continue;
            }
            
            if entry_name.starts_with('.') && entry_name != ".env" {
                continue;
            }
            
            nodes.push(FileNode {
                name: entry_name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir: entry_path.is_dir(),
                children: if entry_path.is_dir() { Some(Vec::new()) } else { None },
            });
        }
    }
    
    nodes.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(nodes)
}
