// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Configuration loading
//
// Demonstrates how to load a Mahalaxmi configuration from the default path
// (~/.mahalaxmi/config.toml), access typed config values, and print a summary
// of the active settings. If the file does not exist, built-in defaults are used.

use mahalaxmi_core::config::loader::{default_config_path, load_config};
use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};

fn main() {
    // Create an i18n service — required for localized error messages.
    let i18n = I18nService::new(SupportedLocale::EnUs);

    let config_path = default_config_path();
    println!("Config path: {}", config_path.display());

    // load_config returns defaults when the file does not exist.
    let config = load_config(Some(&config_path), &i18n).unwrap_or_else(|err| {
        eprintln!("Warning: could not load config ({err}), using defaults");
        Default::default()
    });

    // Access typed fields from each sub-section.
    println!("--- General ---");
    println!("  log_level:           {}", config.general.log_level);
    println!("  locale:              {}", config.general.locale);

    println!("--- Orchestration ---");
    println!("  max_concurrent:      {}", config.orchestration.max_concurrent_workers);

    println!("--- Providers ---");
    println!("  default_provider:    {}", config.providers.default_provider);

    println!("--- Indexing ---");
    println!("  max_file_size_bytes: {}", config.indexing.max_file_size_bytes);
    println!("  excluded_dirs:       {:?}", config.indexing.excluded_dirs);

    println!("--- UI ---");
    println!("  theme:               {}", config.ui.theme);
    println!("  terminal_font_size:  {}", config.ui.terminal_font_size);
}
