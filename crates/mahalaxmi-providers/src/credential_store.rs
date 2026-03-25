//! Credential storage backends for AI provider authentication.
//!
//! Provides a `CredentialStore` trait with multiple implementations:
//!
//! - `KeyringCredentialStore` — OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
//! - `EncryptedFileCredentialStore` — AES-256-GCM encrypted file (WSL2/headless Linux fallback)
//! - `EnvCredentialStore` — environment variable lookup
//! - `MemoryCredentialStore` — in-memory store (for testing)
//! - `ChainedCredentialStore` — tries multiple stores in priority order
//!
//! The chained store enables a fallback pattern: try keyring first, then env, then memory.
//! On WSL2/headless Linux where the keyring daemon is unavailable, the encrypted file
//! store is used instead.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::MahalaxmiResult;

use crate::credentials::CredentialSpec;

/// Service name used as the keyring "service" identifier.
const KEYRING_SERVICE: &str = "mahalaxmi-terminal-orchestration";

/// Key used to probe keyring availability at startup.
const KEYRING_PROBE_KEY: &str = "__mahalaxmi_keyring_probe__";

/// AES-256-GCM nonce size in bytes.
const NONCE_SIZE: usize = 12;

/// Helper to create a Config error without i18n (infrastructure-level code).
fn credential_error(msg: String) -> MahalaxmiError {
    MahalaxmiError::Config {
        message: msg,
        i18n_key: "error-credential-store".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Encryption helpers
// ---------------------------------------------------------------------------

/// Application secret for credential encryption.
///
/// Returns `MAHALAXMI_APP_SECRET` env var when set.  Otherwise derives a
/// machine-unique value from hostname + OS + username via SHA-256 — no
/// hardcoded fallback string is embedded in the binary.
fn credential_app_secret() -> String {
    std::env::var("MAHALAXMI_APP_SECRET").unwrap_or_else(|_| {
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "localhost".to_owned());
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_owned());
        let mut h = Sha256::new();
        h.update(b"mahalaxmi:app-secret:v1:");
        h.update(std::env::consts::OS.as_bytes());
        h.update(b"::");
        h.update(hostname.as_bytes());
        h.update(b"::");
        h.update(user.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    })
}

/// Current OS username for per-user key derivation.
fn os_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Derive a 256-bit encryption key via SHA-256.
///
/// `key = SHA-256(app_secret + "::" + domain + "::" + username)`
///
/// This ensures different domains (credential-index vs credentials) and different
/// OS users produce distinct keys.
pub fn derive_encryption_key(domain: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(credential_app_secret().as_bytes());
    hasher.update(b"::");
    hasher.update(domain.as_bytes());
    hasher.update(b"::");
    hasher.update(os_username().as_bytes());
    hasher.finalize().into()
}

/// Encrypt plaintext bytes with AES-256-GCM.
///
/// Returns `[12-byte random nonce][ciphertext+tag]`.
pub fn encrypt_bytes(key: &[u8; 32], plaintext: &[u8]) -> MahalaxmiResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| credential_error(format!("encryption failed: {e}")))?;

    let mut out = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt data produced by `encrypt_bytes`.
