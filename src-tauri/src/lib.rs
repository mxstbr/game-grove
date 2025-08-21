use std::fs;
use std::path::Path;
use serde::Serialize;

#[derive(Serialize)]
struct FolderInfo {
    name: String,
    path: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_folders_in_src() -> Result<Vec<FolderInfo>, String> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Unable to determine home directory")?;
    
    let src_path = Path::new(&home_dir).join("src");
    
    if !src_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut folders = Vec::new();
    
    match fs::read_dir(&src_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name() {
                            if let Some(name_str) = name.to_str() {
                                folders.push(FolderInfo {
                                    name: name_str.to_string(),
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Error reading directory: {}", e)),
    }
    
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(folders)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_folders_in_src])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
