//! Translation service backed by Fluent `.ftl` locale files.

use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

use super::locale::SupportedLocale;

/// Embedded locale file contents for en-US.
const EN_US_CORE: &str = include_str!("../../locales/en-US/core.ftl");
/// Embedded locale file contents for es-ES.
const ES_ES_CORE: &str = include_str!("../../locales/es-ES/core.ftl");
/// Embedded locale file contents for fr-FR.
const FR_FR_CORE: &str = include_str!("../../locales/fr-FR/core.ftl");
/// Embedded locale file contents for de-DE.
const DE_DE_CORE: &str = include_str!("../../locales/de-DE/core.ftl");
/// Embedded locale file contents for pt-BR.
const PT_BR_CORE: &str = include_str!("../../locales/pt-BR/core.ftl");
/// Embedded locale file contents for ja-JP.
const JA_JP_CORE: &str = include_str!("../../locales/ja-JP/core.ftl");
/// Embedded locale file contents for zh-CN.
const ZH_CN_CORE: &str = include_str!("../../locales/zh-CN/core.ftl");
/// Embedded locale file contents for ko-KR.
const KO_KR_CORE: &str = include_str!("../../locales/ko-KR/core.ftl");
/// Embedded locale file contents for hi-IN.
const HI_IN_CORE: &str = include_str!("../../locales/hi-IN/core.ftl");
/// Embedded locale file contents for ar-SA.
const AR_SA_CORE: &str = include_str!("../../locales/ar-SA/core.ftl");

/// Service for translating user-facing strings.
///
/// Uses Fluent `.ftl` locale files loaded at construction time.
/// Supports runtime locale switching and argument interpolation.
///
/// Locales without dedicated `.ftl` files fall back to en-US.
pub struct I18nService {
    current_locale: SupportedLocale,
    bundles: HashMap<SupportedLocale, FluentBundle<FluentResource>>,
}

impl I18nService {
    /// Creates a new i18n service with the specified locale.
    ///
    /// Loads all available `.ftl` locale files at creation time.
    /// Locales without `.ftl` files will fall back to en-US at translation time.
    pub fn new(locale: SupportedLocale) -> Self {
        let mut bundles = HashMap::new();

        let locale_sources: &[(SupportedLocale, &str)] = &[
            (SupportedLocale::EnUs, EN_US_CORE),
            (SupportedLocale::EsEs, ES_ES_CORE),
            (SupportedLocale::FrFr, FR_FR_CORE),
            (SupportedLocale::DeDe, DE_DE_CORE),
            (SupportedLocale::PtBr, PT_BR_CORE),
            (SupportedLocale::JaJp, JA_JP_CORE),
            (SupportedLocale::ZhCn, ZH_CN_CORE),
            (SupportedLocale::KoKr, KO_KR_CORE),
            (SupportedLocale::HiIn, HI_IN_CORE),
            (SupportedLocale::ArSa, AR_SA_CORE),
        ];

        for (supported_locale, source) in locale_sources {
            if let Some(bundle) = Self::build_bundle(*supported_locale, source) {
                bundles.insert(*supported_locale, bundle);
            }
        }

        Self {
            current_locale: locale,
            bundles,
        }
    }

    /// Translates a message key with interpolation arguments.
    ///
    /// Returns the translated string for the current locale.
    /// If the key is not found in the current locale, falls back to en-US.
    /// If the key is not found in any locale, returns the key string itself.
    ///
    /// # Arguments
    /// * `key` - The Fluent message identifier (e.g., `"error-config-file-not-found"`)
    /// * `args` - Key-value pairs for interpolation (e.g., `&[("path", "/etc/config")]`)
    pub fn translate(&self, key: &str, args: &[(&str, &str)]) -> String {
        if let Some(result) = self.try_translate_with_locale(self.current_locale, key, args) {
            return result;
        }

        if self.current_locale != SupportedLocale::EnUs {
            if let Some(result) = self.try_translate_with_locale(SupportedLocale::EnUs, key, args) {
                return result;
            }
        }

        key.to_string()
    }