///
/// Returns `None` on any error (wrong key, corrupt data, too short).
pub fn decrypt_bytes(key: &[u8; 32], data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < NONCE_SIZE + 1 {
        return None;
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
}

// ---------------------------------------------------------------------------
// CredentialKeyIndex — persistent index of known key names
// ---------------------------------------------------------------------------

/// Encrypted persistent index of credential key names.
///
/// File format: `[12-byte nonce][AES-256-GCM(JSON array of strings)]`
///
/// The key is derived from the app secret + OS username, so a different user
/// on the same machine cannot read the index.
pub struct CredentialKeyIndex {
    path: PathBuf,
    encryption_key: [u8; 32],
}

impl CredentialKeyIndex {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join("credential_index.bin"),
            encryption_key: derive_encryption_key("credential-index"),
        }
    }

    /// Load key names from disk. Returns empty vec on any failure.
    pub fn load(&self) -> Vec<String> {
        let data = match std::fs::read(&self.path) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        let plaintext = match decrypt_bytes(&self.encryption_key, &data) {
            Some(p) => p,
            None => return Vec::new(),
        };
        serde_json::from_slice::<Vec<String>>(&plaintext).unwrap_or_default()
    }

    /// Save key names to disk, creating parent directories as needed.
    pub fn save(&self, keys: &[String]) -> MahalaxmiResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                credential_error(format!("failed to create credential index dir: {e}"))
            })?;
        }
        let json = serde_json::to_vec(keys)
            .map_err(|e| credential_error(format!("failed to serialize key index: {e}")))?;
        let encrypted = encrypt_bytes(&self.encryption_key, &json)?;

        // Atomic write: write to temp file, then rename
        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, &encrypted)
            .map_err(|e| credential_error(format!("failed to write credential index: {e}")))?;
        set_file_permissions_0600(&tmp_path);
        std::fs::rename(&tmp_path, &self.path)
            .map_err(|e| credential_error(format!("failed to rename credential index: {e}")))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CredentialStore trait
// ---------------------------------------------------------------------------

/// A store that can get, set, and delete credentials by key.
///
/// Keys follow the pattern `provider_id/credential_name` (e.g.,
/// `claude-code/ANTHROPIC_API_KEY`).
pub trait CredentialStore: Send + Sync {
    /// Retrieve a credential by key.
    ///
    /// Returns `Ok(None)` if the key doesn't exist in this store.
    /// Returns `Err` only for backend failures (keyring locked, permission denied).
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>>;

    /// Store a credential by key.
    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()>;

    /// Delete a credential by key.
    ///
    /// Returns `Ok(())` even if the key didn't exist.
    fn delete(&self, key: &str) -> MahalaxmiResult<()>;

    /// Check if a credential exists without retrieving its value.
    fn exists(&self, key: &str) -> MahalaxmiResult<bool> {
        Ok(self.get(key)?.is_some())
    }

    /// List all credential keys in this store.
    fn list_keys(&self) -> MahalaxmiResult<Vec<String>>;

    /// Human-readable name for this backend (e.g., "os-keyring", "env", "memory").
    fn backend_name(&self) -> &'static str;

    /// Seed previously-discovered key names into this store's index.
    ///
    /// Default is a no-op. `KeyringCredentialStore` overrides this to merge
    /// and persist the names so that `list_keys()` includes keys stored before
    /// the persistence feature was added.
    fn seed_known_keys(&self, _keys: &[String]) {}
}

// ---------------------------------------------------------------------------
// KeyringCredentialStore — OS keychain
// ---------------------------------------------------------------------------

/// Credential store backed by the OS keychain.
///
/// Uses the `keyring` crate which delegates to:
/// - Windows: Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service (GNOME Keyring / KDE Wallet)
pub struct KeyringCredentialStore {
    /// Tracked keys (keyring has no native "list" operation).
    known_keys: Mutex<Vec<String>>,
    /// Optional persistent index — populated via `with_persistence`.
    key_index: Option<CredentialKeyIndex>,
}

impl KeyringCredentialStore {
    pub fn new() -> Self {
        Self {
            known_keys: Mutex::new(Vec::new()),
            key_index: None,
        }
    }

    /// Create a keyring store with persistent key index at `data_dir/credential_index.bin`.
    ///
    /// Loads existing key names from disk on creation.
    pub fn with_persistence(data_dir: &Path) -> Self {
        let index = CredentialKeyIndex::new(data_dir);
        let known = index.load();
        tracing::debug!(count = known.len(), "Loaded credential key index from disk");
        Self {
            known_keys: Mutex::new(known),
            key_index: Some(index),
        }
    }

