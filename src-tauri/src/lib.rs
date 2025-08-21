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
    last_modified: u64, // Unix timestamp
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
                        // Get last modified timestamp
                        let last_modified = match fs::metadata(&path) {
                            Ok(metadata) => {
                                match metadata.modified() {
                                    Ok(time) => time.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                    Err(_) => 0,
                                }
                            },
                            Err(_) => 0,
                        };
                        
                        folders.push(FolderEntry {
                            name: name_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                            last_modified,
                        });
                    }
                }
            }
        }
    }
    
    // Sort folders by last modified (newest first)
    folders.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    
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
                        // Get last modified timestamp
                        let last_modified = match fs::metadata(&path) {
                            Ok(metadata) => {
                                match metadata.modified() {
                                    Ok(time) => time.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                    Err(_) => 0,
                                }
                            },
                            Err(_) => 0,
                        };
                        
                        folders.push(FolderEntry {
                            name: name_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                            last_modified,
                        });
                    }
                }
            }
        }
    }
    
    // Sort folders by last modified (newest first)
    folders.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    
    Ok(folders)
}

#[tauri::command]
fn create_game_folder(parent_path: String, folder_name: String, game_type: String) -> Result<String, String> {
    let parent = PathBuf::from(&parent_path);
    
    // Check if parent directory exists
    if !parent.exists() {
        return Err(format!("Parent directory does not exist: {}", parent_path));
    }
    
    if !parent.is_dir() {
        return Err(format!("Parent path is not a directory: {}", parent_path));
    }
    
    // Validate game type
    if game_type != "2d" && game_type != "3d" {
        return Err(format!("Invalid game type: {}. Must be '2d' or '3d'", game_type));
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
    
    // Copy boilerplate files
    copy_boilerplate_files(&game_type, &new_folder_path)?;
    
    Ok(new_folder_path.to_string_lossy().to_string())
}

fn copy_boilerplate_files(game_type: &str, target_path: &PathBuf) -> Result<(), String> {
    // Get current working directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Get the executable directory 
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable directory: {}", e))?
        .parent()
        .ok_or("Failed to get parent directory of executable")?
        .to_path_buf();
    
    // Look for boilerplate templates in various possible locations
    let possible_source_dirs = vec![
        // For development (from current working directory)
        current_dir.join("src").join(format!("{}-game-boilerplate", game_type)),
        // For development (from project root)
        PathBuf::from("src").join(format!("{}-game-boilerplate", game_type)),
        // For packaged app (relative to executable)
        exe_dir.join("src").join(format!("{}-game-boilerplate", game_type)),
        // For packaged app (go up from target/debug or similar)
        exe_dir.parent().unwrap_or(&exe_dir).join("src").join(format!("{}-game-boilerplate", game_type)),
        exe_dir.parent().unwrap_or(&exe_dir).parent().unwrap_or(&exe_dir).join("src").join(format!("{}-game-boilerplate", game_type)),
        exe_dir.parent().unwrap_or(&exe_dir).parent().unwrap_or(&exe_dir).parent().unwrap_or(&exe_dir).join("src").join(format!("{}-game-boilerplate", game_type)),
    ];
    
    let mut source_dir: Option<PathBuf> = None;
    let mut checked_paths = Vec::new();
    
    for possible_dir in possible_source_dirs {
        checked_paths.push(possible_dir.to_string_lossy().to_string());
        if possible_dir.exists() && possible_dir.is_dir() {
            source_dir = Some(possible_dir);
            break;
        }
    }
    
    let source_dir = source_dir.ok_or_else(|| format!(
        "Could not find {}-game-boilerplate directory. Checked paths: {}", 
        game_type,
        checked_paths.join(", ")
    ))?;
    
    // Copy all files from the boilerplate directory to the target
    copy_dir_contents(&source_dir, target_path)
        .map_err(|e| format!("Failed to copy boilerplate files: {}", e))
}

fn copy_dir_contents(source: &PathBuf, target: &PathBuf) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(file_name);
        
        if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
        } else if file_type.is_dir() {
            fs::create_dir(&target_path)?;
            copy_dir_contents(&source_path, &target_path)?;
        }
    }
    Ok(())
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