    /// Shorthand: translate with no arguments.
    pub fn t(&self, key: &str) -> String {
        self.translate(key, &[])
    }

    /// Switches the active locale at runtime.
    pub fn set_locale(&mut self, locale: SupportedLocale) {
        self.current_locale = locale;
    }

    /// Returns the current locale.
    pub fn current_locale(&self) -> SupportedLocale {
        self.current_locale
    }

    /// Lists all available locales.
    pub fn available_locales() -> &'static [SupportedLocale] {
        super::locale::ALL
    }

    /// Attempts to translate a key using a specific locale's bundle.
    fn try_translate_with_locale(
        &self,
        locale: SupportedLocale,
        key: &str,
        args: &[(&str, &str)],
    ) -> Option<String> {
        let bundle = self.bundles.get(&locale)?;
        let message = bundle.get_message(key)?;
        let pattern = message.value()?;

        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, FluentValue::from(*v));
        }

        let mut errors = Vec::new();
        let result = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);

        Some(result.into_owned())
    }

    /// Builds a `FluentBundle` from raw `.ftl` source text.
    fn build_bundle(
        locale: SupportedLocale,
        ftl_source: &str,
    ) -> Option<FluentBundle<FluentResource>> {
        let resource = FluentResource::try_new(ftl_source.to_string()).ok()?;
        let lang_id: LanguageIdentifier = locale.as_code().parse().ok()?;
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.set_use_isolating(false);
        bundle.add_resource(resource).ok()?;
        Some(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_en_us_with_args() {
        let service = I18nService::new(SupportedLocale::EnUs);
        let result = service.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
        assert_eq!(result, "Configuration file not found at /etc/foo");
    }

    #[test]
    fn translate_es_es_with_args() {
        let service = I18nService::new(SupportedLocale::EsEs);
        let result = service.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
        assert_eq!(result, "Archivo de configuración no encontrado en /etc/foo");
    }

    #[test]
    fn translate_missing_key_returns_key() {
        let service = I18nService::new(SupportedLocale::EnUs);
        let result = service.translate("nonexistent-key", &[]);
        assert_eq!(result, "nonexistent-key");
    }

    #[test]
    fn t_shorthand_works() {
        let service = I18nService::new(SupportedLocale::EnUs);
        let result = service.t("app-ready");
        assert_eq!(result, "Mahalaxmi is ready");
    }

    #[test]
    fn set_locale_switches_translation() {
        let mut service = I18nService::new(SupportedLocale::EnUs);
        assert_eq!(service.t("app-ready"), "Mahalaxmi is ready");

        service.set_locale(SupportedLocale::EsEs);
        assert_eq!(service.t("app-ready"), "Mahalaxmi está listo");
    }

    #[test]
    fn missing_key_falls_back_to_en_us() {
        let mut service = I18nService::new(SupportedLocale::KoKr);
        let result = service.translate("nonexistent-key-xyz", &[]);
        assert_eq!(result, "nonexistent-key-xyz");

        service.set_locale(SupportedLocale::ArSa);
        let result = service.translate("nonexistent-key-xyz", &[]);
        assert_eq!(result, "nonexistent-key-xyz");
    }

    #[test]
    fn current_locale_returns_set_locale() {
        let mut service = I18nService::new(SupportedLocale::EnUs);
        assert_eq!(service.current_locale(), SupportedLocale::EnUs);

        service.set_locale(SupportedLocale::EsEs);
        assert_eq!(service.current_locale(), SupportedLocale::EsEs);
    }

    #[test]
    fn available_locales_returns_all() {
        let locales = I18nService::available_locales();
        assert_eq!(locales.len(), 10);
        assert!(locales.contains(&SupportedLocale::EnUs));
        assert!(locales.contains(&SupportedLocale::ArSa));
    }

    #[test]
    fn translate_no_args_message() {
        let service = I18nService::new(SupportedLocale::EnUs);
        assert_eq!(
            service.t("config-using-default"),
            "No configuration file found, using defaults"
        );
    }

    #[test]
    fn translate_multiple_args() {
        let service = I18nService::new(SupportedLocale::EnUs);
        let result = service.translate(
            "error-log-dir-create-failed",
            &[("path", "/var/log"), ("reason", "permission denied")],
        );
        assert_eq!(
            result,
            "Failed to create log directory at /var/log: permission denied"
        );
    }

    #[test]
    fn translate_app_starting_with_version() {
        let service = I18nService::new(SupportedLocale::EnUs);
        let result = service.translate("app-starting", &[("version", "0.1.0")]);
        assert_eq!(result, "Mahalaxmi v0.1.0 starting");
    }

    #[test]
    fn translate_es_es_app_starting() {
        let service = I18nService::new(SupportedLocale::EsEs);
        let result = service.translate("app-starting", &[("version", "0.1.0")]);
        assert_eq!(result, "Mahalaxmi v0.1.0 iniciando");
    }

    #[test]
    fn translate_all_en_us_keys() {
        let service = I18nService::new(SupportedLocale::EnUs);

        let keys_with_args: Vec<(&str, Vec<(&str, &str)>)> = vec![
            ("error-config-file-not-found", vec![("path", "/test")]),
            ("error-config-parse-failed", vec![("reason", "bad syntax")]),
            (
                "error-config-validation-failed",
                vec![("details", "missing field")],
            ),
            ("error-locale-not-supported", vec![("locale", "xx-XX")]),
            ("error-log-init-failed", vec![("reason", "io error")]),
            (
                "error-log-dir-create-failed",
                vec![("path", "/tmp"), ("reason", "full")],
            ),
            ("config-loaded-successfully", vec![("path", "/etc/config")]),
            ("config-using-default", vec![]),
            ("config-env-override", vec![("var", "MAHALAXMI_PORT")]),
            ("logging-initialized", vec![("level", "debug")]),
            ("app-starting", vec![("version", "1.0.0")]),
            ("app-ready", vec![]),
            ("app-shutting-down", vec![]),
        ];

        for (key, args) in &keys_with_args {
            let result = service.translate(key, args);
            assert_ne!(result, *key, "Key '{}' was not found in en-US locale", key);
        }
    }

    #[test]
    fn translate_all_es_es_keys() {
        let service = I18nService::new(SupportedLocale::EsEs);

        let keys_with_args: Vec<(&str, Vec<(&str, &str)>)> = vec![
            ("error-config-file-not-found", vec![("path", "/test")]),
            ("error-config-parse-failed", vec![("reason", "sintaxis")]),
            (
                "error-config-validation-failed",
                vec![("details", "campo faltante")],
            ),
            ("error-locale-not-supported", vec![("locale", "xx-XX")]),
            ("error-log-init-failed", vec![("reason", "error io")]),
            (
                "error-log-dir-create-failed",
                vec![("path", "/tmp"), ("reason", "lleno")],
            ),
            ("config-loaded-successfully", vec![("path", "/etc/config")]),
            ("config-using-default", vec![]),
            ("config-env-override", vec![("var", "MAHALAXMI_PORT")]),
            ("logging-initialized", vec![("level", "depuración")]),
            ("app-starting", vec![("version", "1.0.0")]),
            ("app-ready", vec![]),
            ("app-shutting-down", vec![]),
        ];

        for (key, args) in &keys_with_args {
            let result = service.translate(key, args);
            assert_ne!(result, *key, "Key '{}' was not found in es-ES locale", key);
        }
    }

    /// Helper: returns all message keys with sample arguments for comprehensive locale testing.
    fn all_keys_with_args() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
        vec![
            ("error-config-file-not-found", vec![("path", "/test")]),
            ("error-config-parse-failed", vec![("reason", "test")]),
            ("error-config-validation-failed", vec![("details", "test")]),
            ("error-locale-not-supported", vec![("locale", "xx-XX")]),
            ("error-log-init-failed", vec![("reason", "test")]),
            (
                "error-log-dir-create-failed",
                vec![("path", "/tmp"), ("reason", "test")],
            ),
            ("error-app-launch-failed", vec![("reason", "test")]),
            (
                "validation-invalid-log-level",
                vec![("level", "bad"), ("valid", "info,debug")],
            ),
            (
                "validation-workers-out-of-range",
                vec![("min", "1"), ("max", "64"), ("value", "0")],
            ),
            (
                "validation-manager-timeout-too-low",
                vec![("min", "30"), ("value", "5")],
            ),
            (
                "validation-worker-timeout-too-low",
                vec![("min", "30"), ("value", "5")],
            ),
            (
                "validation-offline-grace-too-low",
                vec![("min", "1"), ("value", "0")],
            ),
            (
                "validation-invalid-consensus-strategy",
                vec![("value", "bad"), ("valid", "union,intersection")],
            ),
            ("validation-invalid-data-directory", vec![]),
            ("validation-empty-default-provider", vec![]),
            (
                "validation-invalid-theme",
                vec![("value", "bad"), ("valid", "light,dark")],
            ),
            (
                "validation-font-size-out-of-range",
                vec![("min", "8"), ("max", "72"), ("value", "2")],
            ),
            ("config-loaded-successfully", vec![("path", "/etc/config")]),
            ("config-using-default", vec![]),
            ("config-env-override", vec![("var", "MAHALAXMI_PORT")]),
            (
                "config-env-override-invalid",
                vec![("var", "MAHALAXMI_PORT"), ("value", "abc")],
            ),
            (
                "config-generated-successfully",
                vec![("path", "/etc/config")],
            ),
            ("config-already-exists", vec![("path", "/etc/config")]),
            ("logging-initialized", vec![("level", "debug")]),
            ("logging-rust-log-override", vec![]),
            ("logging-file-path", vec![("path", "/var/log/app.log")]),
            (
                "logging-dir-create-failed-fallback",
                vec![("path", "/var/log")],
            ),
            ("app-starting", vec![("version", "1.0.0")]),
            ("app-ready", vec![]),
            ("app-shutting-down", vec![]),
            ("pty-open-failed", vec![("reason", "test")]),
            (
                "pty-spawn-failed",
                vec![("program", "echo"), ("reason", "test")],
            ),
            (
                "pty-write-failed",
                vec![("terminal_id", "abc-123"), ("reason", "test")],
            ),
            (
                "pty-read-failed",
                vec![("terminal_id", "abc-123"), ("reason", "test")],
            ),
            (
                "pty-resize-failed",
                vec![
                    ("terminal_id", "abc-123"),
                    ("rows", "24"),
                    ("cols", "80"),
                    ("reason", "test"),
                ],
            ),
            (
                "pty-wait-failed",
                vec![("terminal_id", "abc-123"), ("reason", "test")],
            ),
            (
                "pty-kill-failed",
                vec![("terminal_id", "abc-123"), ("reason", "test")],
            ),
        ]
    }

    #[test]
    fn all_locales_load_successfully() {
        for locale in super::super::locale::ALL {
            let service = I18nService::new(*locale);
            assert_eq!(service.current_locale(), *locale);
            let result = service.t("app-ready");
            assert_ne!(
                result,
                "app-ready",
                "Locale {} failed to load app-ready",
                locale.as_code()
            );
        }
    }

    #[test]
    fn all_locales_have_all_keys() {
        let keys = all_keys_with_args();
        for locale in super::super::locale::ALL {
            let service = I18nService::new(*locale);
            for (key, args) in &keys {
                let result = service.translate(key, args);
                assert_ne!(
                    result,
                    *key,
                    "Key '{}' not found in locale {}",
                    key,
                    locale.as_code()
                );
            }
        }
    }

    #[test]
    fn de_de_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::DeDe);
        assert_eq!(service.t("app-ready"), "Mahalaxmi ist bereit");
        assert_eq!(
            service.t("app-shutting-down"),
            "Mahalaxmi wird heruntergefahren"
        );
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 wird gestartet");
    }

    #[test]
    fn pt_br_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::PtBr);
        assert_eq!(service.t("app-ready"), "Mahalaxmi está pronto");
        assert_eq!(service.t("app-shutting-down"), "Mahalaxmi encerrando");
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 iniciando");
    }

    #[test]
    fn ja_jp_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::JaJp);
        assert_eq!(service.t("app-ready"), "Mahalaxmiの準備が完了しました");
        assert_eq!(
            service.t("app-shutting-down"),
            "Mahalaxmiをシャットダウンしています"
        );
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 を起動しています");
    }

    #[test]
    fn zh_cn_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::ZhCn);
        assert_eq!(service.t("app-ready"), "Mahalaxmi已就绪");
        assert_eq!(service.t("app-shutting-down"), "Mahalaxmi正在关闭");
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 正在启动");
    }

    #[test]
    fn ko_kr_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::KoKr);
        assert_eq!(service.t("app-ready"), "Mahalaxmi가 준비되었습니다");
        assert_eq!(service.t("app-shutting-down"), "Mahalaxmi를 종료하는 중");
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 시작 중");
    }

    #[test]
    fn hi_in_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::HiIn);
        assert_eq!(service.t("app-ready"), "Mahalaxmi तैयार है");
        assert_eq!(service.t("app-shutting-down"), "Mahalaxmi बंद हो रहा है");
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi v1.0.0 प्रारंभ हो रहा है");
    }

    #[test]
    fn ar_sa_locale_translates_correctly() {
        let service = I18nService::new(SupportedLocale::ArSa);
        assert_eq!(service.t("app-ready"), "Mahalaxmi جاهز");
        assert_eq!(service.t("app-shutting-down"), "جارٍ إيقاف Mahalaxmi");
        let result = service.translate("app-starting", &[("version", "1.0.0")]);
        assert_eq!(result, "Mahalaxmi الإصدار 1.0.0 قيد التشغيل");
    }

    #[test]
    fn switch_to_each_locale_at_runtime() {
        let mut service = I18nService::new(SupportedLocale::EnUs);

        let locale_expected_ready: &[(SupportedLocale, &str)] = &[
            (SupportedLocale::EnUs, "Mahalaxmi is ready"),
            (SupportedLocale::EsEs, "Mahalaxmi está listo"),
            (SupportedLocale::FrFr, "Mahalaxmi est prêt"),
            (SupportedLocale::DeDe, "Mahalaxmi ist bereit"),
            (SupportedLocale::PtBr, "Mahalaxmi está pronto"),
            (SupportedLocale::JaJp, "Mahalaxmiの準備が完了しました"),
            (SupportedLocale::ZhCn, "Mahalaxmi已就绪"),
            (SupportedLocale::KoKr, "Mahalaxmi가 준비되었습니다"),
            (SupportedLocale::HiIn, "Mahalaxmi तैयार है"),
            (SupportedLocale::ArSa, "Mahalaxmi جاهز"),
        ];

        for (locale, expected) in locale_expected_ready {
            service.set_locale(*locale);
            assert_eq!(
                service.t("app-ready"),
                *expected,
                "Locale {} returned wrong translation for app-ready",
                locale.as_code()
            );
        }
    }

    #[test]
    fn non_english_locales_differ_from_english() {
        let en_service = I18nService::new(SupportedLocale::EnUs);
        let en_ready = en_service.t("app-ready");

        let non_english = [
            SupportedLocale::EsEs,
            SupportedLocale::FrFr,
            SupportedLocale::DeDe,
            SupportedLocale::PtBr,
            SupportedLocale::JaJp,
            SupportedLocale::ZhCn,
            SupportedLocale::KoKr,
            SupportedLocale::HiIn,
            SupportedLocale::ArSa,
        ];

        for locale in non_english {
            let service = I18nService::new(locale);
            let translated = service.t("app-ready");
            assert_ne!(
                translated,
                en_ready,
                "Locale {} returned same text as en-US for app-ready",
                locale.as_code()
            );
        }
    }

    #[test]
    fn all_10_bundles_loaded() {
        let service = I18nService::new(SupportedLocale::EnUs);
        assert_eq!(service.bundles.len(), 10);
    }
}
