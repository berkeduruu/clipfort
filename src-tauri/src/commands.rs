use crate::clipboard_monitor::IS_INTERNAL_COPY;
use crate::storage::{AppSettings, ClipItem, SharedStorage, StorageManager};
use crate::vault::{SharedVault, VaultItem, VaultStatus};
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[tauri::command]
pub fn get_clips(storage: State<'_, SharedStorage>) -> Vec<ClipItem> {
    storage.get_items()
}

#[tauri::command]
pub fn delete_clip(id: String, storage: State<'_, SharedStorage>) -> bool {
    storage.delete_item(&id)
}

#[tauri::command]
pub fn toggle_pin_clip(id: String, storage: State<'_, SharedStorage>) -> Option<bool> {
    storage.toggle_pin(&id)
}

#[tauri::command]
pub fn reorder_clips(ids: Vec<String>, storage: State<'_, SharedStorage>) -> bool {
    storage.reorder_items(ids)
}

#[tauri::command]
pub fn clear_clips(keep_pinned: bool, storage: State<'_, SharedStorage>) {
    storage.clear_all(keep_pinned);
}

#[tauri::command]
pub fn copy_clip_to_clipboard(
    id: String,
    storage: State<'_, SharedStorage>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let items = storage.get_items();
    let item = items
        .iter()
        .find(|it| it.id == id)
        .ok_or_else(|| "Item not found".to_string())?;

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    IS_INTERNAL_COPY.store(true, Ordering::SeqCst);

    if item.item_type == "text" {
        clipboard
            .set_text(&item.content)
            .map_err(|e| format!("Failed to set text to clipboard: {}", e))?;
    } else if item.item_type == "image" {
        let img_path = storage.get_image_path(&item.content);
        let img_bytes = fs::read(&img_path)
            .map_err(|e| format!("Failed to read image file: {}", e))?;
        let dyn_img = image::load_from_memory(&img_bytes)
            .map_err(|e| format!("Failed to decode image: {}", e))?;
        let rgba_img = dyn_img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        let img_data = ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(rgba_img.into_raw()),
        };

        clipboard
            .set_image(img_data)
            .map_err(|e| format!("Failed to set image to clipboard: {}", e))?;
    }

    storage.bump_item(&id);
    let _ = app_handle.emit("clipboard-updated", ());

    let settings = storage.get_settings();
    if settings.close_on_copy {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings(storage: State<'_, SharedStorage>) -> AppSettings {
    storage.get_settings()
}

pub fn update_linux_autostart(enabled: bool) {
    #[cfg(target_os = "linux")]
    {
        if let Some(config_dir) = dirs::config_dir() {
            let autostart_dir = config_dir.join("autostart");
            let desktop_file = autostart_dir.join("clipvault.desktop");
            let old_desktop_file = autostart_dir.join("clipboard-manager.desktop");
            if old_desktop_file.exists() {
                let _ = std::fs::remove_file(&old_desktop_file);
            }
            if enabled {
                let exe_path = if std::path::Path::new("/usr/bin/clipvault").exists() {
                    "/usr/bin/clipvault".to_string()
                } else if let Ok(exe) = std::env::current_exe() {
                    exe.to_string_lossy().to_string()
                } else {
                    "clipvault".to_string()
                };
                let _ = std::fs::create_dir_all(&autostart_dir);
                let content = format!(
                    "[Desktop Entry]\nType=Application\nName=ClipVault\nComment=Modern Linux Clipboard Manager & Secure Vault\nExec={}\nIcon=clipvault\nTerminal=false\nCategories=Utility;\nX-GNOME-Autostart-enabled=true\n",
                    exe_path
                );
                let _ = std::fs::write(&desktop_file, content);
            } else if desktop_file.exists() {
                let _ = std::fs::remove_file(&desktop_file);
            }
        }
    }
}

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    storage: State<'_, SharedStorage>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let old_shortcut = storage.get_settings().global_shortcut;
    let new_shortcut = settings.global_shortcut.clone();
    let run_at_startup = settings.run_at_startup;

    storage.save_settings(settings)?;
    update_linux_autostart(run_at_startup);

    // Dynamically update shortcut if changed
    if old_shortcut != new_shortcut {
        let global_shortcut = app_handle.global_shortcut();
        let _ = global_shortcut.unregister_all();
        if let Ok(sc) = Shortcut::from_str(&new_shortcut) {
            let _ = global_shortcut.register(sc);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn hide_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn show_window(app_handle: AppHandle) {
    crate::toggle_main_window(&app_handle);
}

#[tauri::command]
pub fn get_vault_status(vault: State<'_, SharedVault>) -> VaultStatus {
    vault.get_status()
}

#[tauri::command]
pub fn init_vault(pin: Option<String>, vault: State<'_, SharedVault>) -> Result<(), String> {
    vault.init_vault(pin)
}

#[tauri::command]
pub fn unlock_vault(pin: String, vault: State<'_, SharedVault>) -> Result<Vec<VaultItem>, String> {
    vault.unlock(&pin)
}

#[tauri::command]
pub fn auto_unlock_vault(vault: State<'_, SharedVault>) -> Result<Vec<VaultItem>, String> {
    vault.auto_unlock()
}

#[tauri::command]
pub fn set_vault_pin(pin: String, vault: State<'_, SharedVault>) -> Result<(), String> {
    vault.set_pin(&pin)
}

#[tauri::command]
pub fn remove_vault_pin(vault: State<'_, SharedVault>) -> Result<(), String> {
    vault.remove_pin()
}

#[tauri::command]
pub fn lock_vault(vault: State<'_, SharedVault>) {
    vault.lock();
}

#[tauri::command]
pub fn get_vault_items(vault: State<'_, SharedVault>) -> Result<Vec<VaultItem>, String> {
    vault.get_items()
}

#[tauri::command]
pub fn save_vault_item(
    id: Option<String>,
    title: String,
    secret: String,
    item_type: String,
    tab: String,
    group: String,
    pinned: bool,
    file_name: Option<String>,
    file_base64: Option<String>,
    file_path: Option<String>,
    copy_to_vault: Option<bool>,
    notes: Option<String>,
    vault: State<'_, SharedVault>,
    storage: State<'_, SharedStorage>,
) -> Result<VaultItem, String> {
    use base64::Engine;
    let max_size_mb = storage.get_settings().max_vault_file_size_mb;
    let file_bytes = if let Some(b64) = file_base64 {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .map_err(|e| format!("Base64 ayrıştırma hatası: {}", e))?;
        Some(bytes)
    } else {
        None
    };

    vault.save_item(
        id,
        title,
        secret,
        item_type,
        tab,
        group,
        pinned,
        file_name,
        file_bytes,
        file_path,
        copy_to_vault,
        max_size_mb,
        notes,
    )
}

#[tauri::command]
pub fn get_vault_file_base64(
    file_id: String,
    vault: State<'_, SharedVault>,
) -> Result<String, String> {
    use base64::Engine;
    let bytes = vault.read_encrypted_file(&file_id)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
pub fn get_vault_tabs(vault: State<'_, SharedVault>) -> Result<Vec<String>, String> {
    vault.get_tabs()
}

#[tauri::command]
pub fn add_vault_tab(name: String, vault: State<'_, SharedVault>) -> Result<Vec<String>, String> {
    vault.add_tab(name)
}

#[tauri::command]
pub fn delete_vault_tab(name: String, vault: State<'_, SharedVault>) -> Result<Vec<String>, String> {
    vault.delete_tab(name)
}

#[tauri::command]
pub fn toggle_vault_item_pin(id: String, vault: State<'_, SharedVault>) -> Result<bool, String> {
    vault.toggle_item_pin(&id)
}

#[tauri::command]
pub fn set_window_size(width: f64, height: f64, app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
    }
}

#[tauri::command]
pub fn save_current_window_size(
    app_handle: AppHandle,
    storage: State<'_, SharedStorage>,
) -> Result<(f64, f64), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let phys_size = window.inner_size().map_err(|e| e.to_string())?;
        let width = ((phys_size.width as f64) / scale_factor).round();
        let height = ((phys_size.height as f64) / scale_factor).round();
        let mut settings = storage.get_settings();
        settings.window_width = width;
        settings.window_height = height;
        storage.save_settings(settings)?;
        Ok((width, height))
    } else {
        Err("Pencere bulunamadı.".into())
    }
}

#[tauri::command]
pub fn apply_window_size(
    width: f64,
    height: f64,
    save_as_default: bool,
    app_handle: AppHandle,
    storage: State<'_, SharedStorage>,
) -> Result<(f64, f64), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
        if save_as_default {
            let mut settings = storage.get_settings();
            settings.window_width = width.round();
            settings.window_height = height.round();
            storage.save_settings(settings)?;
        }
        Ok((width, height))
    } else {
        Err("Pencere bulunamadı.".into())
    }
}

