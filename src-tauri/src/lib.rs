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

#[tauri::command]
fn create_game_folder(parent_path: String, folder_name: String) -> Result<String, String> {
    let parent = PathBuf::from(&parent_path);
    
    // Check if parent directory exists
    if !parent.exists() {
        return Err(format!("Parent directory does not exist: {}", parent_path));
    }
    
    if !parent.is_dir() {
        return Err(format!("Parent path is not a directory: {}", parent_path));
    }
    
    // Create the full path for the new folder
    let new_folder_path = parent.join(&folder_name);
    
    // Check if folder already exists
    if new_folder_path.exists() {
        return Err(format!("Folder already exists: {}", folder_name));
    }
    
    // Create the directory
    fs::create_dir(&new_folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;
    
    Ok(new_folder_path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, read_src_folders, read_folders_from_path, create_game_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
