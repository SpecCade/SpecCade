//! Audit logic and parity tracking
//!
//! Handles key classification against the parity matrix and audit operations.

use anyhow::Result;
use std::path::Path;

use crate::parity_data::{self, KeyStatus};

use super::legacy_parser::{parse_legacy_spec_exec, parse_legacy_spec_static, LegacySpec};

/// Migration key status for classification
/// Maps to parity_data::KeyStatus but adds Unknown for keys not in parity matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationKeyStatus {
    Implemented,
    Partial,
    NotImplemented,
    Deprecated,
    Unknown,
}

impl From<KeyStatus> for MigrationKeyStatus {
    fn from(status: KeyStatus) -> Self {
        match status {
            KeyStatus::Implemented => MigrationKeyStatus::Implemented,
            KeyStatus::Partial => MigrationKeyStatus::Partial,
            KeyStatus::NotImplemented => MigrationKeyStatus::NotImplemented,
            KeyStatus::Deprecated => MigrationKeyStatus::Deprecated,
        }
    }
}

impl std::fmt::Display for MigrationKeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationKeyStatus::Implemented => write!(f, "Implemented"),
            MigrationKeyStatus::Partial => write!(f, "Partial"),
            MigrationKeyStatus::NotImplemented => write!(f, "NotImplemented"),
            MigrationKeyStatus::Deprecated => write!(f, "Deprecated"),
            MigrationKeyStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Key classification results for a migrated file
#[derive(Debug, Default, Clone)]
pub struct KeyClassification {
    pub implemented: usize,
    pub partial: usize,
    pub not_implemented: usize,
    pub deprecated: usize,
    pub unknown: usize,
    /// Details for each key: (key_name, status)
    pub key_details: Vec<(String, MigrationKeyStatus)>,
}

impl KeyClassification {
    /// Total number of keys used (excluding deprecated)
    pub fn total_used(&self) -> usize {
        self.implemented + self.partial + self.not_implemented + self.unknown
    }

    /// Compute gap score: (implemented + 0.5*partial) / (total_used)
    /// Returns None if total_used is 0
    pub fn gap_score(&self) -> Option<f64> {
        let total = self.total_used();
        if total == 0 {
            return None;
        }
        let score = (self.implemented as f64 + 0.5 * self.partial as f64) / total as f64;
        Some(score)
    }
}

/// Audit result for a single spec file
#[derive(Debug)]
pub struct AuditEntry {
    pub source_path: std::path::PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub key_classification: KeyClassification,
}

/// Audit a single spec file (parse and classify keys without migrating)
pub fn audit_spec(spec_file: &Path, allow_exec: bool) -> Result<AuditEntry> {
    // Parse the legacy spec
    let legacy = if allow_exec {
        parse_legacy_spec_exec(spec_file)?
    } else {
        parse_legacy_spec_static(spec_file)?
    };

    // Classify legacy keys against parity matrix
    let key_classification = classify_legacy_keys(&legacy);

    Ok(AuditEntry {
        source_path: spec_file.to_path_buf(),
        success: true,
        error: None,
        key_classification,
    })
}

/// Map category to parity matrix section name
/// The parity matrix uses section names like "SOUND (audio_sfx)"
pub fn category_to_parity_section(category: &str) -> &'static str {
    match category {
        "sounds" => "SOUND (audio_sfx)",
        "instruments" => "INSTRUMENT (audio_instrument)",
        "music" => "SONG (music)",
        "textures" => "TEXTURE (texture_2d)",
        "normals" => "NORMAL (normal_map)",
        "meshes" => "MESH (static_mesh)",
        "characters" => "SPEC/CHARACTER (skeletal_mesh)",
        "animations" => "ANIMATION (skeletal_animation)",
        _ => "",
    }
}

