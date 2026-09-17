mod clipboard_monitor;
mod commands;
mod storage;
pub mod vault;

use std::str::FromStr;
use std::sync::Arc;
use storage::StorageManager;
use vault::{SharedVault, VaultManager};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewWindow,
};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub static USER_HAS_DRAGGED_WINDOW: AtomicBool = AtomicBool::new(false);
pub static USER_CUSTOM_X: AtomicI32 = AtomicI32::new(-1);
pub static USER_CUSTOM_Y: AtomicI32 = AtomicI32::new(-1);
pub static IS_PROGRAMMATIC_POSITIONING: AtomicBool = AtomicBool::new(false);

pub fn calculate_bottom_right_position(window: &WebviewWindow) -> (i32, i32) {
    #[cfg(target_os = "linux")]
    {
        use gtk::gdk::prelude::MonitorExt;
        use gtk::prelude::*;
        if let Some(display) = gtk::gdk::Display::default() {
            if let Some(monitor) = display.primary_monitor() {
                let geom = monitor.geometry();
                let (target_w, target_h) = if let Ok(gtk_win) = window.gtk_window() {
                    let (w, h) = gtk_win.size();
                    (if w > 100 { w } else { 640 }, if h > 100 { h } else { 560 })
                } else {
                    (640, 560)
                };

                // 24px from right, 54px from bottom (space for dock/taskbar)
                let x = geom.x() + geom.width() - target_w - 24;
                let y = geom.y() + geom.height() - target_h - 54;
                return (x.max(0), y.max(0));
            }
        }
    }

    let monitor_opt = window.current_monitor().ok().flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor_opt {
        let screen_size = monitor.size();
        let mon_pos = monitor.position();
        let scale_factor = monitor.scale_factor();

        let win_size = window.outer_size().unwrap_or(tauri::PhysicalSize {
            width: (640.0 * scale_factor) as u32,
            height: (560.0 * scale_factor) as u32,
        });

        let margin_x = (24.0 * scale_factor) as i32;
        let margin_bottom = (54.0 * scale_factor) as i32;

        let x = mon_pos.x + (screen_size.width as i32) - (win_size.width as i32) - margin_x;
        let y = mon_pos.y + (screen_size.height as i32) - (win_size.height as i32) - margin_bottom;
        (x.max(0), y.max(0))
    } else {
        (1256, 466)
    }
}

pub fn apply_position(window: &WebviewWindow, x: i32, y: i32) {
    IS_PROGRAMMATIC_POSITIONING.store(true, Ordering::SeqCst);

    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        if let Ok(gtk_win) = window.gtk_window() {
            gtk_win.set_type_hint(gdk::WindowTypeHint::Utility);
            gtk_win.set_decorated(false);
            gtk_win.move_(x, y);
            if let Some(gdk_win) = gtk_win.window() {
                gdk_win.move_(x, y);
            }
        }
    }

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));

    #[cfg(target_os = "linux")]
    {
        let win_clone = window.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(40), move || {
            use gtk::prelude::*;
            if let Ok(gtk_win) = win_clone.gtk_window() {
                gtk_win.move_(x, y);
                if let Some(gdk_win) = gtk_win.window() {
                    gdk_win.move_(x, y);
                }
            }
            let _ = win_clone.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
            IS_PROGRAMMATIC_POSITIONING.store(false, Ordering::SeqCst);
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        IS_PROGRAMMATIC_POSITIONING.store(false, Ordering::SeqCst);
    }
}

pub fn apply_current_target_position(window: &WebviewWindow) {
    let (target_x, target_y) = if USER_HAS_DRAGGED_WINDOW.load(Ordering::SeqCst) {
        let cx = USER_CUSTOM_X.load(Ordering::SeqCst);
        let cy = USER_CUSTOM_Y.load(Ordering::SeqCst);
        if cx > 10 && cy > 10 {
            (cx, cy)
        } else {
            calculate_bottom_right_position(window)
        }
    } else {
        calculate_bottom_right_position(window)
    };

    println!("[Window] Applying target position: ({}, {}) (user_dragged={})",
        target_x, target_y, USER_HAS_DRAGGED_WINDOW.load(Ordering::SeqCst));

    apply_position(window, target_x, target_y);
}

pub fn position_bottom_right(window: &WebviewWindow) {
    USER_HAS_DRAGGED_WINDOW.store(false, Ordering::SeqCst);
    USER_CUSTOM_X.store(-1, Ordering::SeqCst);
    USER_CUSTOM_Y.store(-1, Ordering::SeqCst);
    let (x, y) = calculate_bottom_right_position(window);
    apply_position(window, x, y);
}

pub fn toggle_main_window(app: &AppHandle) {
    let app_handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        toggle_main_window_inner(&app_handle);
    });
}

