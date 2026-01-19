//! Manifest types for batch SFX variation generation.

use serde::{Deserialize, Serialize};

/// Manifest for batch SFX variation generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationsManifest {
    /// Spec ID from the source spec
    pub spec_id: String,
    /// Total number of variations attempted
    pub total: u32,
    /// Number of variations that passed quality constraints
    pub passed: u32,
    /// Number of variations that failed quality constraints
    pub failed: u32,
    /// Base seed used for the first variation
    pub base_seed: u32,
    /// Quality constraints applied (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<VariationConstraints>,
    /// Individual variation results
    pub variations: Vec<VariationEntry>,
}

/// Quality constraints for variation generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationConstraints {
    /// Maximum peak level in dB (None = no constraint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_peak_db: Option<f64>,
    /// Maximum DC offset (None = no constraint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_dc_offset: Option<f64>,
}

/// Result for a single variation in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationEntry {
    /// Variation index (0-based)
    pub index: u32,
    /// Seed used for this variation
    pub seed: u32,
    /// Output file path (None if failed/rejected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Whether this variation passed quality constraints
    pub passed: bool,
    /// Failure reason (if passed is false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// BLAKE3 hash of the output file (if generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Peak level in dB (if measured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_db: Option<f64>,
    /// DC offset (if measured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_offset: Option<f64>,
}

impl VariationsManifest {
    /// Creates a new variations manifest.
    pub fn new(spec_id: String, base_seed: u32, constraints: Option<VariationConstraints>) -> Self {
        Self {
            spec_id,
            total: 0,
            passed: 0,
            failed: 0,
            base_seed,
            constraints,
            variations: Vec::new(),
        }
    }

    /// Adds a variation entry and updates counts.
    pub fn add_variation(&mut self, entry: VariationEntry) {
        self.total += 1;
        if entry.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.variations.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variations_manifest_serialization() {
        let constraints = VariationConstraints {
            max_peak_db: Some(-3.0),
            max_dc_offset: Some(0.01),
        };

        let mut manifest = VariationsManifest::new("test-asset".to_string(), 42, Some(constraints));

        manifest.add_variation(VariationEntry {
            index: 0,
            seed: 42,
            path: Some("test_var_0.wav".to_string()),
            passed: true,
            reason: None,
            hash: Some("abc123".to_string()),
            peak_db: Some(-6.5),
            dc_offset: Some(0.001),
        });

        manifest.add_variation(VariationEntry {
            index: 1,
            seed: 43,
            path: None,
            passed: false,
            reason: Some("peak exceeded".to_string()),
            hash: None,
            peak_db: Some(-1.0),
            dc_offset: Some(0.002),
        });

        let json = serde_json::to_string_pretty(&manifest).unwrap();

        assert!(json.contains("\"spec_id\": \"test-asset\""));
        assert!(json.contains("\"total\": 2"));
        assert!(json.contains("\"passed\": 1"));
        assert!(json.contains("\"failed\": 1"));
        assert!(json.contains("\"base_seed\": 42"));
        assert!(json.contains("\"max_peak_db\": -3.0"));
        assert!(json.contains("\"path\": \"test_var_0.wav\""));
        assert!(json.contains("\"reason\": \"peak exceeded\""));

        // Verify roundtrip
        let parsed: VariationsManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total, 2);
        assert_eq!(parsed.passed, 1);
        assert_eq!(parsed.failed, 1);
        assert_eq!(parsed.variations.len(), 2);
    }

    #[test]
    fn test_variations_manifest_skips_null_fields() {
        let manifest = VariationsManifest::new("test-asset".to_string(), 42, None);

        let json = serde_json::to_string(&manifest).unwrap();

        // constraints should be omitted when None
        assert!(!json.contains("constraints"));
    }
}
