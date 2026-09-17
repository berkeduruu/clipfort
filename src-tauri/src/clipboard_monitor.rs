use crate::storage::{SharedStorage, StorageManager};
use arboard::Clipboard;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub static IS_INTERNAL_COPY: AtomicBool = AtomicBool::new(false);
static IGNORED_SECRET_HASHES: Mutex<Option<HashSet<u64>>> = Mutex::new(None);

pub fn compute_text_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

pub fn add_ignored_secret_hash(hash: u64) {
    let mut guard = IGNORED_SECRET_HASHES.lock().unwrap();
    let set = guard.get_or_insert_with(HashSet::new);
    if set.len() > 200 {
        set.clear();
    }
    set.insert(hash);
}

pub fn is_ignored_secret_hash(hash: u64) -> bool {
    let guard = IGNORED_SECRET_HASHES.lock().unwrap();
    if let Some(set) = guard.as_ref() {
        set.contains(&hash)
    } else {
        false
    }
}

pub fn is_ignored_text(text: &str) -> bool {
    if text.contains("vault_cache") || text.contains("vault_files") || text.contains("vault_storage") {
        return true;
    }
    let hash = compute_text_hash(text);
    is_ignored_secret_hash(hash)
}

pub fn start_clipboard_monitor(app_handle: AppHandle, storage: SharedStorage) {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                eprintln!("[Clipboard Monitor] Failed to initialize clipboard: {}", e);
                return;
            }
        };

        let mut last_text: Option<String> = None;
        let mut last_image_hash: Option<u64> = None;

        // Immediately capture current clipboard item on startup if present
        if let Ok(current_text) = clipboard.get_text() {
            if !current_text.trim().is_empty() {
                last_text = Some(current_text.clone());
                if !is_ignored_text(&current_text) {
                    if let Some(item) = storage.add_text_item(current_text) {
                        println!("[Clipboard Monitor] Captured initial text: {} chars", item.char_count.unwrap_or(0));
                        let _ = app_handle.emit("clipboard-updated", ());
                    }
                }
            }
        } else if let Ok(img) = clipboard.get_image() {
            let hash = StorageManager::compute_image_hash(img.width as u32, img.height as u32, &img.bytes);
            last_image_hash = Some(hash);
            if let Ok(Some(_item)) = storage.add_image_item(img.width as u32, img.height as u32, &img.bytes) {
                println!("[Clipboard Monitor] Captured initial image: {}x{}", img.width, img.height);
                let _ = app_handle.emit("clipboard-updated", ());
            }
        }

        loop {
            thread::sleep(Duration::from_millis(250));

            // If we just copied an item from our own app, skip processing this tick
            if IS_INTERNAL_COPY.swap(false, Ordering::SeqCst) {
                if let Ok(text) = clipboard.get_text() {
                    last_text = Some(text);
                }
                if let Ok(img) = clipboard.get_image() {
                    last_image_hash = Some(StorageManager::compute_image_hash(
                        img.width as u32,
                        img.height as u32,
                        &img.bytes,
                    ));
                }
                continue;
            }

            // 1. Check for text updates
            let mut detected_text_change = false;
            match clipboard.get_text() {
                Ok(text) => {
                    if !text.trim().is_empty() {
                        let is_new = match &last_text {
                            Some(prev) => prev != &text,
                            None => true,
                        };

                        if is_new {
                            last_text = Some(text.clone());
                            last_image_hash = None;
                            detected_text_change = true;

                            if is_ignored_text(&text) {
                                println!("[Clipboard Monitor] Skipped adding vault secret to public clipboard history");
                                continue;
                            }

                            if let Some(item) = storage.add_text_item(text) {
                                println!("[Clipboard Monitor] New text item added: {} chars", item.char_count.unwrap_or(0));
                                let _ = app_handle.emit("clipboard-updated", ());
                            }
                        }
                    }
                }
                Err(_) => {
                    // Normal when clipboard contains image, binary, or is temporarily empty
                }
            }

            // 2. Check for image updates if text didn't change
            if !detected_text_change {
                if let Ok(img) = clipboard.get_image() {
                    let hash = StorageManager::compute_image_hash(
                        img.width as u32,
                        img.height as u32,
                        &img.bytes,
                    );
                    let is_new = match last_image_hash {
                        Some(prev) => prev != hash,
                        None => true,
                    };

                    if is_new {
                        last_image_hash = Some(hash);
                        last_text = None;

                        if is_ignored_secret_hash(hash) {
                            println!("[Clipboard Monitor] Skipped adding vault secret image to public clipboard history");
                            continue;
                        }

                        if let Ok(Some(_item)) = storage.add_image_item(
                            img.width as u32,
                            img.height as u32,
                            &img.bytes,
                        ) {
                            println!("[Clipboard Monitor] New image item added: {}x{}", img.width, img.height);
                            let _ = app_handle.emit("clipboard-updated", ());
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polling_x11() {
        println!("DISPLAY: {:?}", std::env::var("DISPLAY"));
        let mut cb = Clipboard::new().expect("Clipboard new");
        for i in 0..3 {
            let res = cb.get_text();
            println!("Poll #{}: {:?}", i, res);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
