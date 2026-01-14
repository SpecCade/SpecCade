//! Macros and convenience functions for determinism testing.

/// Macro for easy determinism testing.
///
/// This macro generates a test function that runs a generation expression
/// multiple times and verifies byte-identical output. The expression must
/// return a type that implements `AsRef<[u8]>` (e.g., `Vec<u8>`, `&[u8]`).
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::test_determinism;
///
/// test_determinism!(audio_laser_blast, {
///     let spec = create_laser_spec();
///     generate_audio(&spec).wav_data
/// });
///
/// test_determinism!(texture_metal, runs = 5, {
///     let spec = create_metal_texture_spec();
///     generate_texture(&spec).png_data
/// });
/// ```
#[macro_export]
macro_rules! test_determinism {
    ($name:ident, $generate:expr) => {
        #[test]
        fn $name() {
            let mut results: Vec<Vec<u8>> = Vec::with_capacity(3);
            for _ in 0..3 {
                let output = $generate;
                let bytes: &[u8] = output.as_ref();
                results.push(bytes.to_vec());
            }

            let first = &results[0];
            for (i, result) in results.iter().enumerate().skip(1) {
                assert_eq!(
                    first.len(),
                    result.len(),
                    "Output size differs between run 0 and run {}: {} vs {} bytes",
                    i,
                    first.len(),
                    result.len()
                );

                for (offset, (&expected, &actual)) in first.iter().zip(result.iter()).enumerate() {
                    assert_eq!(
                        expected, actual,
                        "Non-deterministic output at byte {}: expected 0x{:02X}, got 0x{:02X} (run {} vs run 0)",
                        offset, expected, actual, i
                    );
                }
            }
        }
    };

    ($name:ident, runs = $runs:expr, $generate:expr) => {
        #[test]
        fn $name() {
            let run_count: usize = $runs;
            assert!(run_count >= 2, "Must run at least 2 times");

            let mut results: Vec<Vec<u8>> = Vec::with_capacity(run_count);
            for _ in 0..run_count {
                let output = $generate;
                let bytes: &[u8] = output.as_ref();
                results.push(bytes.to_vec());
            }

            let first = &results[0];
            for (i, result) in results.iter().enumerate().skip(1) {
                assert_eq!(
                    first.len(),
                    result.len(),
                    "Output size differs between run 0 and run {}: {} vs {} bytes",
                    i,
                    first.len(),
                    result.len()
                );

                for (offset, (&expected, &actual)) in first.iter().zip(result.iter()).enumerate() {
                    assert_eq!(
                        expected, actual,
                        "Non-deterministic output at byte {}: expected 0x{:02X}, got 0x{:02X} (run {} vs run 0)",
                        offset, expected, actual, i
                    );
                }
            }
        }
    };
}
