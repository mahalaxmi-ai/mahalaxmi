//! Core domain types, traits, and shared infrastructure for Mahalaxmi.
//!
//! This crate provides the foundation that all other Mahalaxmi crates build upon,
//! including internationalization, configuration, error handling, and logging.

pub mod config;
pub mod domain;
pub mod developer;
pub mod error;
pub mod i18n;
pub mod logging;
pub mod security;
pub mod singleton;
pub mod types;
pub mod user_message;

pub use developer::{
    Developer, DeveloperId, DeveloperRegistry, DeveloperSession, DeveloperSessionStatus,
};
pub use security::encryption::{derive_key_from_passphrase, EncryptedString, EncryptionError};

/// Convenience result type for the Mahalaxmi project.
pub type MahalaxmiResult<T> = Result<T, error::MahalaxmiError>;
