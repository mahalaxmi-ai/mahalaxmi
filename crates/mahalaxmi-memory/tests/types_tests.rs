//! Tests for memory type system.

use mahalaxmi_memory::types::confidence::ConfidenceLevel;
use mahalaxmi_memory::types::memory_type::MemoryType;
use mahalaxmi_memory::types::source::MemorySource;
use mahalaxmi_memory::types::MemoryId;
use uuid::Uuid;

// MemoryId tests

#[test]
fn memory_id_generates_unique_ids() {
    let ids: Vec<MemoryId> = (0..100).map(|_| MemoryId::new()).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
}

#[test]
fn memory_id_roundtrip_via_uuid() {
    let uuid = Uuid::new_v4();
    let id = MemoryId::from_uuid(uuid);
    assert_eq!(*id.as_uuid(), uuid);
}

#[test]
fn memory_id_display_matches_uuid() {
    let uuid = Uuid::new_v4();
    let id = MemoryId::from_uuid(uuid);
    assert_eq!(format!("{id}"), format!("{uuid}"));
}

#[test]
fn memory_id_serialization_roundtrip() {
    let id = MemoryId::new();
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: MemoryId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

// MemoryType tests

#[test]
fn memory_type_default_confidences() {
    assert!((MemoryType::CodebaseFact.default_confidence() - 0.7).abs() < f64::EPSILON);
    assert!((MemoryType::Convention.default_confidence() - 0.6).abs() < f64::EPSILON);
    assert!((MemoryType::Decision.default_confidence() - 0.8).abs() < f64::EPSILON);
    assert!((MemoryType::Warning.default_confidence() - 0.9).abs() < f64::EPSILON);
}

#[test]
fn memory_type_as_str() {
    assert_eq!(MemoryType::CodebaseFact.as_str(), "codebase_fact");
    assert_eq!(MemoryType::Convention.as_str(), "convention");
    assert_eq!(MemoryType::Decision.as_str(), "decision");
    assert_eq!(MemoryType::Warning.as_str(), "warning");
}

#[test]
fn memory_type_display() {
    for mt in [
        MemoryType::CodebaseFact,
        MemoryType::Convention,
        MemoryType::Decision,
        MemoryType::Warning,
    ] {
        assert_eq!(format!("{mt}"), mt.as_str());
    }
}

// ConfidenceLevel tests

#[test]
fn confidence_level_boundaries() {
    assert_eq!(ConfidenceLevel::from_score(0.0), ConfidenceLevel::Low);
    assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
    assert_eq!(ConfidenceLevel::from_score(0.31), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_score(0.7), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_score(0.71), ConfidenceLevel::High);
    assert_eq!(ConfidenceLevel::from_score(1.0), ConfidenceLevel::High);
}

#[test]
fn confidence_level_clamps_out_of_range() {
    assert_eq!(ConfidenceLevel::from_score(-1.0), ConfidenceLevel::Low);
    assert_eq!(ConfidenceLevel::from_score(2.0), ConfidenceLevel::High);
}

#[test]
fn confidence_level_ranges() {
    assert_eq!(ConfidenceLevel::Low.as_range(), (0.0, 0.3));
    assert_eq!(ConfidenceLevel::Medium.as_range(), (0.3, 0.7));
    assert_eq!(ConfidenceLevel::High.as_range(), (0.7, 1.0));
}

// MemorySource tests

#[test]
fn memory_source_as_str() {
    assert_eq!(
        MemorySource::Worker {
            worker_id: "w1".into()
        }
        .as_str(),
        "worker"
    );
    assert_eq!(
        MemorySource::Manager {
            manager_id: "m1".into()
        }
        .as_str(),
        "manager"
    );
    assert_eq!(MemorySource::System.as_str(), "system");
    assert_eq!(MemorySource::User.as_str(), "user");
}

#[test]
fn memory_source_source_id() {
    assert_eq!(
        MemorySource::Worker {
            worker_id: "w1".into()
        }
        .source_id(),
        Some("w1")
    );
    assert_eq!(
        MemorySource::Manager {
            manager_id: "m1".into()
        }
        .source_id(),
        Some("m1")
    );
    assert_eq!(MemorySource::System.source_id(), None);
    assert_eq!(MemorySource::User.source_id(), None);
}

#[test]
fn memory_source_serialization_roundtrip() {
    let sources = vec![
        MemorySource::Worker {
            worker_id: "w-42".into(),
        },
        MemorySource::Manager {
            manager_id: "m-1".into(),
        },
        MemorySource::System,
        MemorySource::User,
    ];
    for source in sources {
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: MemorySource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, deserialized);
    }
}
