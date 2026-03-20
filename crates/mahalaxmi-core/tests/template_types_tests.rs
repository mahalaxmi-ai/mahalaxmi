use mahalaxmi_core::types::{
    CategoryId, LicenseTier, TemplateDifficulty, TemplateId, ValidationSeverity,
};

// === CategoryId tests ===

#[test]
fn category_id_from_string() {
    let id = CategoryId::new("software");
    assert_eq!(id.as_str(), "software");
}

#[test]
fn category_id_display() {
    let id = CategoryId::new("legal");
    assert_eq!(id.to_string(), "legal");
}

#[test]
fn category_id_serde_roundtrip() {
    let id = CategoryId::new("music");
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: CategoryId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn category_id_equality() {
    assert_eq!(CategoryId::new("x"), CategoryId::new("x"));
    assert_ne!(CategoryId::new("x"), CategoryId::new("y"));
}

// === TemplateId tests ===

#[test]
fn template_id_from_string() {
    let id = TemplateId::new("web-app");
    assert_eq!(id.as_str(), "web-app");
}

#[test]
fn template_id_display() {
    let id = TemplateId::new("cli-tool");
    assert_eq!(id.to_string(), "cli-tool");
}

#[test]
fn template_id_serde_roundtrip() {
    let id = TemplateId::new("api-server");
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: TemplateId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

// === TemplateDifficulty tests ===

#[test]
fn template_difficulty_from_config_str() {
    assert_eq!(
        TemplateDifficulty::from_config_str("beginner"),
        Some(TemplateDifficulty::Beginner)
    );
    assert_eq!(
        TemplateDifficulty::from_config_str("expert"),
        Some(TemplateDifficulty::Expert)
    );
    assert_eq!(
        TemplateDifficulty::from_config_str("ADVANCED"),
        Some(TemplateDifficulty::Advanced)
    );
    assert_eq!(TemplateDifficulty::from_config_str("bad"), None);
}

#[test]
fn template_difficulty_as_str_roundtrip() {
    let variants = [
        TemplateDifficulty::Beginner,
        TemplateDifficulty::Intermediate,
        TemplateDifficulty::Advanced,
        TemplateDifficulty::Expert,
    ];
    for variant in &variants {
        assert_eq!(
            TemplateDifficulty::from_config_str(variant.as_str()),
            Some(*variant)
        );
    }
}

#[test]
fn template_difficulty_display() {
    assert_eq!(TemplateDifficulty::Expert.to_string(), "expert");
    assert_eq!(TemplateDifficulty::Beginner.to_string(), "beginner");
}

#[test]
fn template_difficulty_serde_roundtrip() {
    let variants = [
        TemplateDifficulty::Beginner,
        TemplateDifficulty::Intermediate,
        TemplateDifficulty::Advanced,
        TemplateDifficulty::Expert,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let deserialized: TemplateDifficulty = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, deserialized);
    }
}

// === LicenseTier tests ===

#[test]
fn license_tier_ordering() {
    assert!(LicenseTier::Trial < LicenseTier::Basic);
    assert!(LicenseTier::Basic < LicenseTier::Pro);
    assert!(LicenseTier::Pro < LicenseTier::AllAccess);
    assert!(LicenseTier::AllAccess < LicenseTier::Enterprise);
}

#[test]
fn license_tier_includes_tier() {
    assert!(LicenseTier::Pro.includes_tier(LicenseTier::Basic));
    assert!(!LicenseTier::Basic.includes_tier(LicenseTier::Pro));
    assert!(LicenseTier::Enterprise.includes_tier(LicenseTier::Trial));
    assert!(!LicenseTier::Trial.includes_tier(LicenseTier::Enterprise));
    assert!(LicenseTier::Pro.includes_tier(LicenseTier::Pro));
}

#[test]
fn license_tier_max_workers() {
    assert_eq!(LicenseTier::Trial.max_workers(), 5);
    assert_eq!(LicenseTier::Basic.max_workers(), 20);
    assert_eq!(LicenseTier::Pro.max_workers(), 50);
    assert_eq!(LicenseTier::AllAccess.max_workers(), 100);
    assert_eq!(LicenseTier::Enterprise.max_workers(), 500);
}

#[test]
fn license_tier_serde_roundtrip() {
    let variants = [
        LicenseTier::Trial,
        LicenseTier::Basic,
        LicenseTier::Pro,
        LicenseTier::AllAccess,
        LicenseTier::Enterprise,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let deserialized: LicenseTier = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, deserialized);
    }
}

// === ValidationSeverity tests ===

#[test]
fn validation_severity_display() {
    assert_eq!(ValidationSeverity::Info.to_string(), "Info");
    assert_eq!(ValidationSeverity::Warning.to_string(), "Warning");
    assert_eq!(ValidationSeverity::Error.to_string(), "Error");
}
