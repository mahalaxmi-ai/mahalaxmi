//! Tests for Credential Security — CredentialStore trait and backends.

use mahalaxmi_providers::credential_store::{
    credential_key, decrypt_bytes, derive_encryption_key, encrypt_bytes, probe_keyring,
    resolve_provider_credentials, ChainedCredentialStore, CredentialKeyIndex, CredentialStore,
    EncryptedFileCredentialStore, EnvCredentialStore, KeyringCredentialStore,
    MemoryCredentialStore,
};
use mahalaxmi_providers::credentials::{AuthMethod, CredentialSpec};
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;

// ===========================================================================
// MemoryCredentialStore
// ===========================================================================

#[test]
fn memory_store_get_nonexistent_returns_none() {
    let store = MemoryCredentialStore::new();
    assert!(store.get("nonexistent").unwrap().is_none());
}

#[test]
fn memory_store_set_and_get() {
    let store = MemoryCredentialStore::new();
    let secret = SecretString::from("sk-test-123".to_string());
    store.set("claude-code/ANTHROPIC_API_KEY", &secret).unwrap();

    let retrieved = store.get("claude-code/ANTHROPIC_API_KEY").unwrap().unwrap();
    assert_eq!(retrieved.expose_secret(), "sk-test-123");
}

#[test]
fn memory_store_overwrite() {
    let store = MemoryCredentialStore::new();
    let v1 = SecretString::from("v1".to_string());
    let v2 = SecretString::from("v2".to_string());

    store.set("key", &v1).unwrap();
    store.set("key", &v2).unwrap();

    let retrieved = store.get("key").unwrap().unwrap();
    assert_eq!(retrieved.expose_secret(), "v2");
}

#[test]
fn memory_store_delete() {
    let store = MemoryCredentialStore::new();
    let secret = SecretString::from("value".to_string());
    store.set("key", &secret).unwrap();
    store.delete("key").unwrap();
    assert!(store.get("key").unwrap().is_none());
}

#[test]
fn memory_store_delete_nonexistent_ok() {
    let store = MemoryCredentialStore::new();
    store.delete("nonexistent").unwrap();
}

#[test]
fn memory_store_exists() {
    let store = MemoryCredentialStore::new();
    assert!(!store.exists("key").unwrap());
    let secret = SecretString::from("val".to_string());
    store.set("key", &secret).unwrap();
    assert!(store.exists("key").unwrap());
}

#[test]
fn memory_store_list_keys() {
    let store = MemoryCredentialStore::new();
    assert!(store.list_keys().unwrap().is_empty());

    let s1 = SecretString::from("a".to_string());
    let s2 = SecretString::from("b".to_string());
    store.set("key-1", &s1).unwrap();
    store.set("key-2", &s2).unwrap();

    let mut keys = store.list_keys().unwrap();
    keys.sort();
    assert_eq!(keys, vec!["key-1", "key-2"]);
}

#[test]
fn memory_store_backend_name() {
    let store = MemoryCredentialStore::new();
    assert_eq!(store.backend_name(), "memory");
}

// ===========================================================================
// EnvCredentialStore
// ===========================================================================

#[test]
fn env_store_get_nonexistent_returns_none() {
    let store = EnvCredentialStore::new();
    assert!(store
        .get("MAHALAXMI_TEST_NONEXISTENT_VAR_12345")
        .unwrap()
        .is_none());
}

#[test]
fn env_store_get_with_provider_prefix() {
    // EnvCredentialStore uses the part after the last "/" as the env var name.
    // Since we can't guarantee any env var exists, we test the "not found" path.
    let store = EnvCredentialStore::new();
    assert!(store
        .get("claude-code/MAHALAXMI_MISSING_VAR_99999")
        .unwrap()
        .is_none());
}

#[test]
fn env_store_set_is_noop() {
    let store = EnvCredentialStore::new();
    let secret = SecretString::from("test".to_string());
    store.set("key", &secret).unwrap(); // Should not error
}

#[test]
fn env_store_delete_is_noop() {
    let store = EnvCredentialStore::new();
    store.delete("key").unwrap(); // Should not error
}

#[test]
fn env_store_list_keys_empty() {
    let store = EnvCredentialStore::new();
    assert!(store.list_keys().unwrap().is_empty());
}

#[test]
fn env_store_backend_name() {
    let store = EnvCredentialStore::new();
    assert_eq!(store.backend_name(), "env");
}

// ===========================================================================
// ChainedCredentialStore
// ===========================================================================

#[test]
fn chained_store_get_first_hit_wins() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let v1 = SecretString::from("from-first".to_string());
    let v2 = SecretString::from("from-second".to_string());
    mem1.set("key", &v1).unwrap();
    mem2.set("key", &v2).unwrap();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    let result = chain.get("key").unwrap().unwrap();
    assert_eq!(result.expose_secret(), "from-first");
}

