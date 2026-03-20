//! Locale definitions and utilities for the Mahalaxmi i18n system.

use serde::{Deserialize, Serialize};

/// All supported locales in the Mahalaxmi system.
///
/// Every user-facing string is translated through this locale system.
/// New locales can be added by extending this enum and adding corresponding `.ftl` files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedLocale {
    /// English (US) — default locale
    #[default]
    EnUs,
    /// Spanish (Spain)
    EsEs,
    /// French (France)
    FrFr,
    /// German (Germany)
    DeDe,
    /// Portuguese (Brazil)
    PtBr,
    /// Japanese
    JaJp,
    /// Chinese (Simplified)
    ZhCn,
    /// Korean
    KoKr,
    /// Hindi
    HiIn,
    /// Arabic (Saudi Arabia)
    ArSa,
}

/// Array of all supported locale variants.
pub const ALL: &[SupportedLocale] = &[
    SupportedLocale::EnUs,
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

impl SupportedLocale {
    /// Returns the BCP 47 language tag for this locale.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_core::i18n::SupportedLocale;
    /// assert_eq!(SupportedLocale::EnUs.as_code(), "en-US");
    /// assert_eq!(SupportedLocale::EsEs.as_code(), "es-ES");
    /// ```
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::EsEs => "es-ES",
            Self::FrFr => "fr-FR",
            Self::DeDe => "de-DE",
            Self::PtBr => "pt-BR",
            Self::JaJp => "ja-JP",
            Self::ZhCn => "zh-CN",
            Self::KoKr => "ko-KR",
            Self::HiIn => "hi-IN",
            Self::ArSa => "ar-SA",
        }
    }

    /// Looks up a locale by its BCP 47 language tag (case-insensitive).
    ///
    /// Returns `None` for unrecognized codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_core::i18n::SupportedLocale;
    /// assert_eq!(SupportedLocale::from_code("en-US"), Some(SupportedLocale::EnUs));
    /// assert_eq!(SupportedLocale::from_code("es-es"), Some(SupportedLocale::EsEs));
    /// assert_eq!(SupportedLocale::from_code("EN-US"), Some(SupportedLocale::EnUs));
    /// assert_eq!(SupportedLocale::from_code("xx-XX"), None);
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        let normalized = code.to_lowercase();
        match normalized.as_str() {
            "en-us" => Some(Self::EnUs),
            "es-es" => Some(Self::EsEs),
            "fr-fr" => Some(Self::FrFr),
            "de-de" => Some(Self::DeDe),
            "pt-br" => Some(Self::PtBr),
            "ja-jp" => Some(Self::JaJp),
            "zh-cn" => Some(Self::ZhCn),
            "ko-kr" => Some(Self::KoKr),
            "hi-in" => Some(Self::HiIn),
            "ar-sa" => Some(Self::ArSa),
            _ => None,
        }
    }

    /// Returns the full English display name for this locale.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_core::i18n::SupportedLocale;
    /// assert_eq!(SupportedLocale::EnUs.display_name(), "English (US)");
    /// assert_eq!(SupportedLocale::JaJp.display_name(), "Japanese");
    /// ```
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::EnUs => "English (US)",
            Self::EsEs => "Spanish (Spain)",
            Self::FrFr => "French (France)",
            Self::DeDe => "German (Germany)",
            Self::PtBr => "Portuguese (Brazil)",
            Self::JaJp => "Japanese",
            Self::ZhCn => "Chinese (Simplified)",
            Self::KoKr => "Korean",
            Self::HiIn => "Hindi",
            Self::ArSa => "Arabic (Saudi Arabia)",
        }
    }

    /// Returns the name of this locale in its native language.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_core::i18n::SupportedLocale;
    /// assert_eq!(SupportedLocale::EnUs.native_name(), "English");
    /// assert_eq!(SupportedLocale::EsEs.native_name(), "Español");
    /// ```
    pub fn native_name(&self) -> &'static str {
        match self {
            Self::EnUs => "English",
            Self::EsEs => "Español",
            Self::FrFr => "Français",
            Self::DeDe => "Deutsch",
            Self::PtBr => "Português",
            Self::JaJp => "日本語",
            Self::ZhCn => "中文",
            Self::KoKr => "한국어",
            Self::HiIn => "हिन्दी",
            Self::ArSa => "العربية",
        }
    }

    /// Returns `true` if this locale uses right-to-left text direction.
    ///
    /// Currently only Arabic (Saudi Arabia) is RTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_core::i18n::SupportedLocale;
    /// assert!(!SupportedLocale::EnUs.is_rtl());
    /// assert!(SupportedLocale::ArSa.is_rtl());
    /// ```
    pub fn is_rtl(&self) -> bool {
        matches!(self, Self::ArSa)
    }
}

