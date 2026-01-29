//! Golden hash verification tests for Tier-1 backends.
//!
//! This test suite verifies that Tier-1 backends (audio, texture, music) produce
//! byte-identical output by comparing BLAKE3 hashes of generated assets against
//! expected hash files checked into the repository.
//!
//! ## Hash File Format
//!
//! Each `.hash` file contains a single 64-character lowercase hexadecimal BLAKE3 hash.
//! The file name matches the spec name: `<spec_name>.hash`.
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all hash verification tests
//! cargo test -p speccade-tests --test golden_hash_verification
//!
//! # Update expected hashes (use with caution!)
//! SPECCADE_UPDATE_GOLDEN_HASHES=1 cargo test -p speccade-tests --test golden_hash_verification
//! ```
//!
//! ## Adding New Specs
//!
//! 1. Add the spec Starlark file to `specs/<type>/`
//! 2. Run with `SPECCADE_UPDATE_GOLDEN_HASHES=1` to generate the hash file
//! 3. Commit both the spec and the hash file

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use speccade_spec::{OutputKind, Spec};
use speccade_tests::fixtures::GoldenFixtures;

/// Whether to update expected hashes instead of comparing.
fn should_update_hashes() -> bool {
    std::env::var("SPECCADE_UPDATE_GOLDEN_HASHES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Compute BLAKE3 hash of data and return as hex string.
fn compute_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Read expected hash from a .hash file.
fn read_expected_hash(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Write expected hash to a .hash file.
fn write_expected_hash(path: &Path, hash: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create hash directory");
    }
    fs::write(path, format!("{}\n", hash)).expect("Failed to write hash file");
}

/// Parse a spec JSON file.
fn parse_spec_file(path: &Path) -> Result<Spec, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read spec file: {}", e))?;
    Spec::from_json(&content).map_err(|e| format!("Failed to parse spec: {}", e))
}

/// Get spec name from path (without extension).
fn spec_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Remove .spec suffix if present (for files like foo.spec.json)
            s.strip_suffix(".spec").unwrap_or(s).to_string()
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Generate audio and return the WAV data hash.
fn generate_audio_hash(spec: &Spec) -> Result<String, String> {
    let result = speccade_backend_audio::generate(spec)
        .map_err(|e| format!("Audio generation failed: {}", e))?;
    Ok(compute_hash(&result.wav.wav_data))
}

/// Generate texture and return the combined hash of all primary PNG outputs.
///
/// For textures with multiple outputs (albedo, roughness, normal, etc.),
/// we concatenate all PNG data in output order and hash the result.
fn generate_texture_hash(spec: &Spec) -> Result<String, String> {
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| "Spec has no recipe".to_string())?;

    if recipe.kind != "texture.procedural_v1" {
        return Err(format!("Unsupported texture recipe kind: {}", recipe.kind));
    }

    let params = recipe
        .as_texture_procedural()
        .map_err(|e| format!("Failed to parse texture params: {}", e))?;

    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed)
        .map_err(|e| format!("Texture generation failed: {}", e))?;

    // Collect PNG data for all primary outputs in order
    let mut combined_data = Vec::new();

    for output in spec
        .outputs
        .iter()
        .filter(|o| o.kind == OutputKind::Primary)
    {
        let source = output
            .source
            .as_ref()
            .ok_or_else(|| format!("Output {} missing source", output.path))?;

        let value = nodes
            .get(source)
            .ok_or_else(|| format!("Node '{}' not found in graph", source))?;

        let (png_bytes, _) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| format!("Failed to encode PNG for '{}': {}", source, e))?;

        combined_data.extend_from_slice(&png_bytes);
    }

    if combined_data.is_empty() {
        return Err("No primary outputs generated".to_string());
    }

    Ok(compute_hash(&combined_data))
}

/// Generate music and return the module data hash.
fn generate_music_hash(spec: &Spec) -> Result<String, String> {
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| "Spec has no recipe".to_string())?;

    if recipe.kind != "music.tracker_song_v1" {
        return Err(format!("Unsupported music recipe kind: {}", recipe.kind));
    }

    let params = recipe
        .as_music_tracker_song()
        .map_err(|e| format!("Failed to parse music params: {}", e))?;

    // Use a temp directory for any intermediate files the music backend might need
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let result = speccade_backend_music::generate_music(&params, spec.seed, temp_dir.path())
        .map_err(|e| format!("Music generation failed: {}", e))?;

    Ok(compute_hash(&result.data))
}