/// Classify a single legacy key against the parity matrix
/// Returns the MigrationKeyStatus for the key
pub fn classify_key(section: &str, key: &str) -> MigrationKeyStatus {
    // Try "Top-Level Keys" table first (most common)
    if let Some(info) = parity_data::find(section, "Top-Level Keys", key) {
        return MigrationKeyStatus::from(info.status);
    }

    // Try other common tables based on section
    let tables_to_check: &[&str] = match section {
        "SOUND (audio_sfx)" => &["Layer Keys", "Envelope Keys (ADSR)", "Filter Keys"],
        "INSTRUMENT (audio_instrument)" => &["Synthesis Keys", "Oscillator Keys (Subtractive)", "Output Keys"],
        "SONG (music)" => &["Instrument Keys (inline or ref)", "Pattern Keys", "Note Keys (within pattern)", "Arrangement Entry Keys", "Automation Keys", "IT Options Keys"],
        "TEXTURE (texture_2d)" => &["Layer Keys", "Solid Layer", "Noise Layer", "Gradient Layer", "Checkerboard Layer", "Stripes Layer", "Wood Grain Layer", "Brick Layer"],
        "NORMAL (normal_map)" => &["Processing Keys", "Pattern Keys", "Bricks Pattern", "Tiles Pattern", "Hexagons Pattern", "Noise Pattern", "Scratches Pattern", "Rivets Pattern", "Weave Pattern"],
        "MESH (static_mesh)" => &["Cube", "Cylinder", "Sphere (UV)", "Icosphere", "Cone", "Torus", "Modifier Keys", "Bevel Modifier", "Decimate Modifier", "UV Keys", "Export Keys"],
        "SPEC/CHARACTER (skeletal_mesh)" => &["Skeleton Bone Keys", "Part Keys", "Step Keys", "Instance Keys", "Texturing Keys"],
        "ANIMATION (skeletal_animation)" => &["Pose Keys (per bone)", "Phase Keys", "IK Target Keyframe Keys", "Procedural Layer Keys", "Rig Setup Keys", "IK Chain Keys", "Constraint Keys", "Twist Bone Keys", "Bake Settings Keys", "Animator Rig Config Keys", "Conventions Keys"],
        _ => &[],
    };

    for table in tables_to_check {
        if let Some(info) = parity_data::find(section, table, key) {
            return MigrationKeyStatus::from(info.status);
        }
    }

    // Key not found in parity matrix
    MigrationKeyStatus::Unknown
}

