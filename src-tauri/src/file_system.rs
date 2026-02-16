use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[command]
pub fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[command]
pub fn write_file_content(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = PathBuf::from(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[command]
pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries: Vec<FileEntry> = Vec::new();

    let read_dir = fs::read_dir(&path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path_buf = entry.path();
        let name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.starts_with('.') {
            continue;
        }

        entries.push(FileEntry {
            name,
            path: path_buf.to_string_lossy().to_string(),
            is_dir: path_buf.is_dir(),
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

#[command]
pub fn file_exists(path: String) -> bool {
    PathBuf::from(&path).exists()
}

#[command]
pub fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))
}

#[command]
pub fn delete_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if path_buf.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}

#[command]
pub fn rename_path(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))
}

#[command]
pub fn create_file(path: String, content: Option<String>) -> Result<(), String> {
    let content = content.unwrap_or_default();
    
    if let Some(parent) = PathBuf::from(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }
    
    fs::write(&path, content).map_err(|e| format!("Failed to create file: {}", e))
}

#[command]
pub fn copy_path(source: String, destination: String) -> Result<(), String> {
    let src = PathBuf::from(&source);
    let mut dst = PathBuf::from(&destination);

    // Smart duplicate: If dest exists, append " copy", " copy 2", etc.
    if dst.exists() {
        let file_stem = dst.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let extension = dst.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let parent = dst.parent().unwrap_or(&PathBuf::from("/")).to_path_buf();
        
        let mut counter = 1;
        while dst.exists() {
            let new_name = if extension.is_empty() {
                format!("{} copy {}", file_stem, if counter == 1 { "".to_string() } else { counter.to_string() })
            } else {
                format!("{} copy {}.{}", file_stem, if counter == 1 { "".to_string() } else { counter.to_string() }, extension)
            };
            dst = parent.join(new_name.trim());
            counter += 1;
        }
    }
    
    if src.is_dir() {
        copy_dir_recursive(&src, &dst).map_err(|e| format!("Failed to copy directory: {}", e))
    } else {
        fs::copy(&src, &dst).map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub created: Option<u64>,
    pub modified: Option<u64>,
    pub readonly: bool,
}

#[command]
pub fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    
    let created = meta.created().ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
        
    let modified = meta.modified().ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FileMetadata {
        size: meta.len(),
        created,
        modified,
        readonly: meta.permissions().readonly(),
    })
}

#[command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let path_buf = PathBuf::from(&path);
        let dir = if path_buf.is_dir() {
            &path
        } else {
            path_buf.parent().unwrap().to_str().unwrap()
        };
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}