    fn entry(&self, key: &str) -> MahalaxmiResult<keyring::Entry> {
        keyring::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| credential_error(format!("keyring entry error for '{key}': {e}")))
    }

    /// Persist the current known_keys to disk if an index is configured.
    fn persist_index(&self, keys: &[String]) {
        if let Some(ref index) = self.key_index {
            if let Err(e) = index.save(keys) {
                tracing::warn!(error = %e, "Failed to persist credential key index");
            }
        }
    }
}

impl Default for KeyringCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        let entry = self.entry(key)?;
        match entry.get_password() {
            Ok(password) => Ok(Some(SecretString::from(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(credential_error(format!(
                "keyring get failed for '{key}': {e}"
            ))),
        }
    }

    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()> {
        let entry = self.entry(key)?;
        entry
            .set_password(value.expose_secret())
            .map_err(|e| credential_error(format!("keyring set failed for '{key}': {e}")))?;
        let mut keys = self
            .known_keys
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        if !keys.contains(&key.to_string()) {
            keys.push(key.to_string());
        }
        self.persist_index(&keys);
        Ok(())
    }

    fn delete(&self, key: &str) -> MahalaxmiResult<()> {
        let entry = self.entry(key)?;
        match entry.delete_credential() {
            Ok(()) => {}
            Err(keyring::Error::NoEntry) => {}
            Err(e) => {
                return Err(credential_error(format!(
                    "keyring delete failed for '{key}': {e}"
                )));
            }
        }
        let mut keys = self
            .known_keys
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        keys.retain(|k| k != key);
        self.persist_index(&keys);
        Ok(())
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        let keys = self
            .known_keys
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        Ok(keys.clone())
    }

    fn backend_name(&self) -> &'static str {
        "os-keyring"
    }

    fn seed_known_keys(&self, incoming: &[String]) {
        if let Ok(mut keys) = self.known_keys.lock() {
            let mut changed = false;
            for k in incoming {
                if !keys.contains(k) {
                    keys.push(k.clone());
                    changed = true;
                }
            }
            if changed {
                self.persist_index(&keys);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// EncryptedFileCredentialStore — AES-256-GCM encrypted file
// ---------------------------------------------------------------------------

/// Encrypted file-based credential store for environments without a keyring.
///
/// File format: `[12-byte nonce][AES-256-GCM(JSON object of key→value)]`
///
/// Suitable for WSL2, headless Linux, or any environment where the OS keyring
/// daemon (Secret Service) is unavailable. The file is stored at
/// `~/.mahalaxmi/credentials.bin` with `0600` permissions on Unix.
pub struct EncryptedFileCredentialStore {
    path: PathBuf,
    encryption_key: [u8; 32],
    cache: Mutex<HashMap<String, SecretString>>,
}

impl EncryptedFileCredentialStore {
    /// Create a new encrypted file store at `data_dir/credentials.bin`.
    ///
    /// Loads any existing data from disk into memory.
    pub fn new(data_dir: &Path) -> Self {
        let path = data_dir.join("credentials.bin");
        let encryption_key = derive_encryption_key("credentials");
        let cache = Self::load_from_disk(&path, &encryption_key);
        tracing::debug!(count = cache.len(), "Loaded encrypted credential file");
        Self {
            path,
            encryption_key,
            cache: Mutex::new(cache),
        }
    }

    /// Load credential map from an encrypted file. Returns empty map on any error.
    fn load_from_disk(path: &Path, key: &[u8; 32]) -> HashMap<String, SecretString> {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => return HashMap::new(),
        };
        let plaintext = match decrypt_bytes(key, &data) {
            Some(p) => p,
            None => return HashMap::new(),
        };
        let map: HashMap<String, String> = serde_json::from_slice(&plaintext).unwrap_or_default();
        map.into_iter()
            .map(|(k, v)| (k, SecretString::from(v)))
            .collect()
    }

    /// Flush the in-memory cache to the encrypted file.
    fn flush(&self, cache: &HashMap<String, SecretString>) -> MahalaxmiResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| credential_error(format!("failed to create credentials dir: {e}")))?;
        }
        // Serialize to a plain HashMap<String, String> for JSON
        let plain: HashMap<String, String> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.expose_secret().to_string()))
            .collect();
        let json = serde_json::to_vec(&plain)
            .map_err(|e| credential_error(format!("failed to serialize credentials: {e}")))?;
        let encrypted = encrypt_bytes(&self.encryption_key, &json)?;

        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, &encrypted)
            .map_err(|e| credential_error(format!("failed to write credentials file: {e}")))?;
        set_file_permissions_0600(&tmp_path);
        std::fs::rename(&tmp_path, &self.path)
            .map_err(|e| credential_error(format!("failed to rename credentials file: {e}")))?;
        Ok(())
    }
}

