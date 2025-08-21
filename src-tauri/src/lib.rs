use std::fs;
use std::path::PathBuf;
use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize)]
struct FolderEntry {
    name: String,
    path: String,
}

#[tauri::command]
fn read_src_folders() -> Result<Vec<FolderEntry>, String> {
    // Get the home directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?;
    
    // Build the path to ~/src
    let src_path = home_dir.join("src");
    
    // Check if the directory exists
    if !src_path.exists() {
        return Ok(Vec::new()); // Return empty list if ~/src doesn't exist
    }
    
    // Read the directory
    let entries = fs::read_dir(&src_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    // Filter for directories only and collect their names
    let mut folders = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        folders.push(FolderEntry {
                            name: name_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Sort folders alphabetically
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(folders)
}

#[tauri::command]
fn read_folders_from_path(folder_path: String) -> Result<Vec<FolderEntry>, String> {
    let path = PathBuf::from(&folder_path);
    
    // Check if the directory exists
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    // Read the directory
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    // Filter for directories only and collect their names
    let mut folders = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        folders.push(FolderEntry {
                            name: name_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Sort folders alphabetically
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(folders)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, read_src_folders, read_folders_from_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
