use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: String,
    pub item_type: String, // "text" | "image"
    pub content: String,   // Text string or image filename
    pub preview: String,   // Text preview snippet or base64 data URI for image
    pub timestamp: i64,    // Unix timestamp in ms
    pub pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_hash: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub global_shortcut: String,
    pub max_history: usize,
    pub close_on_copy: bool,
    pub number_keys_copy: bool,
    pub play_sound: bool,
    pub run_at_startup: bool,
    pub max_vault_file_size_mb: u32,
    pub window_width: f64,
    pub window_height: f64,
    pub theme_preset: String,
    pub custom_bg_color: String,
    pub custom_card_color: String,
    pub custom_accent_color: String,
    pub custom_text_color: String,
    pub custom_secondary_text_color: String,
    pub custom_border_color: String,
    pub bg_opacity: u8,
    pub bg_image: Option<String>,
    pub bg_image_opacity: u8,
    pub bg_blur: String,

    // Keyboard Shortcuts
    pub shortcut_close: String,
    pub shortcut_copy: String,
    pub shortcut_delete: String,
    pub shortcut_pin: String,
    pub shortcut_move_up: String,
    pub shortcut_move_down: String,
    pub shortcut_search: String,
    pub shortcut_clear: String,
    pub shortcut_toggle_vault: String,
    pub shortcut_settings: String,
    pub shortcut_export: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            global_shortcut: "Alt+Shift+Q".to_string(),
            max_history: 100,
            close_on_copy: true,
            number_keys_copy: true,
            play_sound: false,
            run_at_startup: false,
            max_vault_file_size_mb: 20,
            window_width: 640.0,
            window_height: 560.0,
            theme_preset: "light".to_string(),
            custom_bg_color: "#f8fafc".to_string(),
            custom_card_color: "#ffffff".to_string(),
            custom_accent_color: "#10b981".to_string(),
            custom_text_color: "#0f172a".to_string(),
            custom_secondary_text_color: "#64748b".to_string(),
            custom_border_color: "#e2e8f0".to_string(),
            bg_opacity: 95,
            bg_image: None,
            bg_image_opacity: 25,
            bg_blur: "sm".to_string(),
            shortcut_close: "Escape".to_string(),
            shortcut_copy: "Enter".to_string(),
            shortcut_delete: "Delete".to_string(),
            shortcut_pin: "P".to_string(),
            shortcut_move_up: "Alt+ArrowUp".to_string(),
            shortcut_move_down: "Alt+ArrowDown".to_string(),
            shortcut_search: "Ctrl+F".to_string(),
            shortcut_clear: "Ctrl+Delete".to_string(),
            shortcut_toggle_vault: "Ctrl+Tab".to_string(),
            shortcut_settings: "Ctrl+,".to_string(),
            shortcut_export: "Ctrl+E".to_string(),
        }
    }
}

pub struct StorageManager {
    #[allow(dead_code)]
    data_dir: PathBuf,
    images_dir: PathBuf,
    history_file: PathBuf,
    settings_file: PathBuf,
    items: Mutex<Vec<ClipItem>>,
    settings: Mutex<AppSettings>,
}

pub type SharedStorage = Arc<StorageManager>;

impl StorageManager {
    pub fn get_base_dir() -> PathBuf {
        let root = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let new_dir = root.join("clipfort");
        let old_dir1 = root.join("clipvault");
        let old_dir2 = root.join("clipboard-manager");

        if !new_dir.exists() {
            if old_dir1.exists() {
                let _ = fs::rename(&old_dir1, &new_dir);
            } else if old_dir2.exists() {
                let _ = fs::rename(&old_dir2, &new_dir);
            }
        }
        if !new_dir.exists() {
            let _ = fs::create_dir_all(&new_dir);
        }
        new_dir
    }