impl CredentialStore for EncryptedFileCredentialStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        Ok(cache.get(key).cloned())
    }

    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        cache.insert(key.to_string(), value.clone());
        self.flush(&cache)
    }

    fn delete(&self, key: &str) -> MahalaxmiResult<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        cache.remove(key);
        self.flush(&cache)
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        Ok(cache.keys().cloned().collect())
    }

    fn backend_name(&self) -> &'static str {
        "encrypted-file"
    }
}

// ---------------------------------------------------------------------------
// EnvCredentialStore — environment variables
// ---------------------------------------------------------------------------

/// Credential store that reads from environment variables.
///
/// Keys are interpreted as environment variable names (e.g., `ANTHROPIC_API_KEY`).
/// This store is read-only — `set` and `delete` are no-ops that return `Ok`.
pub struct EnvCredentialStore;

impl EnvCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnvCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for EnvCredentialStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        // For env store, we use the part after "/" as the env var name,
        // or the whole key if no "/" present.
        let env_name = key.rsplit('/').next().unwrap_or(key);
        match std::env::var(env_name) {
            Ok(val) if !val.is_empty() => Ok(Some(SecretString::from(val))),
            Ok(_) => Ok(None),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(credential_error(format!(
                "env var read error for '{env_name}': {e}"
            ))),
        }
    }

    fn set(&self, _key: &str, _value: &SecretString) -> MahalaxmiResult<()> {
        Ok(())
    }

    fn delete(&self, _key: &str) -> MahalaxmiResult<()> {
        Ok(())
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        Ok(Vec::new())
    }

    fn backend_name(&self) -> &'static str {
        "env"
    }
}

// ---------------------------------------------------------------------------
// MemoryCredentialStore — in-memory (testing)
// ---------------------------------------------------------------------------

/// In-memory credential store for testing and development.
///
/// Not persistent — credentials are lost when the process exits.
pub struct MemoryCredentialStore {
    store: Mutex<HashMap<String, SecretString>>,
}

impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        let store = self
            .store
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        Ok(store.get(key).cloned())
    }

    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        store.insert(key.to_string(), value.clone());
        Ok(())
    }

    fn delete(&self, key: &str) -> MahalaxmiResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        store.remove(key);
        Ok(())
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        let store = self
            .store
            .lock()
            .map_err(|e| credential_error(format!("mutex poisoned: {e}")))?;
        Ok(store.keys().cloned().collect())
    }

    fn backend_name(&self) -> &'static str {
        "memory"
    }
}

// ---------------------------------------------------------------------------
// CloudSecretManagerStore — Cloud-native secret management
// ---------------------------------------------------------------------------

/// Trait for cloud-specific secret manager backends.
pub trait CloudSecretBackend: Send + Sync {
    fn get_secret(&self, key: &str) -> MahalaxmiResult<Option<SecretString>>;
    fn set_secret(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()>;
    fn delete_secret(&self, key: &str) -> MahalaxmiResult<()>;
    fn list_secrets(&self) -> MahalaxmiResult<Vec<String>>;
    fn name(&self) -> &'static str;
}

/// Credential store that delegates to a cloud secret manager (AWS, Azure, GCP).
pub struct CloudSecretManagerStore {
    backend: Box<dyn CloudSecretBackend>,
}

impl CloudSecretManagerStore {
    pub fn new(backend: Box<dyn CloudSecretBackend>) -> Self {
        Self { backend }
    }