/// Result of verifying a single spec.
#[derive(Debug)]
struct VerificationResult {
    spec_path: PathBuf,
    spec_name: String,
    expected_hash: Option<String>,
    actual_hash: Result<String, String>,
    hash_file_path: PathBuf,
}

impl VerificationResult {
    fn is_pass(&self) -> bool {
        match (&self.expected_hash, &self.actual_hash) {
            (Some(expected), Ok(actual)) => expected == actual,
            _ => false,
        }
    }

    fn is_missing_expected(&self) -> bool {
        self.expected_hash.is_none() && self.actual_hash.is_ok()
    }

    fn is_generation_error(&self) -> bool {
        self.actual_hash.is_err()
    }

    /// Check if this is an expected skip (no hash file AND generation error).
    /// This handles specs that have external references that cannot be resolved
    /// in the test context.
    fn is_expected_skip(&self) -> bool {
        self.expected_hash.is_none() && self.actual_hash.is_err()
    }
}

/// Verify all golden specs of a given type.
fn verify_specs(
    asset_type: &str,
    generate_fn: fn(&Spec) -> Result<String, String>,
) -> Vec<VerificationResult> {
    let specs = GoldenFixtures::list_speccade_specs(asset_type);
    let hashes_dir = GoldenFixtures::expected_hashes_dir().join(asset_type);

    let update_mode = should_update_hashes();

    let mut results = Vec::new();

    for spec_path in specs {
        let name = spec_name(&spec_path);
        let hash_file = hashes_dir.join(format!("{}.hash", name));

        let spec_result = parse_spec_file(&spec_path);
        let actual_hash = spec_result.and_then(|spec| generate_fn(&spec));

        let expected_hash = if update_mode {
            // In update mode, write the new hash if generation succeeded
            if let Ok(ref hash) = actual_hash {
                write_expected_hash(&hash_file, hash);
                println!("Updated hash for {}: {}", name, hash);
            }
            actual_hash.clone().ok()
        } else {
            read_expected_hash(&hash_file)
        };

        results.push(VerificationResult {
            spec_path,
            spec_name: name,
            expected_hash,
            actual_hash,
            hash_file_path: hash_file,
        });
    }

    results
}

/// Assert all verification results pass, with detailed failure messages.
fn assert_verification_results(asset_type: &str, results: &[VerificationResult]) {
    let update_mode = should_update_hashes();

    let mut failures = Vec::new();
    let mut missing = Vec::new();
    let mut errors = Vec::new();
    let mut skipped = Vec::new();
    let mut passed = 0;

    for result in results {
        if result.is_pass() {
            passed += 1;
        } else if result.is_expected_skip() {
            // No hash file AND generation error - this is an expected skip
            // (e.g., specs with external references that can't be resolved)
            skipped.push(result);
        } else if result.is_generation_error() {
            // Has a hash file but generation failed - this is an error
            errors.push(result);
        } else if result.is_missing_expected() {
            missing.push(result);
        } else {
            failures.push(result);
        }
    }

    println!(
        "\n{} verification: {} passed, {} failed, {} missing, {} skipped, {} errors",
        asset_type,
        passed,
        failures.len(),
        missing.len(),
        skipped.len(),
        errors.len()
    );

    // In update mode, we only fail on generation errors (excluding expected skips)
    if update_mode {
        if !errors.is_empty() {
            let error_msgs: Vec<String> = errors
                .iter()
                .map(|r| {
                    format!(
                        "  {} ({}): {}",
                        r.spec_name,
                        r.spec_path.display(),
                        r.actual_hash.as_ref().unwrap_err()
                    )
                })
                .collect();

            panic!(
                "Generation errors during hash update:\n{}",
                error_msgs.join("\n")
            );
        }
        return;
    }

    // Build failure message
    let mut failure_msg = String::new();

    if !failures.is_empty() {
        failure_msg.push_str(&format!("\nHash mismatches ({}):\n", failures.len()));
        for result in &failures {
            failure_msg.push_str(&format!(
                "  {} ({}):\n    expected: {}\n    actual:   {}\n",
                result.spec_name,
                result.spec_path.display(),
                result.expected_hash.as_ref().unwrap(),
                result.actual_hash.as_ref().unwrap()
            ));
        }
    }

    if !missing.is_empty() {
        failure_msg.push_str(&format!(
            "\nMissing expected hash files ({}):\n",
            missing.len()
        ));
        for result in &missing {
            failure_msg.push_str(&format!(
                "  {} ({})\n    hash file: {}\n    actual hash: {}\n",
                result.spec_name,
                result.spec_path.display(),
                result.hash_file_path.display(),
                result.actual_hash.as_ref().unwrap()
            ));
        }
        failure_msg.push_str(
            "\nRun with SPECCADE_UPDATE_GOLDEN_HASHES=1 to generate missing hash files.\n",
        );
    }

    if !errors.is_empty() {
        failure_msg.push_str(&format!(
            "\nGeneration errors for specs with expected hashes ({}):\n",
            errors.len()
        ));
        for result in &errors {
            failure_msg.push_str(&format!(
                "  {} ({}): {}\n",
                result.spec_name,
                result.spec_path.display(),
                result.actual_hash.as_ref().unwrap_err()
            ));
        }
    }

    if !failure_msg.is_empty() {
        panic!("{} hash verification failed:{}", asset_type, failure_msg);
    }

    // Print skipped specs for informational purposes (not a failure)
    if !skipped.is_empty() {
        println!(
            "  Skipped {} specs (no hash file, expected):",
            skipped.len()
        );
        for result in &skipped {
            println!("    - {}", result.spec_name);
        }
    }
}

