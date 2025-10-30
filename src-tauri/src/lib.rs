use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;
use tauri_plugin_store::StoreExt;
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;

mod menu;

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
fn create_game_folder(parent_path: String, folder_name: String, game_type: String, app_handle: tauri::AppHandle) -> Result<String, String> {
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
    copy_boilerplate_files(&game_type, &new_folder_path, &app_handle)?;
    
    Ok(new_folder_path.to_string_lossy().to_string())
}

fn copy_boilerplate_files(game_type: &str, target_path: &PathBuf, app_handle: &tauri::AppHandle) -> Result<(), String> {
    // Try to resolve the boilerplate directory from bundled resources first
    let resource_paths = vec![
        format!("{}-game-boilerplate", game_type),
        format!("{}-game-boilerplate/", game_type),
        format!("../src/{}-game-boilerplate", game_type),
        format!("../src/{}-game-boilerplate/", game_type),
    ];
    
    for resource_path in &resource_paths {
        // First try to resolve from bundled resources
        if let Ok(source_dir) = app_handle.path().resolve(resource_path, BaseDirectory::Resource) {
            if source_dir.exists() && source_dir.is_dir() {
                return copy_dir_contents(&source_dir, target_path)
                    .map_err(|e| format!("Failed to copy boilerplate files from resources: {}", e));
            }
        }
    }
    
    // Fallback to development mode - look for boilerplate templates in filesystem
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Get the project root by looking for Cargo.toml
    let project_root = find_project_root();
    
    // Look for boilerplate templates in development locations
    let possible_source_dirs = vec![
        // For development (from current working directory)
        current_dir.join("src").join(format!("{}-game-boilerplate", game_type)),
        // For development (from project root)
        project_root.join("src").join(format!("{}-game-boilerplate", game_type)),
        // Try relative path from current directory
        PathBuf::from("src").join(format!("{}-game-boilerplate", game_type)),
        // Try relative path from project root
        project_root.join("src").join(format!("{}-game-boilerplate", game_type)),
    ];
    
    let mut source_dir: Option<PathBuf> = None;
    let mut checked_paths = Vec::new();
    
    for possible_dir in possible_source_dirs {
        let path_str = possible_dir.to_string_lossy().to_string();
        checked_paths.push(path_str);
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

fn find_project_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    loop {
        if current.join("Cargo.toml").exists() {
            return current;
        }
        
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    // Fallback to current directory if we can't find Cargo.toml
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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

#[tauri::command]
async fn check_for_updates_manually(app_handle: AppHandle) -> Result<String, String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        
        match app_handle.updater() {
            Ok(updater) => {
                match updater.check().await {
                    Ok(update) => {
                        if let Some(update) = update {
                            return Ok(format!("Update available: {}", update.version));
                        } else {
                            return Ok("No updates available. You're running the latest version!".to_string());
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to check for updates: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to get updater: {}", e));
            }
        }
    }
    
    #[cfg(not(desktop))]
    return Err("Update checking is not supported on this platform".to_string());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_menu_event(|app, event| {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                menu::handle_menu_event(&app_handle, event).await;
            });
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Create and set the menu
            let menu = menu::create_menu(&app.handle())?;
            app.set_menu(menu)?;
            
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
            open_html_in_browser,
            check_for_updates_manually
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
