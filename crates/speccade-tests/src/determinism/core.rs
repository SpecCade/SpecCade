//! Core determinism verification types and functions.
//!
//! This module provides the fundamental building blocks for verifying that
//! asset generation produces byte-identical output across multiple runs.

use std::fmt;

/// Result of a determinism verification.
#[derive(Debug, Clone)]
pub struct DeterminismResult {
    /// Whether all runs produced identical output.
    pub is_deterministic: bool,
    /// Number of runs performed.
    pub runs: usize,
    /// Size of the output in bytes.
    pub output_size: usize,
    /// BLAKE3 hash of the output (all runs should match).
    pub hash: String,
    /// If non-deterministic, information about the first difference found.
    pub diff_info: Option<DiffInfo>,
}

/// Information about the first byte difference found between runs.
#[derive(Debug, Clone)]
pub struct DiffInfo {
    /// Byte offset where the difference was found.
    pub offset: usize,
    /// Value from the first run.
    pub expected: u8,
    /// Value from the differing run.
    pub actual: u8,
    /// Which run (0-indexed) produced the differing output.
    pub run_index: usize,
    /// Context bytes before and after the difference (for debugging).
    pub context: DiffContext,
}

/// Context around a byte difference for debugging.
#[derive(Debug, Clone)]
pub struct DiffContext {
    /// Bytes before the difference (up to 8 bytes).
    pub before: Vec<u8>,
    /// Bytes after the difference (up to 8 bytes).
    pub after: Vec<u8>,
}

impl fmt::Display for DiffInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Difference at byte {}: expected 0x{:02X}, got 0x{:02X} (run {})",
            self.offset, self.expected, self.actual, self.run_index
        )?;
        if !self.context.before.is_empty() || !self.context.after.is_empty() {
            write!(f, "\n  Context: ")?;
            for b in &self.context.before {
                write!(f, "{:02X} ", b)?;
            }
            write!(f, "[{:02X}] ", self.expected)?;
            for b in &self.context.after {
                write!(f, "{:02X} ", b)?;
            }
        }
        Ok(())
    }
}

impl DeterminismResult {
    /// Create a successful (deterministic) result.
    pub fn success(runs: usize, output_size: usize, hash: String) -> Self {
        Self {
            is_deterministic: true,
            runs,
            output_size,
            hash,
            diff_info: None,
        }
    }

    /// Create a failure (non-deterministic) result.
    pub fn failure(runs: usize, output_size: usize, hash: String, diff_info: DiffInfo) -> Self {
        Self {
            is_deterministic: false,
            runs,
            output_size,
            hash,
            diff_info: Some(diff_info),
        }
    }

    /// Panic with a detailed message if not deterministic.
    pub fn assert_deterministic(&self) {
        if !self.is_deterministic {
            let diff = self.diff_info.as_ref().unwrap();
            panic!(
                "Non-deterministic output detected!\n\
                 Runs: {}\n\
                 Output size: {} bytes\n\
                 Hash: {}\n\
                 {}",
                self.runs, self.output_size, self.hash, diff
            );
        }
    }
}

/// Run generation N times and verify all outputs are identical.
///
/// This is the core determinism verification function. It runs the provided
/// generation function multiple times and compares all outputs byte-by-byte.
///
/// # Arguments
///
/// * `generate_fn` - A function that generates output data
/// * `runs` - Number of times to run the generation (minimum 2)
///
/// # Returns
///
/// A `DeterminismResult` indicating whether outputs were identical and,
/// if not, details about the first difference found.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::verify_determinism;
///
/// let result = verify_determinism(|| {
///     generate_audio(&spec).wav_data
/// }, 3);
///
/// assert!(result.is_deterministic, "Audio generation must be deterministic");
/// ```
pub fn verify_determinism<F, O>(generate_fn: F, runs: usize) -> DeterminismResult
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    assert!(runs >= 2, "Must run at least 2 times to verify determinism");

    // Generate first output as reference
    let reference = generate_fn();
    let reference_bytes = reference.as_ref();
    let reference_hash = blake3::hash(reference_bytes).to_hex().to_string();

    // Run remaining times and compare
    for run_index in 1..runs {
        let output = generate_fn();
        let output_bytes = output.as_ref();

        // Check length first
        if output_bytes.len() != reference_bytes.len() {
            return DeterminismResult::failure(
                runs,
                reference_bytes.len(),
                reference_hash,
                DiffInfo {
                    offset: reference_bytes.len().min(output_bytes.len()),
                    expected: if reference_bytes.len() > output_bytes.len() {
                        reference_bytes[output_bytes.len()]
                    } else {
                        0
                    },
                    actual: if output_bytes.len() > reference_bytes.len() {
                        output_bytes[reference_bytes.len()]
                    } else {
                        0
                    },
                    run_index,
                    context: DiffContext {
                        before: Vec::new(),
                        after: Vec::new(),
                    },
                },
            );
        }

        // Compare byte-by-byte
        if let Some(diff) = find_first_difference(reference_bytes, output_bytes, run_index) {
            return DeterminismResult::failure(runs, reference_bytes.len(), reference_hash, diff);
        }
    }

    DeterminismResult::success(runs, reference_bytes.len(), reference_hash)
}

/// Find the first byte difference between two slices.
pub(crate) fn find_first_difference(
    expected: &[u8],
    actual: &[u8],
    run_index: usize,
) -> Option<DiffInfo> {
    for (offset, (&e, &a)) in expected.iter().zip(actual.iter()).enumerate() {
        if e != a {
            let context = extract_context(expected, offset);
            return Some(DiffInfo {
                offset,
                expected: e,
                actual: a,
                run_index,
                context,
            });
        }
    }
    None
}

/// Extract context bytes around a given offset.
pub(crate) fn extract_context(data: &[u8], offset: usize) -> DiffContext {
    let before_start = offset.saturating_sub(8);
    let after_end = (offset + 9).min(data.len());

    DiffContext {
        before: data[before_start..offset].to_vec(),
        after: if offset + 1 < data.len() {
            data[(offset + 1)..after_end].to_vec()
        } else {
            Vec::new()
        },
    }
}

/// Compare BLAKE3 hashes of multiple runs.
///
/// A quick check for determinism when you already have computed hashes
/// from multiple runs.
///
/// # Arguments
///
/// * `hashes` - Slice of BLAKE3 hash strings (64-char hex)
///
/// # Returns
///
/// `true` if all hashes are identical, `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::verify_hash_determinism;
///
/// let hashes = vec![
///     hash_from_run_1,
///     hash_from_run_2,
///     hash_from_run_3,
/// ];
/// assert!(verify_hash_determinism(&hashes));
/// ```
pub fn verify_hash_determinism(hashes: &[String]) -> bool {
    if hashes.is_empty() {
        return true;
    }
    let reference = &hashes[0];
    hashes.iter().all(|h| h == reference)
}

/// Compute BLAKE3 hash of data.
pub fn compute_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Helper to verify determinism of a closure returning `Vec<u8>`.
///
/// This is useful when you want to verify determinism without the macro,
/// or when you need to capture variables in a closure.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::assert_deterministic;
///
/// let spec = load_spec("laser.json");
/// assert_deterministic(3, || generate_audio(&spec).wav_data);
/// ```
pub fn assert_deterministic<F>(runs: usize, generate_fn: F)
where
    F: Fn() -> Vec<u8>,
{
    let result = verify_determinism(&generate_fn, runs);
    result.assert_deterministic();
}
