//! Unit tests for determinism framework.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    use crate::determinism::core::*;
    use crate::determinism::report::*;

    #[test]
    fn test_verify_determinism_identical() {
        let result = verify_determinism(|| vec![1u8, 2, 3, 4, 5], 5);
        assert!(result.is_deterministic);
        assert_eq!(result.runs, 5);
        assert_eq!(result.output_size, 5);
    }

    #[test]
    fn test_verify_determinism_with_counter() {
        let counter = AtomicU32::new(0);

        // This simulates non-deterministic behavior
        let result = verify_determinism(
            || {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                vec![n as u8, 2, 3]
            },
            3,
        );

        assert!(!result.is_deterministic);
        assert!(result.diff_info.is_some());
        let diff = result.diff_info.unwrap();
        assert_eq!(diff.offset, 0);
        assert_eq!(diff.expected, 0);
        assert_eq!(diff.actual, 1);
    }

    #[test]
    fn test_verify_hash_determinism_same() {
        let hashes = vec![
            "abc123".to_string(),
            "abc123".to_string(),
            "abc123".to_string(),
        ];
        assert!(verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_verify_hash_determinism_different() {
        let hashes = vec![
            "abc123".to_string(),
            "abc123".to_string(),
            "def456".to_string(),
        ];
        assert!(!verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_verify_hash_determinism_empty() {
        let hashes: Vec<String> = vec![];
        assert!(verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"hello world");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_diff_context_extraction() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let context = extract_context(&data, 8);

        assert_eq!(context.before, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(context.after, vec![9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_diff_context_at_start() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let context = extract_context(&data, 0);

        assert!(context.before.is_empty());
        assert_eq!(context.after, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_diff_context_at_end() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let context = extract_context(&data, 5);

        assert_eq!(context.before, vec![0, 1, 2, 3, 4]);
        assert!(context.after.is_empty());
    }

    #[test]
    fn test_determinism_result_display() {
        let diff = DiffInfo {
            offset: 100,
            expected: 0xAB,
            actual: 0xCD,
            run_index: 2,
            context: DiffContext {
                before: vec![1, 2, 3],
                after: vec![4, 5, 6],
            },
        };

        let display = format!("{}", diff);
        assert!(display.contains("byte 100"));
        assert!(display.contains("0xAB"));
        assert!(display.contains("0xCD"));
        assert!(display.contains("run 2"));
    }

    #[test]
    fn test_fixture_builder() {
        use crate::determinism::fixture::DeterminismFixture;

        let fixture = DeterminismFixture::new()
            .add_spec("a.json")
            .add_spec("b.json")
            .runs(5);

        assert_eq!(fixture.specs.len(), 2);
        assert_eq!(fixture.runs, 5);
    }

    #[test]
    fn test_report_counting() {
        let mut report = DeterminismReport::new();

        report.add_entry(DeterminismReportEntry {
            spec_path: PathBuf::from("a.json"),
            result: Ok(DeterminismResult::success(3, 100, "hash1".to_string())),
        });

        report.add_entry(DeterminismReportEntry {
            spec_path: PathBuf::from("b.json"),
            result: Err(DeterminismError::SpecNotFound),
        });

        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert_eq!(report.total_count(), 2);
        assert!(!report.all_deterministic());
    }

    // Test macro usage
    use crate::test_determinism;

    test_determinism!(macro_test_constant, { vec![1u8, 2, 3, 4, 5] });

    test_determinism!(macro_test_with_runs, runs = 4, { vec![0u8; 100] });
}