#[test]
fn chained_store_get_falls_through() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let v2 = SecretString::from("from-second".to_string());
    mem2.set("key", &v2).unwrap();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    let result = chain.get("key").unwrap().unwrap();
    assert_eq!(result.expose_secret(), "from-second");
}

#[test]
fn chained_store_get_none_if_all_miss() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    assert!(chain.get("key").unwrap().is_none());
}

#[test]
fn chained_store_set_writes_to_first() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    let secret = SecretString::from("value".to_string());
    chain.set("key", &secret).unwrap();

    // First store should have it
    let result = chain.get("key").unwrap().unwrap();
    assert_eq!(result.expose_secret(), "value");
}

#[test]
fn chained_store_delete_removes_from_all() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let v1 = SecretString::from("v1".to_string());
    let v2 = SecretString::from("v2".to_string());
    mem1.set("key", &v1).unwrap();
    mem2.set("key", &v2).unwrap();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    chain.delete("key").unwrap();
    assert!(chain.get("key").unwrap().is_none());
}

#[test]
fn chained_store_backend_count() {
    let chain = ChainedCredentialStore::new(vec![
        Box::new(MemoryCredentialStore::new()),
        Box::new(EnvCredentialStore::new()),
    ]);
    assert_eq!(chain.backend_count(), 2);
}

#[test]
fn chained_store_backend_names() {
    let chain = ChainedCredentialStore::new(vec![
        Box::new(MemoryCredentialStore::new()),
        Box::new(EnvCredentialStore::new()),
    ]);
    assert_eq!(chain.backend_names(), vec!["memory", "env"]);
}

#[test]
fn chained_store_backend_name() {
    let chain = ChainedCredentialStore::new(vec![]);
    assert_eq!(chain.backend_name(), "chained");
}

#[test]
fn chained_store_list_keys_deduplicates() {
    let mem1 = MemoryCredentialStore::new();
    let mem2 = MemoryCredentialStore::new();

    let secret = SecretString::from("val".to_string());
    mem1.set("key-a", &secret).unwrap();
    mem1.set("key-b", &secret).unwrap();
    mem2.set("key-b", &secret).unwrap();
    mem2.set("key-c", &secret).unwrap();

    let chain = ChainedCredentialStore::new(vec![Box::new(mem1), Box::new(mem2)]);
    let mut keys = chain.list_keys().unwrap();
    keys.sort();
    assert_eq!(keys, vec!["key-a", "key-b", "key-c"]);
}

// ===========================================================================
// credential_key helper
// ===========================================================================

#[test]
fn credential_key_formats_correctly() {
    assert_eq!(
        credential_key("claude-code", "ANTHROPIC_API_KEY"),
        "claude-code/ANTHROPIC_API_KEY"
    );
}

#[test]
fn credential_key_empty_parts() {
    assert_eq!(credential_key("", "KEY"), "/KEY");
    assert_eq!(credential_key("provider", ""), "provider/");
}

// ===========================================================================
// resolve_provider_credentials
// ===========================================================================

#[test]
fn resolve_all_required_present() {
    let store = MemoryCredentialStore::new();
    let secret = SecretString::from("sk-test".to_string());
    store.set("test-provider/API_KEY", &secret).unwrap();

    let specs = vec![CredentialSpec {
        method: AuthMethod::ApiKey,
        env_var_name: Some("API_KEY".into()),
        description_key: "desc".into(),
        required: true,
    }];

    let result = resolve_provider_credentials(&store, "test-provider", &specs).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result["API_KEY"].expose_secret(), "sk-test");
}

#[test]
fn resolve_missing_required_errors() {
    let store = MemoryCredentialStore::new();

    let specs = vec![CredentialSpec {
        method: AuthMethod::ApiKey,
        env_var_name: Some("MISSING_KEY".into()),
        description_key: "desc".into(),
        required: true,
    }];

    let result = resolve_provider_credentials(&store, "test-provider", &specs);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("MISSING_KEY"));
}

#[test]
fn resolve_missing_optional_succeeds() {
    let store = MemoryCredentialStore::new();

    let specs = vec![CredentialSpec {
        method: AuthMethod::ApiKey,
        env_var_name: Some("OPTIONAL_KEY".into()),
        description_key: "desc".into(),
        required: false,
    }];

    let result = resolve_provider_credentials(&store, "test-provider", &specs).unwrap();
    assert!(result.is_empty());
}