    pub fn new() -> Self {
        let base_dir = Self::get_base_dir();

        let images_dir = base_dir.join("images");
        let history_file = base_dir.join("history.json");
        let settings_file = base_dir.join("settings.json");

        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        if !images_dir.exists() {
            let _ = fs::create_dir_all(&images_dir);
        }

        // Load or create settings
        let settings = if settings_file.exists() {
            fs::read_to_string(&settings_file)
                .ok()
                .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
                .unwrap_or_default()
        } else {
            let s = AppSettings::default();
            if let Ok(json) = serde_json::to_string_pretty(&s) {
                let _ = fs::write(&settings_file, json);
            }
            s
        };

        // Load or create history
        let raw_items = if history_file.exists() {
            fs::read_to_string(&history_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Vec<ClipItem>>(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Backfill missing image hashes and deduplicate history items on load
        let mut items: Vec<ClipItem> = Vec::new();
        let mut modified = false;
        for mut item in raw_items {
            if item.item_type == "image" && item.image_hash.is_none() {
                let img_path = images_dir.join(&item.content);
                if let Ok(img) = image::open(&img_path) {
                    let rgba = img.to_rgba8();
                    let hash = Self::compute_image_hash(rgba.width(), rgba.height(), &rgba);
                    item.image_hash = Some(hash);
                    modified = true;
                }
            }

            let is_dup = items.iter().any(|existing: &ClipItem| {
                if existing.item_type == "text" && item.item_type == "text" {
                    existing.content == item.content
                } else if existing.item_type == "image" && item.item_type == "image" {
                    (existing.image_hash.is_some() && item.image_hash.is_some() && existing.image_hash == item.image_hash)
                        || (existing.image_width == item.image_width
                            && existing.image_height == item.image_height
                            && existing.file_size_bytes == item.file_size_bytes)
                } else {
                    false
                }
            });
            if !is_dup {
                items.push(item);
            } else {
                modified = true;
                if item.item_type == "image" {
                    let _ = fs::remove_file(images_dir.join(&item.content));
                }
            }
        }

        if modified {
            if let Ok(json) = serde_json::to_string_pretty(&items) {
                let _ = fs::write(&history_file, json);
            }
        }

        Self {
            data_dir: base_dir,
            images_dir,
            history_file,
            settings_file,
            items: Mutex::new(items),
            settings: Mutex::new(settings),
        }
    }

    pub fn get_items(&self) -> Vec<ClipItem> {
        let items = self.items.lock().unwrap();
        items.clone()
    }

    pub fn get_settings(&self) -> AppSettings {
        let settings = self.settings.lock().unwrap();
        settings.clone()
    }

    pub fn save_settings(&self, new_settings: AppSettings) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&self.settings_file, json)
            .map_err(|e| format!("Failed to save settings file: {}", e))?;
        let mut settings = self.settings.lock().unwrap();
        *settings = new_settings;
        Ok(())
    }

    pub fn add_text_item(&self, text: String) -> Option<ClipItem> {
        if text.trim().is_empty() {
            return None;
        }

        let mut items = self.items.lock().unwrap();

        // Check if top item is identical
        if let Some(first) = items.first() {
            if first.item_type == "text" && first.content == text {
                return None;
            }
        }

        // If duplicate exists elsewhere in history, remove old instance so we bring it to the top
        if let Some(pos) = items.iter().position(|it| it.item_type == "text" && it.content == text) {
            items.remove(pos);
        }

        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let preview = if char_count > 300 {
            let end: String = text.chars().take(300).collect();
            format!("{}...", end)
        } else {
            text.clone()
        };

        let item = ClipItem {
            id: Uuid::new_v4().to_string(),
            item_type: "text".to_string(),
            content: text,
            preview,
            timestamp: Utc::now().timestamp_millis(),
            pinned: false,
            char_count: Some(char_count),
            word_count: Some(word_count),
            image_width: None,
            image_height: None,
            file_size_bytes: None,
            image_hash: None,
        };

        items.insert(0, item.clone());
        self.enforce_max_limit_locked(&mut items);
        self.save_items_locked(&items);

        Some(item)
    }

    pub fn compute_image_hash(width: u32, height: u32, rgba_bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        hash ^= width as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= height as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= rgba_bytes.len() as u64;
        hash = hash.wrapping_mul(0x100000001b3);

        let step = (rgba_bytes.len() / 4096).max(1);
        for chunk in rgba_bytes.chunks(step) {
            if let Some(&b) = chunk.first() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash
    }

    pub fn add_image_item(
        &self,
        width: u32,
        height: u32,
        rgba_bytes: &[u8],
    ) -> Result<Option<ClipItem>, String> {
        let hash = Self::compute_image_hash(width, height, rgba_bytes);

        let mut items = self.items.lock().unwrap();

        // 1. If top item is identical image, skip!
        if let Some(first) = items.first() {
            if first.item_type == "image" && first.image_hash == Some(hash) {
                return Ok(None);
            }
        }

        // 2. If duplicate image exists elsewhere in history, remove old instance so it moves to top
        if let Some(pos) = items.iter().position(|it| it.item_type == "image" && it.image_hash == Some(hash)) {
            let removed = items.remove(pos);
            let _ = fs::remove_file(self.images_dir.join(removed.content));
        }

        let image_filename = format!("{}.png", Uuid::new_v4());
        let image_path = self.images_dir.join(&image_filename);

        // Convert raw RGBA to PNG and save to disk
        let img_buffer = image::RgbaImage::from_raw(width, height, rgba_bytes.to_vec())
            .ok_or_else(|| "Failed to construct image buffer from RGBA bytes".to_string())?;

        img_buffer
            .save(&image_path)
            .map_err(|e| format!("Failed to save image to disk: {}", e))?;

        let file_size = fs::metadata(&image_path)
            .map(|m| m.len() as usize)
            .unwrap_or(rgba_bytes.len());

        // Generate thumbnail preview
        let dyn_img = image::DynamicImage::ImageRgba8(img_buffer);
        let thumb = dyn_img.thumbnail(160, 100);
        let mut thumb_png_bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut thumb_png_bytes);
        thumb
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

        use base64::Engine;
        let base64_thumb = base64::engine::general_purpose::STANDARD.encode(&thumb_png_bytes);
        let preview = format!("data:image/png;base64,{}", base64_thumb);

        let item = ClipItem {
            id: Uuid::new_v4().to_string(),
            item_type: "image".to_string(),
            content: image_filename,
            preview,
            timestamp: Utc::now().timestamp_millis(),
            pinned: false,
            char_count: None,
            word_count: None,
            image_width: Some(width),
            image_height: Some(height),
            file_size_bytes: Some(file_size),
            image_hash: Some(hash),
        };

        items.insert(0, item.clone());
        self.enforce_max_limit_locked(&mut items);
        self.save_items_locked(&items);

        Ok(Some(item))
    }

    pub fn delete_item(&self, id: &str) -> bool {
        let mut items = self.items.lock().unwrap();
        if let Some(pos) = items.iter().position(|it| it.id == id) {
            let removed = items.remove(pos);
            if removed.item_type == "image" {
                let img_path = self.images_dir.join(&removed.content);
                let _ = fs::remove_file(img_path);
            }
            self.save_items_locked(&items);
            true
        } else {
            false
        }
    }

    pub fn toggle_pin(&self, id: &str) -> Option<bool> {
        let mut items = self.items.lock().unwrap();
        if let Some(item) = items.iter_mut().find(|it| it.id == id) {
            item.pinned = !item.pinned;
            let status = item.pinned;
            self.save_items_locked(&items);
            Some(status)
        } else {
            None
        }
    }

    pub fn reorder_items(&self, ordered_ids: Vec<String>) -> bool {
        let mut items = self.items.lock().unwrap();
        let mut new_list: Vec<ClipItem> = Vec::with_capacity(items.len());

        for id in &ordered_ids {
            if let Some(pos) = items.iter().position(|it| it.id == *id) {
                new_list.push(items.remove(pos));
            }
        }
        // Append any items that weren't in ordered_ids
        new_list.extend(items.drain(..));
        *items = new_list;
        self.save_items_locked(&items);
        true
    }

    pub fn bump_item(&self, id: &str) -> Option<ClipItem> {
        let mut items = self.items.lock().unwrap();
        if let Some(pos) = items.iter().position(|it| it.id == id) {
            let mut item = items.remove(pos);
            item.timestamp = Utc::now().timestamp_millis();
            let insert_idx = if item.pinned {
                0
            } else {
                items.iter().position(|it| !it.pinned).unwrap_or(0)
            };
            items.insert(insert_idx, item.clone());
            self.save_items_locked(&items);
            Some(item)
        } else {
            None
        }
    }

    pub fn clear_all(&self, keep_pinned: bool) {
        let mut items = self.items.lock().unwrap();
        if keep_pinned {
            let mut retained: Vec<ClipItem> = Vec::new();
            for item in items.drain(..) {
                if item.pinned {
                    retained.push(item);
                } else if item.item_type == "image" {
                    let _ = fs::remove_file(self.images_dir.join(&item.content));
                }
            }
            *items = retained;
        } else {
            for item in items.iter() {
                if item.item_type == "image" {
                    let _ = fs::remove_file(self.images_dir.join(&item.content));
                }
            }
            items.clear();
        }
        self.save_items_locked(&items);
    }

    pub fn get_image_path(&self, filename: &str) -> PathBuf {
        self.images_dir.join(filename)
    }

    fn enforce_max_limit_locked(&self, items: &mut Vec<ClipItem>) {
        let max_limit = self.settings.lock().unwrap().max_history;
        if items.len() <= max_limit {
            return;
        }

        // Count unpinned items to remove from the back
        while items.len() > max_limit {
            if let Some(last_unpinned_pos) = items.iter().rposition(|it| !it.pinned) {
                let removed = items.remove(last_unpinned_pos);
                if removed.item_type == "image" {
                    let _ = fs::remove_file(self.images_dir.join(&removed.content));
                }
            } else {
                break;
            }
        }
    }

    fn save_items_locked(&self, items: &[ClipItem]) {
        if let Ok(json) = serde_json::to_string_pretty(items) {
            let _ = fs::write(&self.history_file, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_text_operations() {
        let temp_dir = std::env::temp_dir().join(format!("test_cb_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let storage = StorageManager {
            data_dir: temp_dir.clone(),
            images_dir: temp_dir.join("images"),
            history_file: temp_dir.join("history.json"),
            settings_file: temp_dir.join("settings.json"),
            items: Mutex::new(Vec::new()),
            settings: Mutex::new(AppSettings::default()),
        };
        let _ = fs::create_dir_all(&storage.images_dir);

        // 1. Add text item
        let item1 = storage.add_text_item("Hello World".to_string()).unwrap();
        assert_eq!(item1.content, "Hello World");
        assert_eq!(item1.word_count, Some(2));
        assert_eq!(item1.char_count, Some(11));
        assert_eq!(storage.get_items().len(), 1);

        // 2. Add another item
        let item2 = storage.add_text_item("Second Note".to_string()).unwrap();
        assert_eq!(storage.get_items().len(), 2);
        assert_eq!(storage.get_items()[0].id, item2.id);

        // 3. Add duplicate text -> should move to top and keep count 2
        let item1_dup = storage.add_text_item("Hello World".to_string()).unwrap();
        let items = storage.get_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "Hello World");

        // 4. Toggle pin
        let pin_res = storage.toggle_pin(&item1_dup.id);
        assert_eq!(pin_res, Some(true));
        assert!(storage.get_items()[0].pinned);

        // 5. Reorder
        let ids = vec![item2.id.clone(), item1_dup.id.clone()];
        storage.reorder_items(ids);
        assert_eq!(storage.get_items()[0].id, item2.id);

        // 6. Clear unpinned -> item1_dup should remain because it's pinned
        storage.clear_all(true);
        let items_after_clear = storage.get_items();
        assert_eq!(items_after_clear.len(), 1);
        assert_eq!(items_after_clear[0].id, item1_dup.id);

        // 7. Delete item
        let del_res = storage.delete_item(&item1_dup.id);
        assert!(del_res);
        assert_eq!(storage.get_items().len(), 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_storage_image_operations() {
        let temp_dir = std::env::temp_dir().join(format!("test_cb_img_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let storage = StorageManager {
            data_dir: temp_dir.clone(),
            images_dir: temp_dir.join("images"),
            history_file: temp_dir.join("history.json"),
            settings_file: temp_dir.join("settings.json"),
            items: Mutex::new(Vec::new()),
            settings: Mutex::new(AppSettings::default()),
        };
        let _ = fs::create_dir_all(&storage.images_dir);

        // 2x2 red pixel RGBA buffer
        let rgba_data = vec![
            255, 0, 0, 255,   255, 0, 0, 255,
            255, 0, 0, 255,   255, 0, 0, 255,
        ];

        let img_item = storage.add_image_item(2, 2, &rgba_data).unwrap().unwrap();
        assert_eq!(img_item.item_type, "image");
        assert_eq!(img_item.image_width, Some(2));
        assert_eq!(img_item.image_height, Some(2));
        assert!(img_item.preview.starts_with("data:image/png;base64,"));

        // Adding identical image should return Ok(None) and not duplicate
        let dup_res = storage.add_image_item(2, 2, &rgba_data).unwrap();
        assert!(dup_res.is_none());
        assert_eq!(storage.get_items().len(), 1);

        let img_path = storage.get_image_path(&img_item.content);
        assert!(img_path.exists());

        // Delete image item and verify file is cleaned up
        assert!(storage.delete_item(&img_item.id));
        assert!(!img_path.exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_bump_item() {
        let temp_dir = std::env::temp_dir().join(format!("test_cb_bump_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let storage = StorageManager {
            data_dir: temp_dir.clone(),
            images_dir: temp_dir.join("images"),
            history_file: temp_dir.join("history.json"),
            settings_file: temp_dir.join("settings.json"),
            items: Mutex::new(Vec::new()),
            settings: Mutex::new(AppSettings::default()),
        };

        let item1 = storage.add_text_item("First Item".into()).unwrap();
        let item2 = storage.add_text_item("Second Item".into()).unwrap();
        let item3 = storage.add_text_item("Third Item".into()).unwrap();

        // Initially order is: item3, item2, item1
        let items = storage.get_items();
        assert_eq!(items[0].id, item3.id);
        assert_eq!(items[1].id, item2.id);
        assert_eq!(items[2].id, item1.id);

        // Bump item1 (which was at index 2)
        storage.bump_item(&item1.id).expect("bump item1 failed");
        let items_after = storage.get_items();
        assert_eq!(items_after[0].id, item1.id, "Bumped item must now be at index 0");
        assert_eq!(items_after[1].id, item3.id);
        assert_eq!(items_after[2].id, item2.id);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_app_settings_backward_compatibility_and_shortcuts() {
        // 1. Deserializing legacy settings JSON (without new shortcut fields)
        let legacy_json = r#"{
            "global_shortcut": "Ctrl+Shift+V",
            "max_history": 50,
            "close_on_copy": true,
            "play_sound": false,
            "run_at_startup": true
        }"#;

        let settings: AppSettings = serde_json::from_str(legacy_json)
            .expect("Failed to deserialize legacy settings JSON");

        // Verify custom settings preserved
        assert_eq!(settings.global_shortcut, "Ctrl+Shift+V");
        assert_eq!(settings.max_history, 50);
        assert!(settings.run_at_startup);

        // Verify default shortcuts automatically applied
        assert_eq!(settings.shortcut_close, "Escape");
        assert_eq!(settings.shortcut_copy, "Enter");
        assert_eq!(settings.shortcut_delete, "Delete");
        assert_eq!(settings.shortcut_pin, "P");
        assert_eq!(settings.shortcut_move_up, "Alt+ArrowUp");
        assert_eq!(settings.shortcut_move_down, "Alt+ArrowDown");
        assert_eq!(settings.shortcut_search, "Ctrl+F");
        assert_eq!(settings.shortcut_clear, "Ctrl+Delete");
        assert_eq!(settings.shortcut_toggle_vault, "Ctrl+Tab");
        assert_eq!(settings.shortcut_settings, "Ctrl+,");
        assert_eq!(settings.shortcut_export, "Ctrl+E");

        // 2. Modifying shortcuts and saving
        let temp_dir = std::env::temp_dir().join(format!("test_cb_settings_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let storage = StorageManager {
            data_dir: temp_dir.clone(),
            images_dir: temp_dir.join("images"),
            history_file: temp_dir.join("history.json"),
            settings_file: temp_dir.join("settings.json"),
            items: Mutex::new(Vec::new()),
            settings: Mutex::new(settings.clone()),
        };

        let mut updated = settings;
        updated.shortcut_copy = "Space".to_string();
        updated.shortcut_pin = "Alt+P".to_string();
        storage.save_settings(updated.clone()).unwrap();

        let loaded = storage.get_settings();
        assert_eq!(loaded.shortcut_copy, "Space");
        assert_eq!(loaded.shortcut_pin, "Alt+P");

        let _ = fs::remove_dir_all(temp_dir);
    }
}

