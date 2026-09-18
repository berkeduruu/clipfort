use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const VAULT_MAGIC_PIN: &[u8; 9] = b"CP_VAULT1";
pub const VAULT_MAGIC_AUTO: &[u8; 9] = b"CP_VAULTA";
const MAGIC_LEN: usize = 9;
const PBKDF2_ROUNDS: u32 = 100_000;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

fn default_item_type() -> String {
    "password".to_string()
}
fn default_tab() -> String {
    "Personal Info".to_string()
}
fn default_group() -> String {
    "Personal Data".to_string()
}
fn default_tabs() -> Vec<String> {
    vec![
        "Personal Info".to_string(),
        "Websites & Accounts".to_string(),
        "Development & Keys".to_string(),
        "Notes & Documents".to_string(),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultItem {
    pub id: String,
    pub title: String,
    pub secret: String,
    #[serde(default = "default_item_type")]
    pub item_type: String, // "text", "password", "file", "note", "link"
    #[serde(default = "default_tab")]
    pub tab: String, // e.g. "Personal Info", "Websites & Accounts"
    #[serde(default = "default_group")]
    pub group: String, // e.g. "Personal Data", "Documents", "My Store"
    #[serde(default)]
    pub pinned: bool,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<usize>,
    pub file_id: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultData {
    pub version: u32,
    #[serde(default = "default_tabs")]
    pub tabs: Vec<String>,
    pub items: Vec<VaultItem>,
}

impl Default for VaultData {
    fn default() -> Self {
        Self {
            version: 1,
            tabs: default_tabs(),
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultStatus {
    pub is_initialized: bool,
    pub is_unlocked: bool,
    pub has_pin: bool,
    pub vault_path: String,
    pub item_count: usize,
    pub tabs: Vec<String>,
}

pub struct VaultSession {
    pub key: [u8; 32],
    pub salt: [u8; SALT_LEN],
    pub has_pin: bool,
    pub data: VaultData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupFileEntry {
    pub file_name: String,
    pub base64_data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultBackupData {
    pub version: u32,
    pub created_at: String,
    pub vault_enc_base64: String,
    pub files: Vec<BackupFileEntry>,
    pub storage_files: Vec<BackupFileEntry>,
}

pub struct VaultManager {
    vault_file: PathBuf,
    vault_files_dir: PathBuf,
    vault_storage_dir: PathBuf,
    device_key_file: PathBuf,
    session: Mutex<Option<VaultSession>>,
}

pub type SharedVault = Arc<VaultManager>;

impl VaultManager {
    pub fn new() -> Self {
        let root = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let base_dir = root.join("clipfort");
        let old_dir1 = root.join("clipvault");
        let old_dir2 = root.join("clipboard-manager");

        if !base_dir.exists() {
            if old_dir1.exists() {
                let _ = fs::rename(&old_dir1, &base_dir);
            } else if old_dir2.exists() {
                let _ = fs::rename(&old_dir2, &base_dir);
            }
        }

        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }

        let vault_file = base_dir.join("vault.enc");
        let vault_files_dir = base_dir.join("vault_files");
        let vault_storage_dir = base_dir.join("vault_storage");
        let device_key_file = base_dir.join(".vault_key");

        if !vault_files_dir.exists() {
            let _ = fs::create_dir_all(&vault_files_dir);
        }
        if !vault_storage_dir.exists() {
            let _ = fs::create_dir_all(&vault_storage_dir);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&vault_storage_dir, fs::Permissions::from_mode(0o700));
        }

        Self {
            vault_file,
            vault_files_dir,
            vault_storage_dir,
            device_key_file,
            session: Mutex::new(None),
        }
    }

    #[cfg(test)]
    pub fn new_with_path(path: PathBuf, files_dir: PathBuf) -> Self {
        if !files_dir.exists() {
            let _ = fs::create_dir_all(&files_dir);
        }
        let vault_storage_dir = files_dir.parent().unwrap_or(Path::new(".")).join("vault_storage");
        if !vault_storage_dir.exists() {
            let _ = fs::create_dir_all(&vault_storage_dir);
        }
        let device_key_file = path.parent().unwrap_or(Path::new(".")).join(".vault_key");
        Self {
            vault_file: path,
            vault_files_dir: files_dir,
            vault_storage_dir,
            device_key_file,
            session: Mutex::new(None),
        }
    }

    pub fn get_vault_path(&self) -> String {
        self.vault_file.to_string_lossy().to_string()
    }

    pub fn is_initialized(&self) -> bool {
        self.vault_file.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    fn check_file_has_pin(&self) -> bool {
        if !self.vault_file.exists() {
            return false;
        }
        if let Ok(bytes) = fs::read(&self.vault_file) {
            if bytes.len() >= MAGIC_LEN {
                return &bytes[0..MAGIC_LEN] == VAULT_MAGIC_PIN;
            }
        }
        false
    }

    pub fn get_status(&self) -> VaultStatus {
        let session_guard = self.session.lock().unwrap();
        let is_unlocked = session_guard.is_some();
        let (item_count, tabs, has_pin) = if let Some(s) = session_guard.as_ref() {
            (s.data.items.len(), s.data.tabs.clone(), s.has_pin)
        } else {
            (0, default_tabs(), self.check_file_has_pin())
        };

        VaultStatus {
            is_initialized: self.is_initialized(),
            is_unlocked,
            has_pin,
            vault_path: self.get_vault_path(),
            item_count,
            tabs,
        }
    }

    pub fn get_or_create_device_key(&self) -> Result<[u8; 32], String> {
        if self.device_key_file.exists() {
            if let Ok(bytes) = fs::read(&self.device_key_file) {
                if bytes.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    return Ok(key);
                }
            }
        }

        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&self.device_key_file)
                .map_err(|e| format!("Failed to create device key: {}", e))?;
            file.write_all(&key)
                .map_err(|e| format!("Failed to write device key: {}", e))?;
        }
        #[cfg(not(unix))]
        {
            fs::write(&self.device_key_file, &key)
                .map_err(|e| format!("Failed to write device key: {}", e))?;
        }

        Ok(key)
    }

    fn derive_key(pin: &str, salt: &[u8; SALT_LEN]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(
            pin.as_bytes(),
            salt,
            PBKDF2_ROUNDS,
            &mut key,
        );
        key
    }

    fn encrypt_and_save(
        vault_file: &Path,
        magic: &[u8; 9],
        key: &[u8; 32],
        salt: &[u8; SALT_LEN],
        data: &VaultData,
    ) -> Result<(), String> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("Cipher init error: {}", e))?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let json_bytes = serde_json::to_vec(data)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let ciphertext = cipher
            .encrypt(nonce, json_bytes.as_ref())
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Format: [MAGIC (9)] [SALT (32)] [NONCE (12)] [CIPHERTEXT + TAG]
        let mut file_payload = Vec::with_capacity(MAGIC_LEN + SALT_LEN + NONCE_LEN + ciphertext.len());
        file_payload.extend_from_slice(magic);
        file_payload.extend_from_slice(salt);
        file_payload.extend_from_slice(&nonce_bytes);
        file_payload.extend_from_slice(&ciphertext);

        fs::write(vault_file, file_payload)
            .map_err(|e| format!("Failed to write vault file: {}", e))?;

        Ok(())
    }

    fn save_session_data(&self, session: &VaultSession) -> Result<(), String> {
        let magic = if session.has_pin {
            VAULT_MAGIC_PIN
        } else {
            VAULT_MAGIC_AUTO
        };
        Self::encrypt_and_save(
            &self.vault_file,
            magic,
            &session.key,
            &session.salt,
            &session.data,
        )
    }

    fn read_and_decrypt(&self, pin: Option<&str>) -> Result<([u8; 32], [u8; SALT_LEN], bool, VaultData), String> {
        let bytes = fs::read(&self.vault_file)
            .map_err(|e| format!("Failed to read vault file: {}", e))?;

        let min_len = MAGIC_LEN + SALT_LEN + NONCE_LEN + 16; // 16 bytes auth tag minimum
        if bytes.len() < min_len {
            return Err("Vault file is invalid or corrupted (file size too small).".into());
        }

        let magic = &bytes[0..MAGIC_LEN];
        let is_pin_vault = magic == VAULT_MAGIC_PIN;
        let is_auto_vault = magic == VAULT_MAGIC_AUTO;

        if !is_pin_vault && !is_auto_vault {
            return Err("Invalid vault format (magic header mismatch).".into());
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&bytes[MAGIC_LEN..MAGIC_LEN + SALT_LEN]);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        let nonce_start = MAGIC_LEN + SALT_LEN;
        nonce_bytes.copy_from_slice(&bytes[nonce_start..nonce_start + NONCE_LEN]);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = &bytes[nonce_start + NONCE_LEN..];

        let key = if is_pin_vault {
            let p = pin.ok_or_else(|| "This vault is PIN-locked. Please enter your PIN.".to_string())?;
            let trimmed = p.trim();
            if trimmed.is_empty() {
                return Err("Please enter your PIN.".into());
            }
            Self::derive_key(trimmed, &salt)
        } else {
            self.get_or_create_device_key()?
        };

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher init error: {}", e))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| {
                if is_pin_vault {
                    "Incorrect PIN! Failed to unlock vault.".to_string()
                } else {
                    "Device encryption key verification failed! Could not decrypt vault.".to_string()
                }
            })?;

        let mut data: VaultData = serde_json::from_slice(&plaintext)
            .map_err(|e| format!("Failed to parse vault data: {}", e))?;

        if data.tabs.is_empty() {
            data.tabs = default_tabs();
        }

        Ok((key, salt, is_pin_vault, data))
    }

    pub fn init_vault(&self, pin: Option<String>) -> Result<(), String> {
        if self.is_initialized() {
            return Err("Vault already exists! Delete the existing file to reset.".into());
        }

        let pin_trimmed = pin.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());

        let (key, salt, magic, has_pin) = match pin_trimmed {
            Some(p) => {
                if p.len() < 4 {
                    return Err("PIN must be at least 4 characters.".into());
                }
                let mut salt = [0u8; SALT_LEN];
                rand::thread_rng().fill_bytes(&mut salt);
                let key = Self::derive_key(p, &salt);
                (key, salt, VAULT_MAGIC_PIN, true)
            }
            None => {
                let key = self.get_or_create_device_key()?;
                let mut salt = [0u8; SALT_LEN];
                rand::thread_rng().fill_bytes(&mut salt);
                (key, salt, VAULT_MAGIC_AUTO, false)
            }
        };

        let data = VaultData::default();
        Self::encrypt_and_save(&self.vault_file, magic, &key, &salt, &data)?;

        // Immediately unlock for this session
        let mut session_guard = self.session.lock().unwrap();
        *session_guard = Some(VaultSession {
            key,
            salt,
            has_pin,
            data,
        });

        Ok(())
    }

    fn unlock_internal(&self, pin: Option<&str>) -> Result<Vec<VaultItem>, String> {
        if !self.is_initialized() {
            return Err("Vault has not been created yet.".into());
        }

        let (key, salt, has_pin, data) = self.read_and_decrypt(pin)?;
        let items = data.items.clone();
        *self.session.lock().unwrap() = Some(VaultSession {
            key,
            salt,
            has_pin,
            data,
        });

        Ok(items)
    }

    pub fn unlock(&self, pin: &str) -> Result<Vec<VaultItem>, String> {
        self.unlock_internal(Some(pin))
    }

    pub fn auto_unlock(&self) -> Result<Vec<VaultItem>, String> {
        self.unlock_internal(None)
    }

    fn rekey_file(&self, file_id: &str, old_key: &[u8; 32], new_key: &[u8; 32]) -> Result<(), String> {
        let path = self.vault_files_dir.join(format!("{}.enc", file_id));
        if !path.exists() {
            return Ok(());
        }
        let bytes = fs::read(&path)
            .map_err(|e| format!("Failed to read attached file: {}", e))?;
        if bytes.len() < NONCE_LEN + 16 {
            return Ok(());
        }

        let mut old_nonce_bytes = [0u8; NONCE_LEN];
        old_nonce_bytes.copy_from_slice(&bytes[0..NONCE_LEN]);
        let old_nonce = Nonce::from_slice(&old_nonce_bytes);
        let ciphertext = &bytes[NONCE_LEN..];

        let old_cipher = Aes256Gcm::new_from_slice(old_key)
            .map_err(|e| format!("Old key error: {}", e))?;
        let plaintext = old_cipher
            .decrypt(old_nonce, ciphertext)
            .map_err(|e| format!("Failed to decrypt file: {}", e))?;

        let new_cipher = Aes256Gcm::new_from_slice(new_key)
            .map_err(|e| format!("New key error: {}", e))?;
        let mut new_nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut new_nonce_bytes);
        let new_nonce = Nonce::from_slice(&new_nonce_bytes);

        let new_ciphertext = new_cipher
            .encrypt(new_nonce, plaintext.as_ref())
            .map_err(|e| format!("New file encryption error: {}", e))?;

        let mut payload = Vec::with_capacity(NONCE_LEN + new_ciphertext.len());
        payload.extend_from_slice(&new_nonce_bytes);
        payload.extend_from_slice(&new_ciphertext);

        fs::write(&path, payload)
            .map_err(|e| format!("Failed to write re-encrypted file: {}", e))?;

        Ok(())
    }

    pub fn set_pin(&self, new_pin: &str) -> Result<(), String> {
        let new_pin = new_pin.trim();
        if new_pin.len() < 4 {
            return Err("PIN must be at least 4 characters.".into());
        }

        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let old_key = session.key;
        let mut new_salt = [0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut new_salt);
        let new_key = Self::derive_key(new_pin, &new_salt);

        // Rekey all attached files
        for item in &session.data.items {
            if let Some(fid) = &item.file_id {
                self.rekey_file(fid, &old_key, &new_key)?;
            }
        }

        Self::encrypt_and_save(
            &self.vault_file,
            VAULT_MAGIC_PIN,
            &new_key,
            &new_salt,
            &session.data,
        )?;

        session.key = new_key;
        session.salt = new_salt;
        session.has_pin = true;

        Ok(())
    }

    pub fn remove_pin(&self) -> Result<(), String> {
        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let old_key = session.key;
        let new_key = self.get_or_create_device_key()?;
        let mut new_salt = [0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut new_salt);

        // Rekey all attached files
        for item in &session.data.items {
            if let Some(fid) = &item.file_id {
                self.rekey_file(fid, &old_key, &new_key)?;
            }
        }

        Self::encrypt_and_save(
            &self.vault_file,
            VAULT_MAGIC_AUTO,
            &new_key,
            &new_salt,
            &session.data,
        )?;

        session.key = new_key;
        session.salt = new_salt;
        session.has_pin = false;

        Ok(())
    }

    pub fn lock(&self) {
        let mut session_guard = self.session.lock().unwrap();
        *session_guard = None;
    }

    pub fn get_items(&self) -> Result<Vec<VaultItem>, String> {
        let session_guard = self.session.lock().unwrap();
        match &*session_guard {
            Some(session) => Ok(session.data.items.clone()),
            None => Err("Vault is locked. Please unlock first.".into()),
        }
    }

    pub fn get_tabs(&self) -> Result<Vec<String>, String> {
        let session_guard = self.session.lock().unwrap();
        match &*session_guard {
            Some(session) => Ok(session.data.tabs.clone()),
            None => Ok(default_tabs()),
        }
    }

    pub fn add_tab(&self, name: String) -> Result<Vec<String>, String> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("Tab name cannot be empty.".into());
        }

        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        if session.data.tabs.contains(&name) {
            return Ok(session.data.tabs.clone());
        }

        session.data.tabs.push(name);
        self.save_session_data(session)?;

        Ok(session.data.tabs.clone())
    }

    pub fn delete_tab(&self, name: String) -> Result<Vec<String>, String> {
        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        session.data.tabs.retain(|t| t != &name);
        if session.data.tabs.is_empty() {
            session.data.tabs = default_tabs();
        }

        self.save_session_data(session)?;

        Ok(session.data.tabs.clone())
    }

    pub fn toggle_item_pin(&self, id: &str) -> Result<bool, String> {
        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let item = session
            .data
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| "Item not found.".to_string())?;

        item.pinned = !item.pinned;
        let new_state = item.pinned;

        self.save_session_data(session)?;

        Ok(new_state)
    }

    // Encrypt and store a file attachment in vault_files/<file_id>.enc
    pub fn save_encrypted_file(
        &self,
        file_bytes: &[u8],
        max_size_mb: u32,
    ) -> Result<(String, usize), String> {
        let limit_bytes = (max_size_mb as usize) * 1024 * 1024;
        if file_bytes.len() > limit_bytes {
            return Err(format!(
                "File exceeds the {} MB size limit! (Selected file: {:.2} MB)",
                max_size_mb,
                file_bytes.len() as f64 / 1_048_576.0
            ));
        }

        let session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_ref()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let cipher = Aes256Gcm::new_from_slice(&session.key)
            .map_err(|e| format!("Cipher error: {}", e))?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, file_bytes)
            .map_err(|e| format!("File encryption error: {}", e))?;

        let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);

        let file_id = Uuid::new_v4().to_string();
        let dest = self.vault_files_dir.join(format!("{}.enc", file_id));

        fs::write(&dest, payload)
            .map_err(|e| format!("Failed to write encrypted file to disk: {}", e))?;

        Ok((file_id, file_bytes.len()))
    }

    fn read_encrypted_file_internal(&self, key: &[u8; 32], file_id: &str) -> Result<Vec<u8>, String> {
        let path = self.vault_files_dir.join(format!("{}.enc", file_id));
        if !path.exists() {
            return Err("Encrypted file not found on disk.".into());
        }

        let bytes = fs::read(&path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if bytes.len() < NONCE_LEN + 16 {
            return Err("File is corrupted or has invalid format.".into());
        }

        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes.copy_from_slice(&bytes[0..NONCE_LEN]);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = &bytes[NONCE_LEN..];

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("Cipher error: {}", e))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Failed to decrypt file: {}", e))?;

        Ok(plaintext)
    }

    // Read and decrypt a file attachment
    pub fn read_encrypted_file(&self, file_id: &str) -> Result<Vec<u8>, String> {
        let session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_ref()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        self.read_encrypted_file_internal(&session.key, file_id)
    }

    pub fn get_vault_storage_dir(&self) -> PathBuf {
        self.vault_storage_dir.clone()
    }

    pub fn save_item(
        &self,
        id: Option<String>,
        title: String,
        secret: String,
        item_type: String,
        tab: String,
        group: String,
        pinned: bool,
        file_name: Option<String>,
        file_bytes: Option<Vec<u8>>,
        file_path: Option<String>,
        copy_to_vault: Option<bool>,
        max_size_mb: u32,
        notes: Option<String>,
    ) -> Result<VaultItem, String> {
        let mut final_file_name = file_name;
        let mut final_file_path = None;
        let mut final_file_size = None;
        let mut file_id_opt = None;

        if item_type == "file" {
            if let Some(ref src_path_str) = file_path {
                let src_path = PathBuf::from(src_path_str);
                if src_path.exists() {
                    let meta = fs::metadata(&src_path).ok();
                    let size = meta.as_ref().map(|m| m.len() as usize).unwrap_or(0);
                    final_file_size = Some(size);
                    if final_file_name.is_none() {
                        final_file_name = src_path.file_name().map(|n| n.to_string_lossy().to_string());
                    }
                    let resolved_name = final_file_name.clone().unwrap_or_else(|| "file".into());

                    if copy_to_vault == Some(true) {
                        // Copy file to vault_storage_dir
                        let dest_path = self.vault_storage_dir.join(&resolved_name);
                        let _ = fs::copy(&src_path, &dest_path)
                            .map_err(|e| format!("Failed to copy file to vault folder: {}", e))?;
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o600));
                        }
                        final_file_path = Some(dest_path.to_string_lossy().to_string());
                    } else {
                        // Shortcut mode: use original path
                        final_file_path = Some(src_path.to_string_lossy().to_string());
                    }
                } else {
                    return Err("Specified file path not found on disk.".into());
                }
            } else if let Some(bytes) = file_bytes {
                let limit_bytes = (max_size_mb as usize) * 1024 * 1024;
                if bytes.len() > limit_bytes {
                    return Err(format!(
                        "File exceeds the {} MB size limit! (Selected file: {:.2} MB)",
                        max_size_mb,
                        bytes.len() as f64 / 1_048_576.0
                    ));
                }
                let resolved_name = final_file_name.clone().unwrap_or_else(|| format!("file_{}.dat", Uuid::new_v4()));
                let dest_path = self.vault_storage_dir.join(&resolved_name);
                fs::write(&dest_path, &bytes).map_err(|e| format!("Failed to save file: {}", e))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o600));
                }
                final_file_path = Some(dest_path.to_string_lossy().to_string());
                final_file_size = Some(bytes.len());

                // Encrypted backup
                if let Ok((fid, _)) = self.save_encrypted_file(&bytes, max_size_mb) {
                    file_id_opt = Some(fid);
                }
            }
        }

        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let now = Utc::now().timestamp_millis();

        let item = if let Some(existing_id) = id {
            // Update
            if let Some(existing) = session.data.items.iter_mut().find(|i| i.id == existing_id) {
                existing.title = title;
                existing.secret = secret;
                existing.item_type = item_type;
                existing.tab = tab;
                existing.group = group;
                existing.pinned = pinned;
                existing.notes = notes;
                existing.updated_at = now;

                if final_file_path.is_some() || file_id_opt.is_some() {
                    existing.file_id = file_id_opt;
                    existing.file_path = final_file_path;
                    existing.file_name = final_file_name;
                    existing.file_size_bytes = final_file_size;
                }

                existing.clone()
            } else {
                return Err("Item to update not found.".into());
            }
        } else {
            // Create
            let new_item = VaultItem {
                id: Uuid::new_v4().to_string(),
                title,
                secret,
                item_type,
                tab,
                group,
                pinned,
                file_name: final_file_name,
                file_size_bytes: final_file_size,
                file_id: file_id_opt,
                file_path: final_file_path,
                notes,
                created_at: now,
                updated_at: now,
            };
            session.data.items.insert(0, new_item.clone());
            new_item
        };

        // Re-encrypt and persist
        self.save_session_data(session)?;

        Ok(item)
    }

    pub fn delete_item(&self, id: &str) -> Result<(), String> {
        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_mut()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let (file_to_delete, path_to_delete) = session
            .data
            .items
            .iter()
            .find(|i| i.id == id)
            .map(|i| (i.file_id.clone(), i.file_path.clone()))
            .unwrap_or((None, None));

        let initial_len = session.data.items.len();
        session.data.items.retain(|i| i.id != id);

        if session.data.items.len() == initial_len {
            return Err("Item to delete not found.".into());
        }

        if let Some(fid) = file_to_delete {
            let _ = fs::remove_file(self.vault_files_dir.join(format!("{}.enc", fid)));
        }

        // Only delete file if it is located inside self.vault_storage_dir (don't delete user original shortcuts!)
        if let Some(p_str) = path_to_delete {
            let p = PathBuf::from(&p_str);
            if p.starts_with(&self.vault_storage_dir) && p.exists() {
                let _ = fs::remove_file(p);
            }
        }

        self.save_session_data(session)?;

        Ok(())
    }

    pub fn get_secret(&self, id: &str) -> Result<String, String> {
        let session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_ref()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        session
            .data
            .items
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.secret.clone())
            .ok_or_else(|| "Item not found.".to_string())
    }

    pub fn get_file_info_and_path(&self, id: &str) -> Result<(String, PathBuf, Vec<u8>), String> {
        let session_guard = self.session.lock().unwrap();
        let session = session_guard
            .as_ref()
            .ok_or_else(|| "Vault is locked. Please unlock first.".to_string())?;

        let item = session
            .data
            .items
            .iter()
            .find(|i| i.id == id)
            .ok_or_else(|| "Item not found.".to_string())?;

        let file_name = item
            .file_name
            .clone()
            .unwrap_or_else(|| format!("file_{}", id));

        // 1. Check if item has file_path that exists
        if let Some(ref path_str) = item.file_path {
            let p = PathBuf::from(path_str);
            if p.exists() {
                let bytes = fs::read(&p).unwrap_or_default();
                return Ok((file_name, p, bytes));
            }
        }

        // 2. Check if file exists in vault_storage_dir
        let storage_path = self.vault_storage_dir.join(&file_name);
        if storage_path.exists() {
            let bytes = fs::read(&storage_path).unwrap_or_default();
            return Ok((file_name, storage_path, bytes));
        }

        // 3. Fallback: decrypt legacy file_id into vault_storage_dir
        if let Some(ref file_id) = item.file_id {
            let bytes = self.read_encrypted_file_internal(&session.key, file_id)?;
            let _ = fs::create_dir_all(&self.vault_storage_dir);
            let _ = fs::write(&storage_path, &bytes);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&storage_path, fs::Permissions::from_mode(0o600));
            }
            return Ok((file_name, storage_path, bytes));
        }

        Err("File for this item not found or has been deleted from disk.".to_string())
    }

    pub fn create_backup_bundle(&self) -> Result<String, String> {
        use base64::Engine;
        let b64 = &base64::engine::general_purpose::STANDARD;

        if !self.vault_file.exists() {
            return Err("Vault file not found for backup.".into());
        }

        let vault_enc_bytes = fs::read(&self.vault_file)
            .map_err(|e| format!("Failed to read vault file: {}", e))?;

        let mut files = Vec::new();
        if self.vault_files_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.vault_files_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let (Some(fname), Ok(data)) = (path.file_name().and_then(|n| n.to_str()), fs::read(&path)) {
                            files.push(BackupFileEntry {
                                file_name: fname.to_string(),
                                base64_data: b64.encode(&data),
                            });
                        }
                    }
                }
            }
        }

        let mut storage_files = Vec::new();
        if self.vault_storage_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.vault_storage_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let (Some(fname), Ok(data)) = (path.file_name().and_then(|n| n.to_str()), fs::read(&path)) {
                            storage_files.push(BackupFileEntry {
                                file_name: fname.to_string(),
                                base64_data: b64.encode(&data),
                            });
                        }
                    }
                }
            }
        }

        let bundle = VaultBackupData {
            version: 1,
            created_at: Utc::now().to_rfc3339(),
            vault_enc_base64: b64.encode(&vault_enc_bytes),
            files,
            storage_files,
        };

        serde_json::to_string_pretty(&bundle)
            .map_err(|e| format!("Failed to serialize backup data: {}", e))
    }

    pub fn restore_backup_bundle(&self, backup_json: &str) -> Result<(), String> {
        use base64::Engine;
        let b64 = &base64::engine::general_purpose::STANDARD;

        let bundle: VaultBackupData = serde_json::from_str(backup_json)
            .map_err(|e| format!("Invalid backup file format: {}", e))?;

        let new_vault_bytes = b64.decode(&bundle.vault_enc_base64)
            .map_err(|e| format!("Failed to decode backup vault data: {}", e))?;

        if new_vault_bytes.len() < MAGIC_LEN + SALT_LEN + NONCE_LEN + 16 {
            return Err("Backup vault file is corrupted or incomplete.".into());
        }

        // 1. Create a backup of current vault.enc as vault.enc.pre_restore_bak if exists
        if self.vault_file.exists() {
            let bak_path = self.vault_file.with_extension("pre_restore_bak");
            let _ = fs::copy(&self.vault_file, &bak_path);
        }

        // 2. Write new vault.enc
        fs::write(&self.vault_file, &new_vault_bytes)
            .map_err(|e| format!("Failed to write vault file: {}", e))?;

        // 3. Restore vault_files
        let _ = fs::create_dir_all(&self.vault_files_dir);
        for entry in bundle.files {
            if let Ok(data) = b64.decode(&entry.base64_data) {
                let p = self.vault_files_dir.join(&entry.file_name);
                let _ = fs::write(&p, &data);
            }
        }

        // 4. Restore vault_storage
        let _ = fs::create_dir_all(&self.vault_storage_dir);
        for entry in bundle.storage_files {
            if let Ok(data) = b64.decode(&entry.base64_data) {
                let p = self.vault_storage_dir.join(&entry.file_name);
                let _ = fs::write(&p, &data);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o600));
                }
            }
        }

        // 5. Lock current session to require unlocking with the restored vault's credentials
        self.lock();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_flow_and_files() {
        let temp_dir = std::env::temp_dir().join(format!("vault_test_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let vault_file = temp_dir.join("test_vault.enc");
        let files_dir = temp_dir.join("vault_files");

        let manager = VaultManager::new_with_path(vault_file.clone(), files_dir.clone());
        assert!(!manager.is_initialized());
        assert!(!manager.is_unlocked());

        // 1. Initialize vault with PIN 1234
        manager.init_vault(Some("1234".to_string())).expect("Init vault failed");
        assert!(manager.is_initialized());
        assert!(manager.is_unlocked());
        assert!(manager.get_status().has_pin);
        assert_eq!(manager.get_items().unwrap().len(), 0);

        // 2. Add an item with file
        let fake_pdf = b"%PDF-1.4 Fake encrypted document content".to_vec();
        let saved = manager
            .save_item(
                None,
                "ID Document".into(),
                "TC: 12345678901".into(),
                "file".into(),
                "Personal Info".into(),
                "Documents".into(),
                true,
                Some("kimlik.pdf".into()),
                Some(fake_pdf.clone()),
                None,
                None,
                20,
                Some("Official ID scan".into()),
            )
            .expect("Save item failed");

        assert_eq!(saved.title, "ID Document");
        assert!(saved.pinned);
        assert!(saved.file_id.is_some());

        // 3. Verify file size limit rejection (> 1 MB limit)
        let large_bytes = vec![0u8; 2 * 1024 * 1024]; // 2 MB
        let rejected = manager.save_item(
            None,
            "Large File".into(),
            "".into(),
            "file".into(),
            "Personal Info".into(),
            "Documents".into(),
            false,
            Some("large.zip".into()),
            Some(large_bytes),
            None,
            None,
            1, // Max 1 MB
            None,
        );
        assert!(rejected.is_err());

        // 4. Read back decrypted file
        let decrypted_bytes = manager
            .read_encrypted_file(saved.file_id.as_ref().unwrap())
            .expect("Read decrypted file failed");
        assert_eq!(decrypted_bytes, fake_pdf);

        // 4b. Test get_file_info_and_path for legacy file_id item (MUST NOT DEADLOCK!)
        let (fname, resolved_path, resolved_bytes) = manager
            .get_file_info_and_path(&saved.id)
            .expect("get_file_info_and_path for legacy file_id failed");
        assert_eq!(fname, "kimlik.pdf");
        assert_eq!(resolved_bytes, fake_pdf);
        assert!(resolved_path.exists());

        // 5. Test pin toggle
        let is_pinned = manager.toggle_item_pin(&saved.id).expect("Toggle pin failed");
        assert!(!is_pinned); // Was true, now false

        // 6. Test tab management
        let tabs = manager.add_tab("Finance".into()).expect("Add tab failed");
        assert!(tabs.contains(&"Finance".to_string()));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_auto_and_pin_vault_flow() {
        let temp_dir = std::env::temp_dir().join(format!("vault_auto_test_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let vault_file = temp_dir.join("auto_vault.enc");
        let files_dir = temp_dir.join("vault_files");

        let manager = VaultManager::new_with_path(vault_file.clone(), files_dir.clone());
        assert!(!manager.is_initialized());

        // 1. Initialize in Auto mode (no PIN)
        manager.init_vault(None).expect("Init auto vault failed");
        assert!(manager.is_initialized());
        assert!(manager.is_unlocked());
        assert!(!manager.get_status().has_pin);

        // 2. Add an item and an attached file
        let doc_content = b"Secret Auto Encrypted Text Document".to_vec();
        let saved = manager.save_item(
            None,
            "Auto Note".into(),
            "secret_text_123".into(),
            "file".into(),
            "Personal Info".into(),
            "General".into(),
            false,
            Some("gizli.txt".into()),
            Some(doc_content.clone()),
            None,
            None,
            10,
            None,
        ).expect("Save item in auto mode failed");

        // 3. Lock and auto unlock
        manager.lock();
        assert!(!manager.is_unlocked());
        assert!(!manager.get_status().has_pin);

        let items = manager.auto_unlock().expect("Auto unlock failed");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Auto Note");

        let decrypted_file = manager.read_encrypted_file(saved.file_id.as_ref().unwrap()).unwrap();
        assert_eq!(decrypted_file, doc_content);

        // 4. Switch from Auto mode to PIN mode
        manager.set_pin("9876").expect("Set PIN failed");
        assert!(manager.get_status().has_pin);

        // 5. Lock and test PIN unlock
        manager.lock();
        // auto_unlock should fail because PIN is now required
        assert!(manager.auto_unlock().is_err());

        // unlock with wrong PIN should fail
        assert!(manager.unlock("0000").is_err());

        // unlock with correct PIN succeeds
        let items_after_pin = manager.unlock("9876").expect("Unlock with PIN failed");
        assert_eq!(items_after_pin.len(), 1);
        assert_eq!(items_after_pin[0].title, "Auto Note");

        // Decrypted file still works after rekey!
        let rekeyed_file = manager.read_encrypted_file(saved.file_id.as_ref().unwrap()).unwrap();
        assert_eq!(rekeyed_file, doc_content);

        // 6. Switch back from PIN mode to Auto mode (remove PIN)
        manager.remove_pin().expect("Remove PIN failed");
        assert!(!manager.get_status().has_pin);

        manager.lock();
        let items_auto_again = manager.auto_unlock().expect("Auto unlock after remove_pin failed");
        assert_eq!(items_auto_again.len(), 1);

        let file_auto_again = manager.read_encrypted_file(saved.file_id.as_ref().unwrap()).unwrap();
        assert_eq!(file_auto_again, doc_content);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_shortcut_and_storage_modes() {
        let temp_dir = std::env::temp_dir().join(format!("vault_mode_test_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let vault_file = temp_dir.join("mode_vault.enc");
        let files_dir = temp_dir.join("vault_files");

        let manager = VaultManager::new_with_path(vault_file.clone(), files_dir.clone());
        manager.init_vault(None).expect("Init vault failed");

        // Create an external test file
        let external_file = temp_dir.join("external_doc.pdf");
        fs::write(&external_file, b"External Important Document Content").expect("Write external file failed");

        // 1. Test Shortcut Mode (copy_to_vault = false)
        let shortcut_item = manager.save_item(
            None,
            "Shortcut Document".into(),
            "".into(),
            "file".into(),
            "Personal Info".into(),
            "General".into(),
            false,
            Some("external_doc.pdf".into()),
            None,
            Some(external_file.to_string_lossy().to_string()),
            Some(false),
            20,
            None,
        ).expect("Save shortcut item failed");

        assert_eq!(shortcut_item.file_path, Some(external_file.to_string_lossy().to_string()));
        let (fname, resolved_path, bytes) = manager.get_file_info_and_path(&shortcut_item.id).unwrap();
        assert_eq!(fname, "external_doc.pdf");
        assert_eq!(resolved_path, external_file);
        assert_eq!(bytes, b"External Important Document Content");

        // 2. Test Copy to Vault Storage Mode (copy_to_vault = true)
        let copied_item = manager.save_item(
            None,
            "Copied Document".into(),
            "".into(),
            "file".into(),
            "Personal Info".into(),
            "General".into(),
            false,
            Some("vault_copied.pdf".into()),
            None,
            Some(external_file.to_string_lossy().to_string()),
            Some(true),
            20,
            None,
        ).expect("Save copy item failed");

        let expected_storage_path = manager.get_vault_storage_dir().join("vault_copied.pdf");
        assert_eq!(copied_item.file_path, Some(expected_storage_path.to_string_lossy().to_string()));
        assert!(expected_storage_path.exists());
        let (fname2, resolved_path2, bytes2) = manager.get_file_info_and_path(&copied_item.id).unwrap();
        assert_eq!(fname2, "vault_copied.pdf");
        assert_eq!(resolved_path2, expected_storage_path);
        assert_eq!(bytes2, b"External Important Document Content");

        // 3. Delete shortcut item -> External file must NOT be deleted!
        manager.delete_item(&shortcut_item.id).expect("Delete shortcut failed");
        assert!(external_file.exists(), "External file should NOT be deleted when shortcut item is deleted!");

        // 4. Delete copied item -> Storage copy should be removed!
        manager.delete_item(&copied_item.id).expect("Delete copied item failed");
        assert!(!expected_storage_path.exists(), "Storage file SHOULD be deleted when item is deleted!");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_and_restore_bundle() {
        let temp_dir = std::env::temp_dir().join(format!("vault_bak_test_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let vault_file = temp_dir.join("vault.enc");
        let files_dir = temp_dir.join("vault_files");

        let manager = VaultManager::new_with_path(vault_file.clone(), files_dir.clone());
        manager.init_vault(Some("9999".into())).expect("Init failed");

        // Add a secret
        manager.save_item(
            None,
            "Secret Note".into(),
            "Top Secret Token".into(),
            "password".into(),
            "Personal Info".into(),
            "Personal Data".into(),
            true,
            None,
            None,
            None,
            None,
            20,
            None,
        ).expect("Save item failed");

        // 1. Create backup
        let backup_json = manager.create_backup_bundle().expect("Backup creation failed");
        assert!(backup_json.contains("vault_enc_base64"));

        // 2. Clear out the vault directory
        let temp_dir2 = std::env::temp_dir().join(format!("vault_bak_target_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir2);
        let target_vault_file = temp_dir2.join("vault.enc");
        let target_files_dir = temp_dir2.join("vault_files");

        let target_manager = VaultManager::new_with_path(target_vault_file, target_files_dir);
        target_manager.restore_backup_bundle(&backup_json).expect("Restore failed");

        // 3. Unlock restored vault with original PIN 9999
        let restored_items = target_manager.unlock("9999").expect("Unlock restored vault failed");
        assert_eq!(restored_items.len(), 1);
        assert_eq!(restored_items[0].title, "Secret Note");

        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::remove_dir_all(&temp_dir2);
    }
}