    /// Detects if running in a cloud environment and returns the appropriate backend.
    pub fn auto_detect() -> Option<Self> {
        // This would use metadata services or env vars to detect cloud provider.
        // For now, we return None unless explicitly configured,
        // but the architecture is ready for AWS/Azure/GCP integrations.
        None
    }
}

impl CredentialStore for CloudSecretManagerStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        self.backend.get_secret(key)
    }

    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()> {
        self.backend.set_secret(key, value)
    }

    fn delete(&self, key: &str) -> MahalaxmiResult<()> {
        self.backend.delete_secret(key)
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        self.backend.list_secrets()
    }

    fn backend_name(&self) -> &'static str {
        self.backend.name()
    }
}

// ---------------------------------------------------------------------------
// Keyring probe
// ---------------------------------------------------------------------------

/// Probe whether the OS keyring backend is functional.
///
/// Attempts a test write and delete on a sentinel key. Returns `true` if the
/// keyring is responsive, `false` if the Secret Service daemon is missing or
/// the backend is otherwise unusable.
pub fn probe_keyring() -> bool {
    let entry = match keyring::Entry::new(KEYRING_SERVICE, KEYRING_PROBE_KEY) {
        Ok(e) => e,
        Err(_) => return false,
    };
    if entry.set_password("probe").is_err() {
        return false;
    }
    let _ = entry.delete_credential();
    true
}

// ---------------------------------------------------------------------------
// ChainedCredentialStore — fallback chain
// ---------------------------------------------------------------------------

/// Credential store that tries multiple backends in priority order.
///
/// - `get`: returns the first hit from any backend
/// - `set`: writes to the **first** (highest-priority) backend
/// - `delete`: removes from **all** backends
///
/// Typical chain: `[KeyringCredentialStore, EnvCredentialStore]`
pub struct ChainedCredentialStore {
    stores: Vec<Box<dyn CredentialStore>>,
}

impl ChainedCredentialStore {
    pub fn new(stores: Vec<Box<dyn CredentialStore>>) -> Self {
        Self { stores }
    }

    /// Create the default chain: keyring (if available) → env vars.
    ///
    /// No persistence — for backward compatibility and tests.
    pub fn default_chain() -> Self {
        Self::new(vec![
            Box::new(KeyringCredentialStore::new()),
            Box::new(EnvCredentialStore::new()),
        ])
    }

    /// Create a persistent chain with keyring probe.
    ///
    /// - If in Cloud: `cloud-secret-manager → keyring → env`
    /// - If the OS keyring is functional: `keyring (with persistent index) → env`
    /// - If the OS keyring is unavailable: `encrypted-file → env`
    pub fn default_chain_with_persistence(data_dir: &Path) -> Self {
        let mut stores: Vec<Box<dyn CredentialStore>> = Vec::new();

        // 1. Try Cloud Secret Manager first if detected (production cloud environments)
        if let Some(cloud_store) = CloudSecretManagerStore::auto_detect() {
            tracing::info!(
                backend = cloud_store.backend_name(),
                "Cloud environment detected — adding cloud secret manager to chain"
            );
            stores.push(Box::new(cloud_store));
        }

        // 2. Add local primary store
        if probe_keyring() {
            tracing::info!(
                "OS keyring is functional — adding keyring with persistent index to chain"
            );
            stores.push(Box::new(KeyringCredentialStore::with_persistence(data_dir)));
        } else {
            tracing::debug!(
                "OS keyring unavailable — using encrypted file credential store (expected on WSL2/headless)"
            );
            stores.push(Box::new(EncryptedFileCredentialStore::new(data_dir)));
        }

        // 3. Always fallback to environment variables
        stores.push(Box::new(EnvCredentialStore::new()));

        Self::new(stores)
    }

