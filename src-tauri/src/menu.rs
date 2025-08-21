use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};

pub fn create_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    // Create the "Check for Updates..." menu item
    let check_updates = MenuItemBuilder::with_id("check_updates", "Check for Updates...").build(app)?;
    
    // Create the main app menu
    let app_menu = SubmenuBuilder::new(app, "Game Grove")
        .item(&PredefinedMenuItem::about(app, Some("Game Grove"), None)?)
        .separator()
        .item(&check_updates)
        .separator()
        .item(&PredefinedMenuItem::services(app, Some("Services"))?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("Hide Game Grove"))?)
        .item(&PredefinedMenuItem::hide_others(app, Some("Hide Others"))?)
        .item(&PredefinedMenuItem::show_all(app, Some("Show All"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Quit Game Grove"))?)
        .build()?;
    
    // Create the Edit menu
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;
    
    // Create the View menu
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&PredefinedMenuItem::fullscreen(app, Some("Enter Full Screen"))?)
        .build()?;
    
    // Create the Window menu
    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, Some("Minimize"))?)
        .item(&PredefinedMenuItem::maximize(app, Some("Zoom"))?)
        .build()?;
    
    // Combine all menus
    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()
}

pub async fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "check_updates" => {
            println!("Check for updates menu item clicked");
            
            // Call the update check command
            match super::check_for_updates_manually(app.clone()).await {
                Ok(message) => {
                    println!("Update check result: {}", message);
                    // You could show a dialog here if needed
                }
                Err(error) => {
                    println!("Update check error: {}", error);
                }
            }
        }
        _ => {}
    }
}