#[tauri::command]
pub fn reset_window_position(app_handle: AppHandle) {
    crate::USER_HAS_DRAGGED_WINDOW.store(false, Ordering::SeqCst);
    if let Some(window) = app_handle.get_webview_window("main") {
        crate::position_bottom_right(&window);
    }
}

#[tauri::command]
pub fn notify_user_dragged() {
    crate::USER_HAS_DRAGGED_WINDOW.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn move_window_by(dx: i32, dy: i32, app_handle: AppHandle) {
    crate::USER_HAS_DRAGGED_WINDOW.store(true, Ordering::SeqCst);
    let new_x = crate::CURRENT_WINDOW_X.fetch_add(dx, Ordering::SeqCst) + dx;
    let new_y = crate::CURRENT_WINDOW_Y.fetch_add(dy, Ordering::SeqCst) + dy;

    if let Some(window) = app_handle.get_webview_window("main") {
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            if let Ok(gtk_win) = window.gtk_window() {
                gtk_win.move_(new_x, new_y);
                return;
            }
        }
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }));
    }
}

#[tauri::command]
pub fn delete_vault_item(id: String, vault: State<'_, SharedVault>) -> Result<(), String> {
    vault.delete_item(&id)
}

#[tauri::command]
pub fn copy_vault_secret(
    id: String,
    vault: State<'_, SharedVault>,
    storage: State<'_, SharedStorage>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let secret = vault.get_secret(&id)?;
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Panoya erişilemedi: {}", e))?;

    // Mark secret hash so clipboard monitor will NEVER record this secret in public history!
    let secret_hash = crate::clipboard_monitor::compute_text_hash(&secret);
    crate::clipboard_monitor::add_ignored_secret_hash(secret_hash);
    IS_INTERNAL_COPY.store(true, Ordering::SeqCst);
    clipboard
        .set_text(&secret)
        .map_err(|e| format!("Şifre panoya kopyalanamadı: {}", e))?;

    let settings = storage.get_settings();
    if settings.close_on_copy {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_vault_path(vault: State<'_, SharedVault>) -> String {
    vault.get_vault_path()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickedFileInfo {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn pick_vault_file() -> Result<Option<PickedFileInfo>, String> {
    println!("[Vault File] pick_vault_file invoked!");
    let mut cmd = std::process::Command::new("zenity");
    cmd.args(["--file-selection", "--title=Güvenli Kasaya Belge/Dosya Seç"]);
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let p = std::path::PathBuf::from(&path_str);
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "dosya".into());
                let size_bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                return Ok(Some(PickedFileInfo {
                    path: path_str,
                    name,
                    size_bytes,
                }));
            }
        }
        return Ok(None);
    }

    // Fallback: kdialog (KDE)
    if let Ok(output) = std::process::Command::new("kdialog")
        .args(["--getopenfilename", "--title", "Güvenli Kasaya Belge/Dosya Seç"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let p = std::path::PathBuf::from(&path_str);
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "dosya".into());
                let size_bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                return Ok(Some(PickedFileInfo {
                    path: path_str,
                    name,
                    size_bytes,
                }));
            }
        }
        return Ok(None);
    }

    Err("Yerel dosya seçici penceresi başlatılamadı.".into())
}