fn toggle_main_window_inner(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);

        println!("[Window] Toggle requested: visible={}", is_visible);

        if is_visible {
            let _ = window.hide();
        } else {
            // Apply target position before showing
            apply_current_target_position(&window);

            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();

            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                if let Ok(gtk_win) = window.gtk_window() {
                    gtk_win.set_keep_above(true);
                    gtk_win.set_accept_focus(true);
                    gtk_win.set_focus_on_map(true);
                    gtk_win.present_with_time(0);
                    gtk_win.activate_focus();
                    if let Some(gdk_win) = gtk_win.window() {
                        gdk_win.focus(0);
                    }
                }
            }

            // Re-affirm position after presentation so WM cannot reset it to (0, 0)
            apply_current_target_position(&window);

            let _ = window.emit("window-opened", ());
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let storage_manager = Arc::new(StorageManager::new());
    let vault_manager: SharedVault = Arc::new(VaultManager::new());
    let storage_clone_for_tray = storage_manager.clone();
    let storage_clone_for_monitor = storage_manager.clone();

    let saved_shortcut_str = storage_manager.get_settings().global_shortcut;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    println!("[GlobalShortcut] Detected event: {:?} on shortcut: {:?}", event.state(), shortcut);
                    if event.state() == ShortcutState::Pressed {
                        toggle_main_window(app);
                    }
                })
                .build(),
        )
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Moved(pos) = event {
                // Ignore programmatic moves
                if IS_PROGRAMMATIC_POSITIONING.load(Ordering::SeqCst) {
                    return;
                }
                // Ignore spurious (0, 0) coordinates from X11 window mapping
                if pos.x < 15 && pos.y < 15 {
                    return;
                }
                // Only save custom coordinate if user has intentionally dragged the window
                if USER_HAS_DRAGGED_WINDOW.load(Ordering::SeqCst) {
                    USER_CUSTOM_X.store(pos.x, Ordering::SeqCst);
                    USER_CUSTOM_Y.store(pos.y, Ordering::SeqCst);
                    println!("[Window] Recorded user custom position: ({}, {})", pos.x, pos.y);
                }
            }
        })
        .manage(storage_manager.clone())
        .manage(vault_manager.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::delete_clip,
            commands::toggle_pin_clip,
            commands::reorder_clips,
            commands::clear_clips,
            commands::copy_clip_to_clipboard,
            commands::get_settings,
            commands::save_settings,
            commands::hide_window,
            commands::show_window,
            commands::get_vault_status,
            commands::init_vault,
            commands::unlock_vault,
            commands::auto_unlock_vault,
            commands::set_vault_pin,
            commands::remove_vault_pin,
            commands::lock_vault,
            commands::get_vault_items,
            commands::save_vault_item,
            commands::delete_vault_item,
            commands::copy_vault_secret,
            commands::copy_vault_file,
            commands::export_vault_file,
            commands::open_vault_file,
            commands::pick_vault_file,
            commands::get_vault_path,
            commands::get_vault_file_base64,
            commands::get_vault_tabs,
            commands::add_vault_tab,
            commands::delete_vault_tab,
            commands::toggle_vault_item_pin,
            commands::set_window_size,
            commands::save_current_window_size,
            commands::apply_window_size,
            commands::reset_window_position,
            commands::move_window_by,
            commands::notify_user_dragged,
            commands::open_external_url,
            commands::export_clipboard_history,
            commands::export_vault_backup,
            commands::restore_vault_backup,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            let saved_settings = storage_manager.get_settings();
            commands::update_linux_autostart(saved_settings.run_at_startup);

            // Apply saved window size and position window at bottom right on initial launch
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "linux")]
                {
                    use gtk::prelude::*;
                    if let Ok(gtk_win) = window.gtk_window() {
                        gtk_win.set_type_hint(gdk::WindowTypeHint::Utility);
                        gtk_win.set_decorated(false);
                    }
                }
                let _ = window.set_size(tauri::LogicalSize::new(saved_settings.window_width, saved_settings.window_height));
                position_bottom_right(&window);
            }

            // 1. Setup system tray
            let toggle_item = MenuItem::with_id(app, "toggle", "Show / Hide ClipFort", true, None::<&str>)?;
            let clear_unpinned_item = MenuItem::with_id(app, "clear_unpinned", "Clear Unpinned Items", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit ClipFort", true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[&toggle_item, &clear_unpinned_item, &quit_item])?;

            let storage_for_tray_click = storage_clone_for_tray.clone();
            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "toggle" => {
                        toggle_main_window(app);
                    }
                    "clear_unpinned" => {
                        storage_for_tray_click.clear_all(true);
                        let _ = app.emit("clipboard-updated", ());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // 2. Register single global shortcut (saved in settings, default Alt+Shift+Q)
            if let Ok(sc) = Shortcut::from_str(&saved_shortcut_str) {
                if let Err(e) = app.global_shortcut().register(sc) {
                    eprintln!("[GlobalShortcut] Could not register {}: {}", saved_shortcut_str, e);
                } else {
                    println!("[GlobalShortcut] Successfully registered single hotkey: {}", saved_shortcut_str);
                }
            }

            // 3. Start clipboard monitor background thread
            clipboard_monitor::start_clipboard_monitor(handle, storage_clone_for_monitor);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