/// Classify all top-level keys in a legacy spec dict
/// Returns a KeyClassification with counts and details
pub fn classify_legacy_keys(legacy: &LegacySpec) -> KeyClassification {
    let section = category_to_parity_section(&legacy.category);
    let mut classification = KeyClassification::default();

    for key in legacy.data.keys() {
        let status = classify_key(section, key);
        classification.key_details.push((key.clone(), status));

        match status {
            MigrationKeyStatus::Implemented => classification.implemented += 1,
            MigrationKeyStatus::Partial => classification.partial += 1,
            MigrationKeyStatus::NotImplemented => classification.not_implemented += 1,
            MigrationKeyStatus::Deprecated => classification.deprecated += 1,
            MigrationKeyStatus::Unknown => classification.unknown += 1,
        }
    }

    // Sort key_details by status (for consistent output)
    classification.key_details.sort_by(|a, b| {
        let status_order = |s: &MigrationKeyStatus| match s {
            MigrationKeyStatus::Implemented => 0,
            MigrationKeyStatus::Partial => 1,
            MigrationKeyStatus::NotImplemented => 2,
            MigrationKeyStatus::Unknown => 3,
            MigrationKeyStatus::Deprecated => 4,
        };
        status_order(&a.1).cmp(&status_order(&b.1)).then(a.0.cmp(&b.0))
    });

    classification
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_category_to_parity_section() {
        assert_eq!(category_to_parity_section("sounds"), "SOUND (audio_sfx)");
        assert_eq!(category_to_parity_section("instruments"), "INSTRUMENT (audio_instrument)");
        assert_eq!(category_to_parity_section("music"), "SONG (music)");
        assert_eq!(category_to_parity_section("textures"), "TEXTURE (texture_2d)");
        assert_eq!(category_to_parity_section("normals"), "NORMAL (normal_map)");
        assert_eq!(category_to_parity_section("meshes"), "MESH (static_mesh)");
        assert_eq!(category_to_parity_section("characters"), "SPEC/CHARACTER (skeletal_mesh)");
        assert_eq!(category_to_parity_section("animations"), "ANIMATION (skeletal_animation)");
        assert_eq!(category_to_parity_section("unknown"), "");
    }

    #[test]
    fn test_classify_key_known_implemented() {
        // "name" is a well-known implemented key in SOUND section
        let status = classify_key("SOUND (audio_sfx)", "name");
        assert_eq!(status, MigrationKeyStatus::Implemented);
    }

    #[test]
    fn test_classify_key_unknown() {
        // A completely unknown key should return Unknown
        let status = classify_key("SOUND (audio_sfx)", "totally_fake_key_that_doesnt_exist");
        assert_eq!(status, MigrationKeyStatus::Unknown);
    }

    #[test]
    fn test_classify_legacy_keys() {
        let legacy = LegacySpec {
            dict_name: "SOUND".to_string(),
            category: "sounds".to_string(),
            data: HashMap::from([
                ("name".to_string(), serde_json::json!("test")),
                ("duration".to_string(), serde_json::json!(1.0)),
                ("sample_rate".to_string(), serde_json::json!(22050)),
                ("fake_key".to_string(), serde_json::json!("unknown")),
            ]),
        };

        let classification = classify_legacy_keys(&legacy);

        // name, duration, sample_rate are all implemented in SOUND
        assert!(classification.implemented >= 3);
        // fake_key should be unknown
        assert!(classification.unknown >= 1);
        // Total should match
        assert_eq!(
            classification.implemented + classification.partial + classification.not_implemented + classification.deprecated + classification.unknown,
            4
        );
    }

    #[test]
    fn test_key_classification_gap_score() {
        // Test gap score calculation
        let mut kc = KeyClassification::default();
        kc.implemented = 8;
        kc.partial = 2;
        kc.not_implemented = 0;
        kc.unknown = 0;

        // (8 + 0.5*2) / 10 = 9/10 = 0.9
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.9).abs() < 0.001);

        // Test with mixed values
        let mut kc2 = KeyClassification::default();
        kc2.implemented = 4;
        kc2.partial = 2;
        kc2.not_implemented = 2;
        kc2.unknown = 2;

        // (4 + 0.5*2) / (4+2+2+2) = 5/10 = 0.5
        let gap2 = kc2.gap_score().unwrap();
        assert!((gap2 - 0.5).abs() < 0.001);

        // Deprecated keys are excluded from denominator
        kc2.deprecated = 5;
        // Still (4 + 0.5*2) / (4+2+2+2) = 5/10 = 0.5
        let gap3 = kc2.gap_score().unwrap();
        assert!((gap3 - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_key_classification_total_used() {
        let mut kc = KeyClassification::default();
        kc.implemented = 5;
        kc.partial = 3;
        kc.not_implemented = 2;
        kc.unknown = 1;
        kc.deprecated = 4;

        // total_used excludes deprecated
        assert_eq!(kc.total_used(), 5 + 3 + 2 + 1);
    }

    #[test]
    fn test_key_classification_gap_score_empty() {
        let kc = KeyClassification::default();
        assert!(kc.gap_score().is_none());
    }

    #[test]
    fn test_migration_key_status_from_key_status() {
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Implemented), MigrationKeyStatus::Implemented);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Partial), MigrationKeyStatus::Partial);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::NotImplemented), MigrationKeyStatus::NotImplemented);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Deprecated), MigrationKeyStatus::Deprecated);
    }

    #[test]
    fn test_migration_key_status_display() {
        assert_eq!(format!("{}", MigrationKeyStatus::Implemented), "Implemented");
        assert_eq!(format!("{}", MigrationKeyStatus::Partial), "Partial");
        assert_eq!(format!("{}", MigrationKeyStatus::NotImplemented), "NotImplemented");
        assert_eq!(format!("{}", MigrationKeyStatus::Deprecated), "Deprecated");
        assert_eq!(format!("{}", MigrationKeyStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_audit_entry_success() {
        let entry = AuditEntry {
            source_path: PathBuf::from("test.spec.py"),
            success: true,
            error: None,
            key_classification: KeyClassification::default(),
        };
        assert!(entry.success);
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_audit_entry_failure() {
        let entry = AuditEntry {
            source_path: PathBuf::from("test.spec.py"),
            success: false,
            error: Some("Parse error".to_string()),
            key_classification: KeyClassification::default(),
        };
        assert!(!entry.success);
        assert_eq!(entry.error.as_deref(), Some("Parse error"));
    }

    #[test]
    fn test_audit_completeness_threshold_pass() {
        // Create a key classification that passes 90% threshold
        let mut kc = KeyClassification::default();
        kc.implemented = 9;
        kc.partial = 0;
        kc.not_implemented = 1;
        kc.unknown = 0;

        // Gap score = (9 + 0*0.5) / (9+0+1+0) = 9/10 = 0.9
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.9).abs() < 0.001);
        assert!(gap >= 0.90); // Meets threshold
    }

    #[test]
    fn test_audit_completeness_threshold_fail() {
        // Create a key classification that fails 90% threshold
        let mut kc = KeyClassification::default();
        kc.implemented = 7;
        kc.partial = 2;
        kc.not_implemented = 1;
        kc.unknown = 0;

        // Gap score = (7 + 2*0.5) / (7+2+1+0) = 8/10 = 0.8
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.8).abs() < 0.001);
        assert!(gap < 0.90); // Fails threshold
    }

    #[test]
    fn test_missing_keys_frequency_counting() {
        // Simulate collecting missing keys from multiple specs
        let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();

        // Same key appearing in multiple specs should increase frequency
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "reverb".to_string())).or_insert(0) += 1;
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "reverb".to_string())).or_insert(0) += 1;
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "echo".to_string())).or_insert(0) += 1;

        assert_eq!(missing_keys.get(&("SOUND (audio_sfx)".to_string(), "reverb".to_string())), Some(&2));
        assert_eq!(missing_keys.get(&("SOUND (audio_sfx)".to_string(), "echo".to_string())), Some(&1));
    }

    #[test]
    fn test_missing_keys_sorted_by_frequency() {
        let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();
        missing_keys.insert(("A".to_string(), "key1".to_string()), 5);
        missing_keys.insert(("A".to_string(), "key2".to_string()), 10);
        missing_keys.insert(("A".to_string(), "key3".to_string()), 1);

        let mut sorted: Vec<_> = missing_keys.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        assert_eq!(sorted[0].0, &("A".to_string(), "key2".to_string()));
        assert_eq!(sorted[1].0, &("A".to_string(), "key1".to_string()));
        assert_eq!(sorted[2].0, &("A".to_string(), "key3".to_string()));
    }
}