#[tauri::command]
pub fn copy_vault_file(
    id: String,
    vault: State<'_, SharedVault>,
    storage: State<'_, SharedStorage>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let (file_name, target_path, bytes) = vault.get_file_info_and_path(&id)?;
    println!("[Vault File] copy_vault_file: id={}, path={:?}", id, target_path);

    // Clean up any legacy .cache vault folder if it exists
    if let Some(c_dir) = dirs::cache_dir() {
        let old_cache1 = c_dir.join("clipboard-manager").join("vault_cache");
        if old_cache1.exists() {
            let _ = fs::remove_dir_all(&old_cache1);
        }
        let old_cache2 = c_dir.join("clipvault").join("vault_cache");
        if old_cache2.exists() {
            let _ = fs::remove_dir_all(&old_cache2);
        }
    }

    let path_str = target_path.to_string_lossy().to_string();

    #[cfg(target_os = "linux")]
    let uri_str = match gtk::glib::filename_to_uri(&target_path, None) {
        Ok(u) => u.to_string(),
        Err(_) => format!("file://{}", path_str),
    };
    #[cfg(not(target_os = "linux"))]
    let uri_str = format!("file://{}", path_str);

    let gnome_str = format!("copy\n{}\n", uri_str);

    // Register ignored secret hashes so clipboard monitor NEVER records them to public history
    crate::clipboard_monitor::add_ignored_secret_hash(crate::clipboard_monitor::compute_text_hash(&path_str));
    crate::clipboard_monitor::add_ignored_secret_hash(crate::clipboard_monitor::compute_text_hash(&uri_str));
    crate::clipboard_monitor::add_ignored_secret_hash(crate::clipboard_monitor::compute_text_hash(&gnome_str));

    let maybe_img = if !bytes.is_empty() {
        image::load_from_memory(&bytes).ok()
    } else {
        None
    };

    if let Some(ref dyn_img) = maybe_img {
        let rgba = dyn_img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let img_hash = StorageManager::compute_image_hash(w, h, &rgba);
        crate::clipboard_monitor::add_ignored_secret_hash(img_hash);
    }

    IS_INTERNAL_COPY.store(true, Ordering::SeqCst);

    #[cfg(target_os = "linux")]
    {
        use gtk::TargetEntry;
        use gtk::TargetFlags;

        let path_clone = path_str.clone();
        let uri_clone = uri_str.clone();
        let gnome_clone = gnome_str.clone();

        let png_bytes = if let Some(ref dyn_img) = maybe_img {
            let mut buf = std::io::Cursor::new(Vec::new());
            if dyn_img.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                Some(buf.into_inner())
            } else {
                None
            }
        } else {
            None
        };

        let _ = app_handle.run_on_main_thread(move || {
            let cb = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
            let mut targets = vec![
                TargetEntry::new("x-special/gnome-copied-files", TargetFlags::empty(), 0),
                TargetEntry::new("text/uri-list", TargetFlags::empty(), 1),
                TargetEntry::new("UTF8_STRING", TargetFlags::empty(), 2),
                TargetEntry::new("TEXT", TargetFlags::empty(), 3),
                TargetEntry::new("STRING", TargetFlags::empty(), 4),
                TargetEntry::new("text/plain", TargetFlags::empty(), 5),
            ];

            if png_bytes.is_some() {
                targets.push(TargetEntry::new("image/png", TargetFlags::empty(), 6));
            }

            let path_for_closure = path_clone;
            let uri_for_closure = uri_clone;
            let gnome_for_closure = gnome_clone;
            let png_for_closure = png_bytes;

            cb.set_with_data(&targets, move |_cb, sel, _info| {
                let name = sel.target().name();
                match name.as_str() {
                    "x-special/gnome-copied-files" => {
                        sel.set(&sel.target(), 8, gnome_for_closure.as_bytes());
                    }
                    "text/uri-list" => {
                        sel.set_uris(&[&uri_for_closure]);
                    }
                    "UTF8_STRING" | "TEXT" | "STRING" | "text/plain" => {
                        sel.set_text(&path_for_closure);
                    }
                    "image/png" => {
                        if let Some(ref img_data) = png_for_closure {
                            sel.set(&sel.target(), 8, img_data);
                        }
                    }
                    _ => {}
                }
            });
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(&path_str);
        }
    }

    let settings = storage.get_settings();
    if settings.close_on_copy {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    Ok(file_name)
}

#[tauri::command]
pub fn export_vault_file(
    id: String,
    vault: State<'_, SharedVault>,
) -> Result<String, String> {
    let (file_name, target_path, bytes) = vault.get_file_info_and_path(&id)?;
    println!("[Vault File] export_vault_file: id={}, path={:?}", id, target_path);

    let downloads_dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let _ = fs::create_dir_all(&downloads_dir);

    let path = std::path::Path::new(&file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("dosya");
    let ext = path.extension().and_then(|e| e.to_str());

    let mut dest_path = downloads_dir.join(&file_name);
    let mut counter = 1;
    while dest_path.exists() {
        let new_name = match ext {
            Some(e) => format!("{} ({}).{}", stem, counter, e),
            None => format!("{} ({})", stem, counter),
        };
        dest_path = downloads_dir.join(new_name);
        counter += 1;
    }

    if target_path.exists() {
        fs::copy(&target_path, &dest_path)
            .map_err(|e| format!("Dosya kopyalanamadı: {}", e))?;
    } else if !bytes.is_empty() {
        fs::write(&dest_path, &bytes)
            .map_err(|e| format!("Dosya kaydedilemedi: {}", e))?;
    } else {
        return Err("Dışa aktarılacak dosya bulunamadı.".into());
    }

    println!("[Vault File] exported successfully to: {:?}", dest_path);
    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_vault_file(
    id: String,
    vault: State<'_, SharedVault>,
) -> Result<(), String> {
    let (_, target_path, _) = vault.get_file_info_and_path(&id)?;
    println!("[Vault File] open_vault_file: id={}, path={:?}", id, target_path);

    if !target_path.exists() {
        return Err("Açılacak dosya diskte bulunamadı.".into());
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&target_path)
            .spawn()
            .map_err(|e| format!("Dosya açılamadı (xdg-open): {}", e))?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = tauri_plugin_opener::open_path(&target_path, None::<&str>)
            .map_err(|e| format!("Dosya açılamadı: {:?}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("Geçersiz web adresi (http:// veya https:// ile başlamalı).".into());
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(trimmed)
            .spawn()
            .map_err(|e| format!("Tarayıcı açılamadı (xdg-open): {}", e))?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = tauri_plugin_opener::open_url(trimmed, None::<&str>)
            .map_err(|e| format!("Tarayıcı açılamadı: {:?}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn export_clipboard_history(
    format: String,
    storage: State<'_, SharedStorage>,
) -> Result<String, String> {
    let items = storage.get_items();
    let downloads_dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let _ = fs::create_dir_all(&downloads_dir);

    let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let is_markdown = format.to_lowercase() == "markdown" || format.to_lowercase() == "md";

    let (filename, content) = if is_markdown {
        let mut md = String::new();
        md.push_str("# Pano Geçmişi\n\n");
        md.push_str(&format!("*Dışa Aktarılma Tarihi: {}*\n", chrono::Local::now().format("%d.%m.%Y %H:%M:%S")));
        md.push_str(&format!("*Toplam Öğe: {}*\n\n---\n\n", items.len()));

        for (idx, item) in items.iter().enumerate() {
            let item_date = chrono::DateTime::from_timestamp_millis(item.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M:%S").to_string())
                .unwrap_or_else(|| "-".to_string());

            let pin_badge = if item.pinned { " [📌 Sabitli]" } else { "" };
            md.push_str(&format!("### {}. {}{} ({})\n\n", idx + 1, item.item_type.to_uppercase(), pin_badge, item_date));

            if item.item_type == "text" {
                md.push_str("```text\n");
                md.push_str(&item.content);
                md.push_str("\n```\n\n");
            } else if item.item_type == "image" {
                md.push_str(&format!("*Görsel Dosyası: {}*\n", item.content));
                if let (Some(w), Some(h)) = (item.image_width, item.image_height) {
                    md.push_str(&format!("*Boyut: {}x{} px*\n\n", w, h));
                }
            }
        }
        (format!("clipboard_history_{}.md", now_str), md)
    } else {
        let mut txt = String::new();
        txt.push_str(&format!("PANO GEÇMİŞİ - {}\n", chrono::Local::now().format("%d.%m.%Y %H:%M:%S")));
        txt.push_str(&format!("Toplam Öğe: {}\n", items.len()));
        txt.push_str("========================================================\n\n");

        for (idx, item) in items.iter().enumerate() {
            let item_date = chrono::DateTime::from_timestamp_millis(item.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M:%S").to_string())
                .unwrap_or_else(|| "-".to_string());

            txt.push_str(&format!("[#{}] {} ({})\n", idx + 1, item.item_type.to_uppercase(), item_date));
            if item.item_type == "text" {
                txt.push_str(&item.content);
            } else {
                txt.push_str(&format!("[Görsel: {}]", item.content));
            }
            txt.push_str("\n--------------------------------------------------------\n\n");
        }
        (format!("clipboard_history_{}.txt", now_str), txt)
    };

    let dest_path = downloads_dir.join(filename);
    fs::write(&dest_path, content.as_bytes())
        .map_err(|e| format!("Dosya kaydedilemedi: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_vault_backup(vault: State<'_, SharedVault>) -> Result<String, String> {
    let backup_json = vault.create_backup_bundle()?;
    let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let default_name = format!("clipboard_vault_backup_{}.vaultbak", now_str);

    let downloads_dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let default_dest = downloads_dir.join(&default_name);

    // Prompt user with Zenity save dialog if available
    let dest_path = {
        let mut cmd = std::process::Command::new("zenity");
        cmd.arg("--file-selection")
            .arg("--save")
            .arg("--confirm-overwrite")
            .arg(format!("--filename={}", default_dest.to_string_lossy()))
            .arg("--title=Kasa Yedeğini Kaydedin");
        if let Ok(display) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", display);
        }

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !chosen.is_empty() {
                    PathBuf::from(chosen)
                } else {
                    default_dest
                }
            } else {
                default_dest
            }
        } else {
            default_dest
        }
    };

    fs::write(&dest_path, backup_json.as_bytes())
        .map_err(|e| format!("Yedek dosyası kaydedilemedi: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn restore_vault_backup(vault: State<'_, SharedVault>) -> Result<String, String> {
    let mut cmd = std::process::Command::new("zenity");
    cmd.arg("--file-selection")
        .arg("--title=Geri Yüklenecek .vaultbak Dosyasını Seçin")
        .arg("--file-filter=Kasa Yedek Dosyaları (*.vaultbak *.json) | *.vaultbak *.json");
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd.output().map_err(|e| format!("Dosya seçici başlatılamadı: {}", e))?;
    if !output.status.success() {
        return Err("Geri yükleme iptal edildi.".into());
    }

    let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if chosen.is_empty() {
        return Err("Geçerli bir dosya seçilmedi.".into());
    }

    let p = PathBuf::from(&chosen);
    if !p.exists() {
        return Err("Seçilen dosya diskte bulunamadı.".into());
    }

    let json = fs::read_to_string(&p)
        .map_err(|e| format!("Yedek dosyası okunamadı: {}", e))?;

    vault.restore_backup_bundle(&json)?;

    Ok(p.file_name().and_then(|n: &std::ffi::OsStr| n.to_str()).unwrap_or("yedek").to_string())
}

