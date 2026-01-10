//! Parity data generated from `docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md` at build time.
//!
//! This module includes the auto-generated parity data and re-exports
//! all types and helpers for use elsewhere in the crate.
//!
//! # Example
//!
//! ```ignore
//! use speccade_cli::parity_data::{find, KeyStatus, ALL_KEYS};
//!
//! // Find a specific key
//! if let Some(info) = find("SOUND (audio_sfx)", "Top-Level Keys", "name") {
//!     println!("Key 'name' is required: {}", info.required);
//!     println!("Implementation status: {:?}", info.status);
//! }
//!
//! // Iterate over all keys
//! for key_info in ALL_KEYS {
//!     if key_info.status == KeyStatus::NotImplemented {
//!         println!("Not implemented: {}", key_info.key.key);
//!     }
//! }
//! ```

// Include the generated code from build.rs
include!(concat!(env!("OUT_DIR"), "/parity_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_keys_not_empty() {
        // The PARITY_MATRIX.md file should produce at least some keys
        assert!(!ALL_KEYS.is_empty(), "ALL_KEYS should not be empty");
    }

    #[test]
    fn test_find_existing_key() {
        // Check that we can find a known key from the PARITY_MATRIX.md
        // The "name" key under SOUND should exist
        let result = find("SOUND (audio_sfx)", "Top-Level Keys", "name");
        assert!(result.is_some(), "Should find 'name' key in SOUND section");

        let info = result.unwrap();
        assert!(info.required, "'name' should be required");
        assert_eq!(
            info.status,
            KeyStatus::Implemented,
            "'name' should be implemented"
        );
    }

    #[test]
    fn test_find_missing_key() {
        let result = find("NonExistent", "Table", "key");
        assert!(result.is_none(), "Should not find nonexistent key");
    }

    #[test]
    fn test_key_status_values() {
        // Ensure all status variants exist in the data (or at least compile)
        let statuses = [
            KeyStatus::Implemented,
            KeyStatus::Partial,
            KeyStatus::NotImplemented,
            KeyStatus::Deprecated,
        ];

        for status in &statuses {
            // Just check that the enum values work
            let _ = format!("{:?}", status);
        }
    }

    #[test]
    fn test_has_multiple_sections() {
        // The matrix should have keys from multiple sections
        let sections: std::collections::HashSet<_> =
            ALL_KEYS.iter().map(|k| k.key.section).collect();

        assert!(
            sections.len() > 1,
            "Should have keys from multiple sections, found: {:?}",
            sections
        );
    }

    #[test]
    fn test_has_required_keys() {
        // There should be some required keys
        let required_count = ALL_KEYS.iter().filter(|k| k.required).count();
        assert!(required_count > 0, "Should have some required keys");
    }

    #[test]
    fn test_has_various_statuses() {
        // Check that we have keys with different implementation statuses
        let implemented = ALL_KEYS
            .iter()
            .filter(|k| k.status == KeyStatus::Implemented)
            .count();
        let partial = ALL_KEYS
            .iter()
            .filter(|k| k.status == KeyStatus::Partial)
            .count();
        let not_impl = ALL_KEYS
            .iter()
            .filter(|k| k.status == KeyStatus::NotImplemented)
            .count();

        assert!(implemented > 0, "Should have implemented keys");
        // Partial and not implemented might not always be present, so just check implemented
        println!(
            "Status counts - Implemented: {}, Partial: {}, NotImplemented: {}",
            implemented, partial, not_impl
        );
    }
}