impl std::fmt::Display for SupportedLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_code_returns_correct_bcp47_tags() {
        assert_eq!(SupportedLocale::EnUs.as_code(), "en-US");
        assert_eq!(SupportedLocale::EsEs.as_code(), "es-ES");
        assert_eq!(SupportedLocale::FrFr.as_code(), "fr-FR");
        assert_eq!(SupportedLocale::DeDe.as_code(), "de-DE");
        assert_eq!(SupportedLocale::PtBr.as_code(), "pt-BR");
        assert_eq!(SupportedLocale::JaJp.as_code(), "ja-JP");
        assert_eq!(SupportedLocale::ZhCn.as_code(), "zh-CN");
        assert_eq!(SupportedLocale::KoKr.as_code(), "ko-KR");
        assert_eq!(SupportedLocale::HiIn.as_code(), "hi-IN");
        assert_eq!(SupportedLocale::ArSa.as_code(), "ar-SA");
    }

    #[test]
    fn from_code_case_insensitive_lookup() {
        assert_eq!(
            SupportedLocale::from_code("en-US"),
            Some(SupportedLocale::EnUs)
        );
        assert_eq!(
            SupportedLocale::from_code("en-us"),
            Some(SupportedLocale::EnUs)
        );
        assert_eq!(
            SupportedLocale::from_code("EN-US"),
            Some(SupportedLocale::EnUs)
        );
        assert_eq!(
            SupportedLocale::from_code("Es-Es"),
            Some(SupportedLocale::EsEs)
        );
        assert_eq!(
            SupportedLocale::from_code("es-ES"),
            Some(SupportedLocale::EsEs)
        );
    }

    #[test]
    fn from_code_returns_none_for_invalid() {
        assert_eq!(SupportedLocale::from_code("xx-XX"), None);
        assert_eq!(SupportedLocale::from_code(""), None);
        assert_eq!(SupportedLocale::from_code("english"), None);
        assert_eq!(SupportedLocale::from_code("en"), None);
    }

    #[test]
    fn display_names_are_correct() {
        assert_eq!(SupportedLocale::EnUs.display_name(), "English (US)");
        assert_eq!(SupportedLocale::EsEs.display_name(), "Spanish (Spain)");
        assert_eq!(SupportedLocale::JaJp.display_name(), "Japanese");
        assert_eq!(
            SupportedLocale::ArSa.display_name(),
            "Arabic (Saudi Arabia)"
        );
    }

    #[test]
    fn native_names_are_correct() {
        assert_eq!(SupportedLocale::EnUs.native_name(), "English");
        assert_eq!(SupportedLocale::EsEs.native_name(), "Español");
        assert_eq!(SupportedLocale::FrFr.native_name(), "Français");
        assert_eq!(SupportedLocale::DeDe.native_name(), "Deutsch");
        assert_eq!(SupportedLocale::JaJp.native_name(), "日本語");
    }

    #[test]
    fn rtl_only_for_arabic() {
        assert!(!SupportedLocale::EnUs.is_rtl());
        assert!(!SupportedLocale::EsEs.is_rtl());
        assert!(!SupportedLocale::HiIn.is_rtl());
        assert!(SupportedLocale::ArSa.is_rtl());
    }

    #[test]
    fn default_is_en_us() {
        assert_eq!(SupportedLocale::default(), SupportedLocale::EnUs);
    }

    #[test]
    fn all_contains_every_variant() {
        assert_eq!(ALL.len(), 10);
        assert!(ALL.contains(&SupportedLocale::EnUs));
        assert!(ALL.contains(&SupportedLocale::ArSa));
    }

    #[test]
    fn display_shows_code() {
        assert_eq!(format!("{}", SupportedLocale::EnUs), "en-US");
        assert_eq!(format!("{}", SupportedLocale::EsEs), "es-ES");
    }

    #[test]
    fn serde_roundtrip() {
        let locale = SupportedLocale::EsEs;
        let json = serde_json::to_string(&locale).unwrap();
        assert_eq!(json, "\"es-es\"");
        let deserialized: SupportedLocale = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SupportedLocale::EsEs);
    }
}