// ============================================================================
// Audio Hash Verification Tests
// ============================================================================

#[test]
fn test_golden_audio_hashes() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let results = verify_specs("audio", generate_audio_hash);
    assert_verification_results("Audio", &results);
}

// ============================================================================
// Texture Hash Verification Tests
// ============================================================================

#[test]
fn test_golden_texture_hashes() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let results = verify_specs("texture", generate_texture_hash);
    assert_verification_results("Texture", &results);
}

// ============================================================================
// Music Hash Verification Tests
// ============================================================================

#[test]
fn test_golden_music_hashes() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let results = verify_specs("music", generate_music_hash);
    assert_verification_results("Music", &results);
}

// ============================================================================
// Combined Report Test
// ============================================================================

/// Run all hash verifications and produce a combined report.
/// This is useful for CI to get a complete picture in one test.
#[test]
fn test_golden_hashes_all() {
    if !GoldenFixtures::exists() {
        println!("Golden fixtures not found, skipping test");
        return;
    }

    let mut all_results: HashMap<&str, Vec<VerificationResult>> = HashMap::new();

    all_results.insert("audio", verify_specs("audio", generate_audio_hash));
    all_results.insert("texture", verify_specs("texture", generate_texture_hash));
    all_results.insert("music", verify_specs("music", generate_music_hash));

    // Print summary
    println!("\n=== Golden Hash Verification Summary ===");
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_missing = 0;
    let mut total_skipped = 0;
    let mut total_errors = 0;

    for (asset_type, results) in &all_results {
        let passed = results.iter().filter(|r| r.is_pass()).count();
        let skipped = results.iter().filter(|r| r.is_expected_skip()).count();
        let failed = results
            .iter()
            .filter(|r| {
                !r.is_pass()
                    && !r.is_missing_expected()
                    && !r.is_generation_error()
                    && !r.is_expected_skip()
            })
            .count();
        let missing = results.iter().filter(|r| r.is_missing_expected()).count();
        let errors = results
            .iter()
            .filter(|r| r.is_generation_error() && !r.is_expected_skip())
            .count();

        println!(
            "  {}: {} passed, {} failed, {} missing, {} skipped, {} errors",
            asset_type, passed, failed, missing, skipped, errors
        );

        total_passed += passed;
        total_failed += failed;
        total_missing += missing;
        total_skipped += skipped;
        total_errors += errors;
    }

    println!("  ----------------------------------------");
    println!(
        "  Total: {} passed, {} failed, {} missing, {} skipped, {} errors",
        total_passed, total_failed, total_missing, total_skipped, total_errors
    );

    // Assert each type separately for clear error messages
    for (asset_type, results) in all_results {
        assert_verification_results(asset_type, &results);
    }
}
