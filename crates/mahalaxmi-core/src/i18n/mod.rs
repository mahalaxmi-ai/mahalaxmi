//! Internationalization (i18n) infrastructure for Mahalaxmi.
//!
//! Every user-facing string in the system goes through this module.
//! Translations are stored in Fluent `.ftl` files under `locales/`.

pub mod locale;
pub mod messages;
pub mod service;

/// BCP 47 locale enum listing every language the application supports.
pub use locale::SupportedLocale;
/// Thread-safe translation service backed by Fluent resource bundles.
pub use service::I18nService;