#[test]
fn resolve_skips_specs_without_env_var_name() {
    let store = MemoryCredentialStore::new();

    let specs = vec![CredentialSpec {
        method: AuthMethod::AwsIam,
        env_var_name: None,
        description_key: "desc".into(),
        required: true,
    }];

    let result = resolve_provider_credentials(&store, "test-provider", &specs).unwrap();
    assert!(result.is_empty());
}

#[test]
fn resolve_mixed_required_and_optional() {
    let store = MemoryCredentialStore::new();
    let secret = SecretString::from("key-val".to_string());
    store.set("p/REQUIRED_KEY", &secret).unwrap();

    let specs = vec![
        CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("REQUIRED_KEY".into()),
            description_key: "desc".into(),
            required: true,
        },
        CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("OPTIONAL_KEY".into()),
            description_key: "desc".into(),
            required: false,
        },
    ];

    let result = resolve_provider_credentials(&store, "p", &specs).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("REQUIRED_KEY"));
}

// ===========================================================================
// KeyringCredentialStore (unit — no real keyring access)
// ===========================================================================

#[test]
fn keyring_store_backend_name() {
    let store = KeyringCredentialStore::new();
    assert_eq!(store.backend_name(), "os-keyring");
}

#[test]
fn keyring_store_list_keys_initially_empty() {
    let store = KeyringCredentialStore::new();
    assert!(store.list_keys().unwrap().is_empty());
}

// ===========================================================================
// Encryption helpers
// ===========================================================================

