use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;
use tauri_plugin_store::StoreExt;
use serde_json::json;

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

#[tauri::command]
fn open_in_cursor(folder_path: String) -> Result<(), String> {
    let path = PathBuf::from(&folder_path);
    
    // Check if the directory exists
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    // On macOS, use the 'open' command with Cursor
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Cursor")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open Cursor: {}", e))?;
    }
    
    // On Windows, try to use cursor.exe or code.exe
    #[cfg(target_os = "windows")]
    {
        // Try Cursor first, then fall back to VS Code
        let result = Command::new("cursor")
            .arg(&folder_path)
            .spawn();
        
        if result.is_err() {
            Command::new("code")
                .arg(&folder_path)
                .spawn()
                .map_err(|e| format!("Failed to open Cursor/Code: {}", e))?;
        }
    }
    
    // On Linux, try cursor or code command
    #[cfg(target_os = "linux")]
    {
        let result = Command::new("cursor")
            .arg(&folder_path)
            .spawn();
        
        if result.is_err() {
            Command::new("code")
                .arg(&folder_path)
                .spawn()
                .map_err(|e| format!("Failed to open Cursor/Code: {}", e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
fn open_html_in_browser(folder_path: String) -> Result<(), String> {
    let path = PathBuf::from(&folder_path);
    
    // Check if the directory exists
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    // Build path to index.html
    let html_path = path.join("index.html");
    
    // Check if index.html exists
    if !html_path.exists() {
        return Err(format!("index.html not found in: {}", folder_path));
    }
    
    // Convert to file:// URL
    let file_url = format!("file://{}", html_path.to_string_lossy());
    
    // Open in default browser using the 'open' command on macOS
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&file_url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    
    // On Windows, use 'start' command
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "", &file_url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    
    // On Linux, try xdg-open
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&file_url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Initialize the store
            let store = app.store("app_settings.json")?;
            
            // Optionally, set default values if they don't exist
            if store.get("selected_games_path").is_none() {
                store.set("selected_games_path".to_string(), json!(null));
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            read_src_folders, 
            read_folders_from_path, 
            create_game_folder,
            open_in_cursor,
            open_html_in_browser
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