    /// Number of backends in the chain.
    pub fn backend_count(&self) -> usize {
        self.stores.len()
    }

    /// Names of all backends in priority order.
    pub fn backend_names(&self) -> Vec<&'static str> {
        self.stores.iter().map(|s| s.backend_name()).collect()
    }
}

impl CredentialStore for ChainedCredentialStore {
    fn get(&self, key: &str) -> MahalaxmiResult<Option<SecretString>> {
        for store in &self.stores {
            match store.get(key) {
                Ok(Some(val)) => {
                    tracing::debug!(
                        backend = store.backend_name(),
                        key = key,
                        "credential found"
                    );
                    return Ok(Some(val));
                }
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!(
                        backend = store.backend_name(),
                        key = key,
                        error = %e,
                        "credential store error, trying next backend"
                    );
                    continue;
                }
            }
        }
        Ok(None)
    }

    fn set(&self, key: &str, value: &SecretString) -> MahalaxmiResult<()> {
        let mut last_error = None;
        for store in &self.stores {
            match store.set(key, value) {
                Ok(()) => {
                    tracing::debug!(
                        backend = store.backend_name(),
                        key = key,
                        "credential saved"
                    );
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(
                        backend = store.backend_name(),
                        key = key,
                        error = %e,
                        "credential store set failed, trying next backend"
                    );
                    last_error = Some(e);
                }
            }
        }
        Err(last_error
            .unwrap_or_else(|| credential_error("no credential stores configured".into())))
    }

    fn delete(&self, key: &str) -> MahalaxmiResult<()> {
        for store in &self.stores {
            let _ = store.delete(key);
        }
        Ok(())
    }

    fn list_keys(&self) -> MahalaxmiResult<Vec<String>> {
        let mut all_keys = Vec::new();
        for store in &self.stores {
            if let Ok(keys) = store.list_keys() {
                for key in keys {
                    if !all_keys.contains(&key) {
                        all_keys.push(key);
                    }
                }
            }
        }
        Ok(all_keys)
    }

    fn backend_name(&self) -> &'static str {
        "chained"
    }

    fn seed_known_keys(&self, keys: &[String]) {
        for store in &self.stores {
            store.seed_known_keys(keys);
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Set file permissions to 0600 (owner read/write only) on Unix.
/// No-op on non-Unix platforms.
#[cfg(unix)]
fn set_file_permissions_0600(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn set_file_permissions_0600(_path: &Path) {}

/// Format a credential key from provider ID and credential name.
///
/// Example: `credential_key("claude-code", "ANTHROPIC_API_KEY")` → `"claude-code/ANTHROPIC_API_KEY"`
pub fn credential_key(provider_id: &str, credential_name: &str) -> String {
    format!("{provider_id}/{credential_name}")
}

/// Resolve credentials for a provider from a credential store.
///
/// Returns a map of env_var_name → secret value for all required credentials
/// that could be found. Missing optional credentials are silently skipped.
/// Returns an error if any required credential is missing.
pub fn resolve_provider_credentials(
    store: &dyn CredentialStore,
    provider_id: &str,
    specs: &[CredentialSpec],
) -> MahalaxmiResult<HashMap<String, SecretString>> {
    let mut resolved = HashMap::new();
    let mut missing = Vec::new();

    for spec in specs {
        let env_name = match &spec.env_var_name {
            Some(name) => name.clone(),
            None => continue,
        };
        let key = credential_key(provider_id, &env_name);

        match store.get(&key)? {
            Some(secret) => {
                resolved.insert(env_name, secret);
            }
            None => {
                if spec.required {
                    missing.push(env_name);
                }
            }
        }
    }

    if !missing.is_empty() {
        return Err(credential_error(format!(
            "missing required credentials for provider '{provider_id}': {}",
            missing.join(", ")
        )));
    }

    Ok(resolved)
}