#[test]
fn encrypt_decrypt_round_trip() {
    let key = derive_encryption_key("test-domain");
    let plaintext = b"hello, world!";
    let encrypted = encrypt_bytes(&key, plaintext).unwrap();
    assert_ne!(&encrypted, plaintext);
    assert!(encrypted.len() > plaintext.len());

    let decrypted = decrypt_bytes(&key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn decrypt_wrong_key_returns_none() {
    let key1 = derive_encryption_key("domain-1");
    let key2 = derive_encryption_key("domain-2");
    let encrypted = encrypt_bytes(&key1, b"secret").unwrap();
    assert!(decrypt_bytes(&key2, &encrypted).is_none());
}

#[test]
fn decrypt_short_data_returns_none() {
    let key = derive_encryption_key("test");
    assert!(decrypt_bytes(&key, &[]).is_none());
    assert!(decrypt_bytes(&key, &[0u8; 12]).is_none()); // nonce only, no ciphertext
}

#[test]
fn decrypt_corrupt_data_returns_none() {
    let key = derive_encryption_key("test");
    let mut encrypted = encrypt_bytes(&key, b"data").unwrap();
    // Flip a byte in the ciphertext
    let last = encrypted.len() - 1;
    encrypted[last] ^= 0xFF;
    assert!(decrypt_bytes(&key, &encrypted).is_none());
}

// ===========================================================================
// CredentialKeyIndex
// ===========================================================================

/// Helper to create a temporary directory for tests.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mahalaxmi-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn credential_key_index_round_trip() {
    let dir = temp_dir("idx-rt");
    let index = CredentialKeyIndex::new(&dir);
    let keys = vec![
        "claude-code/ANTHROPIC_API_KEY".to_string(),
        "grok/XAI_API_KEY".to_string(),
    ];
    index.save(&keys).unwrap();

    let loaded = index.load();
    assert_eq!(loaded, keys);

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn credential_key_index_missing_file_returns_empty() {
    let dir = temp_dir("idx-missing");
    let index = CredentialKeyIndex::new(&dir);
    assert!(index.load().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn credential_key_index_corrupt_file_returns_empty() {
    let dir = temp_dir("idx-corrupt");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("credential_index.bin"), b"not-encrypted-data").unwrap();
    let index = CredentialKeyIndex::new(&dir);
    assert!(index.load().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// EncryptedFileCredentialStore
// ===========================================================================

#[test]
fn encrypted_file_store_set_get() {
    let dir = temp_dir("efs-sg");
    let store = EncryptedFileCredentialStore::new(&dir);
    let secret = SecretString::from("sk-xai-12345".to_string());
    store.set("grok/XAI_API_KEY", &secret).unwrap();

    let retrieved = store.get("grok/XAI_API_KEY").unwrap().unwrap();
    assert_eq!(retrieved.expose_secret(), "sk-xai-12345");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn encrypted_file_store_list_keys() {
    let dir = temp_dir("efs-lk");
    let store = EncryptedFileCredentialStore::new(&dir);
    let s1 = SecretString::from("a".to_string());
    let s2 = SecretString::from("b".to_string());
    store.set("key-1", &s1).unwrap();
    store.set("key-2", &s2).unwrap();

    let mut keys = store.list_keys().unwrap();
    keys.sort();
    assert_eq!(keys, vec!["key-1", "key-2"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn encrypted_file_store_delete_persists() {
    let dir = temp_dir("efs-del");
    let store = EncryptedFileCredentialStore::new(&dir);
    let secret = SecretString::from("val".to_string());
    store.set("key", &secret).unwrap();
    store.delete("key").unwrap();

    // Re-open to verify deletion persisted to disk
    let store2 = EncryptedFileCredentialStore::new(&dir);
    assert!(store2.get("key").unwrap().is_none());
    assert!(store2.list_keys().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn encrypted_file_store_survives_restart() {
    let dir = temp_dir("efs-restart");
    {
        let store = EncryptedFileCredentialStore::new(&dir);
        let s1 = SecretString::from("secret-1".to_string());
        let s2 = SecretString::from("secret-2".to_string());
        store.set("provider-a/KEY_A", &s1).unwrap();
        store.set("provider-b/KEY_B", &s2).unwrap();
        // store is dropped here
    }

    // Simulate restart — create a new store instance
    let store = EncryptedFileCredentialStore::new(&dir);
    let v1 = store.get("provider-a/KEY_A").unwrap().unwrap();
    assert_eq!(v1.expose_secret(), "secret-1");
    let v2 = store.get("provider-b/KEY_B").unwrap().unwrap();
    assert_eq!(v2.expose_secret(), "secret-2");

    let mut keys = store.list_keys().unwrap();
    keys.sort();
    assert_eq!(keys, vec!["provider-a/KEY_A", "provider-b/KEY_B"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn encrypted_file_store_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("efs-perms");
    let store = EncryptedFileCredentialStore::new(&dir);
    let secret = SecretString::from("val".to_string());
    store.set("key", &secret).unwrap();

    let cred_file = dir.join("credentials.bin");
    let metadata = std::fs::metadata(&cred_file).unwrap();
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "credentials.bin should be 0600, got {mode:o}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn encrypted_file_store_backend_name() {
    let dir = temp_dir("efs-bn");
    let store = EncryptedFileCredentialStore::new(&dir);
    assert_eq!(store.backend_name(), "encrypted-file");
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// KeyringCredentialStore — seed_known_keys
// ===========================================================================

#[test]
fn keyring_store_seed_known_keys() {
    let dir = temp_dir("kr-seed");
    let store = KeyringCredentialStore::with_persistence(&dir);
    assert!(store.list_keys().unwrap().is_empty());

    store.seed_known_keys(&[
        "claude-code/ANTHROPIC_API_KEY".to_string(),
        "grok/XAI_API_KEY".to_string(),
    ]);

    let mut keys = store.list_keys().unwrap();
    keys.sort();
    assert_eq!(
        keys,
        vec!["claude-code/ANTHROPIC_API_KEY", "grok/XAI_API_KEY"]
    );

    // Seed again with overlap — should not duplicate
    store.seed_known_keys(&[
        "grok/XAI_API_KEY".to_string(),
        "gemini/GOOGLE_API_KEY".to_string(),
    ]);

    let mut keys = store.list_keys().unwrap();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "claude-code/ANTHROPIC_API_KEY",
            "gemini/GOOGLE_API_KEY",
            "grok/XAI_API_KEY"
        ]
    );

    // Verify persistence — reload index
    let index = CredentialKeyIndex::new(&dir);
    let mut persisted = index.load();
    persisted.sort();
    assert_eq!(persisted, keys);

    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// probe_keyring
// ===========================================================================

#[test]
fn probe_keyring_returns_bool() {
    // We can't control keyring availability in CI, but probe_keyring should
    // not panic regardless.
    let result = probe_keyring();
    // Type check — it returns bool
    let _: bool = result;
}

// ===========================================================================
// ChainedCredentialStore — persistence integration
// ===========================================================================

#[test]
fn chained_default_with_persistence() {
    let dir = temp_dir("chain-persist");
    let chain = ChainedCredentialStore::default_chain_with_persistence(&dir);
    // Should have 2 backends regardless of keyring availability
    assert_eq!(chain.backend_count(), 2);

    let names = chain.backend_names();
    // First backend is either "os-keyring" or "encrypted-file"
    assert!(
        names[0] == "os-keyring" || names[0] == "encrypted-file",
        "unexpected primary backend: {}",
        names[0]
    );
    assert_eq!(names[1], "env");
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// seed_known_keys trait default
// ===========================================================================

#[test]
fn seed_known_keys_trait_default_is_noop() {
    let store = MemoryCredentialStore::new();
    // Should not panic or change state
    store.seed_known_keys(&["key-1".to_string(), "key-2".to_string()]);
    // Memory store doesn't implement seed_known_keys — list_keys should be empty
    assert!(store.list_keys().unwrap().is_empty());
}
