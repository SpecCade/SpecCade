# Phase 3 Test Plan: Budgets, Caching, and Hardening

**Date**: 2026-01-17
**Phase**: Phase 3 - Budgets, Caching, and Hardening

---

## Overview

This test plan covers all testable requirements for Phase 3:

1. **Budget Enforcement Tests** - Verify budgets are enforced at validation stage
2. **Provenance Tests** - Verify report provenance fields are populated
3. **Caching Tests** - Verify cache key computation and storage (if implemented)
4. **Canonicalization Tests** - Verify idempotence and edge cases

---

## 1. Budget Enforcement Tests

### Location
- `crates/speccade-spec/src/validation/tests.rs` (unit tests)
- `crates/speccade-tests/tests/budget_enforcement.rs` (integration tests)

### 1.1 Audio Budget Tests

```rust
#[test]
fn test_audio_budget_rejects_excessive_duration() {
    let spec = Spec::builder("test-audio-budget-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 60.0,  // Exceeds MAX_AUDIO_DURATION_SECONDS (30.0)
                "sample_rate": 44100,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e|
        e.message.contains("duration_seconds must be <=")
    ));
}

#[test]
fn test_audio_budget_accepts_max_duration() {
    let spec = Spec::builder("test-audio-budget-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 30.0,  // Exactly at limit
                "sample_rate": 44100,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    // Should not fail on duration (may fail on missing layers)
    assert!(!result.errors.iter().any(|e|
        e.message.contains("duration_seconds must be <=")
    ));
}

#[test]
fn test_audio_budget_rejects_excessive_layers() {
    let layers: Vec<_> = (0..33).map(|i| {
        serde_json::json!({
            "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 + i as f64 },
            "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
            "volume": 0.1,
            "pan": 0.0
        })
    }).collect();

    let spec = Spec::builder("test-audio-budget-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "sample_rate": 44100,
                "layers": layers  // 33 layers, exceeds MAX_AUDIO_LAYERS (32)
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e|
        e.message.contains("layers must have at most")
    ));
}

#[test]
fn test_audio_budget_rejects_invalid_sample_rate() {
    let spec = Spec::builder("test-audio-budget-04", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.1,
                "sample_rate": 96000,  // Not in allowed list
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e|
        e.message.contains("sample_rate must be")
    ));
}
```

### 1.2 Music Budget Tests

```rust
#[test]
fn test_music_budget_rejects_excessive_xm_channels() {
    let spec = Spec::builder("test-music-budget-01", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 64  // Exceeds XM_MAX_CHANNELS (32)
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    // Check for channel limit error
}

#[test]
fn test_music_budget_accepts_max_xm_channels() {
    let spec = Spec::builder("test-music-budget-02", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Xm, "songs/test.xm"))
        .recipe(Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "xm",
                "bpm": 120,
                "speed": 6,
                "channels": 32  // Exactly at XM limit
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    // Should not fail on channel count
    assert!(!result.errors.iter().any(|e|
        e.message.contains("channels")
    ));
}

#[test]
fn test_music_budget_allows_more_it_channels() {
    let spec = Spec::builder("test-music-budget-03", AssetType::Music)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::It, "songs/test.it"))
        .recipe(Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({
                "format": "it",
                "bpm": 120,
                "speed": 6,
                "channels": 48  // Within IT_MAX_CHANNELS (64) but exceeds XM
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    // Should not fail on channel count for IT format
    assert!(!result.errors.iter().any(|e|
        e.message.contains("channels")
    ));
}
```

### 1.3 Texture Budget Tests

```rust
#[test]
fn test_texture_budget_rejects_excessive_resolution() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/test.png");
    output.source = Some("result".to_string());

    let spec = Spec::builder("test-texture-budget-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [8192, 8192],  // Exceeds MAX_DIMENSION (4096)
                "tileable": true,
                "nodes": [
                    { "id": "result", "type": "solid_color", "color": "#ffffff" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e|
        e.message.contains("resolution")
    ));
}

#[test]
fn test_texture_budget_accepts_max_resolution() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/test.png");
    output.source = Some("result".to_string());

    let spec = Spec::builder("test-texture-budget-02", AssetType::Texture)
        .license("CC0-1.0")
        .seed(42)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [4096, 4096],  // Exactly at limit
                "tileable": true,
                "nodes": [
                    { "id": "result", "type": "solid_color", "color": "#ffffff" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    // Should not fail on resolution
    assert!(!result.errors.iter().any(|e|
        e.message.contains("resolution") && e.message.contains("too large")
    ));
}
```

### 1.4 Budget Profile Tests

```rust
#[test]
fn test_budget_profile_default() {
    let profile = BudgetProfile::default();
    assert_eq!(profile.name, "default");
    assert_eq!(profile.audio.max_duration_seconds, 30.0);
    assert_eq!(profile.audio.max_layers, 32);
    assert_eq!(profile.texture.max_dimension, 4096);
}

#[test]
fn test_budget_profile_strict() {
    let profile = BudgetProfile::strict();
    assert_eq!(profile.name, "strict");
    assert_eq!(profile.audio.max_duration_seconds, 10.0);
    assert_eq!(profile.audio.max_layers, 16);
}

#[test]
fn test_budget_profile_zx_8bit() {
    let profile = BudgetProfile::zx_8bit();
    assert_eq!(profile.name, "zx-8bit");
    assert_eq!(profile.texture.max_dimension, 256);
    assert_eq!(profile.music.xm_max_channels, 8);
}

#[test]
fn test_budget_profile_by_name() {
    assert!(BudgetProfile::by_name("default").is_some());
    assert!(BudgetProfile::by_name("strict").is_some());
    assert!(BudgetProfile::by_name("zx-8bit").is_some());
    assert!(BudgetProfile::by_name("nonexistent").is_none());
}

#[test]
fn test_validate_with_strict_profile_rejects_default_limits() {
    // Spec that passes default profile but fails strict
    let spec = Spec::builder("test-strict-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 15.0,  // OK for default, exceeds strict (10.0)
                "sample_rate": 44100,
                "layers": []
            }),
        ))
        .build();

    let default_result = validate_for_generate_with_budget(&spec, &BudgetProfile::default());
    let strict_result = validate_for_generate_with_budget(&spec, &BudgetProfile::strict());

    // Default should pass duration check
    assert!(!default_result.errors.iter().any(|e|
        e.message.contains("duration_seconds must be <=")
    ));

    // Strict should fail duration check
    assert!(strict_result.errors.iter().any(|e|
        e.message.contains("duration_seconds must be <=")
    ));
}
```

---

## 2. Provenance Tests

### Location
- `crates/speccade-cli/src/commands/generate.rs` (existing test module)
- `crates/speccade-tests/tests/provenance.rs` (new integration tests)

### 2.1 JSON Source Provenance

```rust
#[test]
fn test_report_contains_json_provenance() {
    let tmp = tempfile::tempdir().unwrap();

    let spec = make_valid_audio_spec();
    let spec_path = write_spec(&tmp, "spec.json", &spec);

    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
    ).unwrap();

    assert_eq!(code, ExitCode::SUCCESS);

    let report_path = reporting::report_path(spec_path.to_str().unwrap(), &spec.asset_id);
    let report: Report = read_report(&report_path);

    // Verify provenance fields
    assert_eq!(report.source_kind.as_deref(), Some("json"));
    assert!(report.source_hash.is_some());
    assert_eq!(report.source_hash.as_ref().unwrap().len(), 64); // BLAKE3 hex

    // JSON sources should not have stdlib_version
    assert!(report.stdlib_version.is_none());

    // Recipe hash should be present
    assert!(report.recipe_hash.is_some());
}
```

### 2.2 Starlark Source Provenance

```rust
#[test]
#[cfg(feature = "starlark")]
fn test_report_contains_starlark_provenance() {
    let tmp = tempfile::tempdir().unwrap();

    let star_source = r#"
{
    "spec_version": 1,
    "asset_id": "starlark-prov-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [{"kind": "primary", "format": "wav", "path": "test.wav"}],
    "recipe": {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.1,
            "sample_rate": 22050,
            "layers": [{
                "synthesis": {"type": "oscillator", "waveform": "sine", "frequency": 440.0},
                "envelope": {"attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0},
                "volume": 0.8,
                "pan": 0.0
            }]
        }
    }
}
"#;

    let spec_path = tmp.path().join("spec.star");
    std::fs::write(&spec_path, star_source).unwrap();

    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        false,
    ).unwrap();

    assert_eq!(code, ExitCode::SUCCESS);

    let report_path = reporting::report_path(spec_path.to_str().unwrap(), "starlark-prov-01");
    let report: Report = read_report(&report_path);

    // Verify Starlark-specific provenance
    assert_eq!(report.source_kind.as_deref(), Some("starlark"));
    assert!(report.source_hash.is_some());
    assert_eq!(report.source_hash.as_ref().unwrap().len(), 64);

    // Starlark sources should have stdlib_version
    assert!(report.stdlib_version.is_some());
    assert!(!report.stdlib_version.as_ref().unwrap().is_empty());

    // Recipe hash should be present
    assert!(report.recipe_hash.is_some());
}
```

### 2.3 Provenance Consistency

```rust
#[test]
fn test_same_spec_produces_same_provenance() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = make_valid_audio_spec();
    let spec_path = write_spec(&tmp, "spec.json", &spec);

    // Run twice
    let out1 = tmp.path().join("run1");
    let out2 = tmp.path().join("run2");
    std::fs::create_dir_all(&out1).unwrap();
    std::fs::create_dir_all(&out2).unwrap();

    run(spec_path.to_str().unwrap(), Some(out1.to_str().unwrap()), false).unwrap();
    run(spec_path.to_str().unwrap(), Some(out2.to_str().unwrap()), false).unwrap();

    // Compare reports
    let report1 = read_report_from_dir(&out1, &spec.asset_id);
    let report2 = read_report_from_dir(&out2, &spec.asset_id);

    assert_eq!(report1.spec_hash, report2.spec_hash);
    assert_eq!(report1.source_hash, report2.source_hash);
    assert_eq!(report1.recipe_hash, report2.recipe_hash);
}
```

---

## 3. Caching Tests (Optional)

### Location
- `crates/speccade-cli/src/cache/mod.rs` (unit tests)
- `crates/speccade-tests/tests/caching.rs` (integration tests)

### 3.1 Cache Key Computation

```rust
#[test]
fn test_cache_key_deterministic() {
    let components = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: Some("0.1.0".to_string()),
        budget_profile: "default".to_string(),
    };

    let key1 = CacheKey::new(components.clone());
    let key2 = CacheKey::new(components);

    assert_eq!(key1.full, key2.full);
}

#[test]
fn test_cache_key_different_recipe_hash() {
    let components1 = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: None,
        budget_profile: "default".to_string(),
    };

    let components2 = CacheKeyComponents {
        recipe_hash: "def456".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: None,
        budget_profile: "default".to_string(),
    };

    let key1 = CacheKey::new(components1);
    let key2 = CacheKey::new(components2);

    assert_ne!(key1.full, key2.full);
}

#[test]
fn test_cache_key_different_stdlib_version() {
    let components1 = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: Some("0.1.0".to_string()),
        budget_profile: "default".to_string(),
    };

    let components2 = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: Some("0.2.0".to_string()),
        budget_profile: "default".to_string(),
    };

    let key1 = CacheKey::new(components1);
    let key2 = CacheKey::new(components2);

    assert_ne!(key1.full, key2.full);
}

#[test]
fn test_cache_key_different_budget_profile() {
    let components1 = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: None,
        budget_profile: "default".to_string(),
    };

    let components2 = CacheKeyComponents {
        recipe_hash: "abc123".to_string(),
        backend_version: "v0.1.0".to_string(),
        stdlib_version: None,
        budget_profile: "strict".to_string(),
    };

    let key1 = CacheKey::new(components1);
    let key2 = CacheKey::new(components2);

    assert_ne!(key1.full, key2.full);
}
```

### 3.2 Cache Storage

```rust
#[test]
fn test_cache_storage_disabled_returns_none() {
    let storage = CacheStorage::disabled();
    let key = make_test_cache_key();

    assert!(storage.lookup(&key).is_none());
}

#[test]
fn test_cache_storage_store_and_lookup() {
    let tmp = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        root: tmp.path().to_path_buf(),
        enabled: true,
    };
    let storage = CacheStorage::new(config);

    let key = make_test_cache_key();
    let report = make_test_report();
    let artifacts = vec![tmp.path().join("test.wav")];
    std::fs::write(&artifacts[0], b"test data").unwrap();

    storage.store(&key, &report, &artifacts, None).unwrap();

    let cached = storage.lookup(&key);
    assert!(cached.is_some());

    let cached = cached.unwrap();
    assert_eq!(cached.report.spec_hash, report.spec_hash);
}

#[test]
fn test_cache_storage_invalidate() {
    let tmp = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        root: tmp.path().to_path_buf(),
        enabled: true,
    };
    let storage = CacheStorage::new(config);

    let key = make_test_cache_key();
    let report = make_test_report();

    storage.store(&key, &report, &[], None).unwrap();
    assert!(storage.lookup(&key).is_some());

    storage.invalidate(&key).unwrap();
    assert!(storage.lookup(&key).is_none());
}
```

### 3.3 Cache Integration

```rust
#[test]
fn test_generate_uses_cache_on_second_run() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_dir = tmp.path().join("cache");

    let spec = make_valid_audio_spec();
    let spec_path = write_spec(&tmp, "spec.json", &spec);

    // First run - should generate
    let start1 = std::time::Instant::now();
    run_with_cache(spec_path.to_str().unwrap(), &cache_dir, false).unwrap();
    let duration1 = start1.elapsed();

    // Second run - should use cache (faster)
    let start2 = std::time::Instant::now();
    run_with_cache(spec_path.to_str().unwrap(), &cache_dir, false).unwrap();
    let duration2 = start2.elapsed();

    // Cache hit should be significantly faster
    assert!(duration2 < duration1 / 2);
}

#[test]
fn test_no_cache_flag_bypasses_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_dir = tmp.path().join("cache");

    let spec = make_valid_audio_spec();
    let spec_path = write_spec(&tmp, "spec.json", &spec);

    // First run with caching
    run_with_cache(spec_path.to_str().unwrap(), &cache_dir, false).unwrap();

    // Second run with --no-cache should not use cache
    // (We can verify by checking generation duration or cache miss logs)
    run_with_cache(spec_path.to_str().unwrap(), &cache_dir, true).unwrap();
}

#[test]
fn test_dirty_builds_not_cached() {
    let report = Report::builder("hash".to_string(), "v1".to_string())
        .git_metadata("abc123", true)  // git_dirty = true
        .ok(true)
        .build();

    assert!(!should_cache(&report));
}

#[test]
fn test_failed_generations_not_cached() {
    let report = Report::builder("hash".to_string(), "v1".to_string())
        .ok(false)
        .build();

    assert!(!should_cache(&report));
}
```

---

## 4. Canonicalization Tests

### Location
- `crates/speccade-spec/src/hash.rs` (unit tests)
- `crates/speccade-tests/tests/canonicalization.rs` (comprehensive tests)

### 4.1 Idempotence Tests

```rust
#[test]
fn test_canonicalization_idempotent_simple() {
    let value = serde_json::json!({"b": 1, "a": 2});
    let canon1 = canonicalize_json(&value).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();

    assert_eq!(canon1, canon2);
}

#[test]
fn test_canonicalization_idempotent_nested() {
    let value = serde_json::json!({
        "z": {"b": 1, "a": 2},
        "y": [3, 2, 1],
        "x": {"nested": {"deep": true}}
    });

    let canon1 = canonicalize_json(&value).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();

    assert_eq!(canon1, canon2);
}

#[test]
fn test_canonicalization_idempotent_with_arrays() {
    let value = serde_json::json!({
        "items": [
            {"id": "b", "value": 2},
            {"id": "a", "value": 1}
        ],
        "nested_arrays": [[1, 2], [3, 4]]
    });

    let canon1 = canonicalize_json(&value).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();

    assert_eq!(canon1, canon2);
}

#[test]
fn test_canonicalization_idempotent_empty_structures() {
    let value = serde_json::json!({
        "empty_object": {},
        "empty_array": [],
        "nested_empty": {"a": {}, "b": []}
    });

    let canon1 = canonicalize_json(&value).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();

    assert_eq!(canon1, canon2);
}
```

### 4.2 Float Edge Case Tests

```rust
#[test]
fn test_canonicalization_float_zero() {
    let value = serde_json::json!({"x": 0.0});
    let canon = canonicalize_json(&value).unwrap();
    assert_eq!(canon, r#"{"x":0}"#);

    // Verify idempotence
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

#[test]
fn test_canonicalization_float_integer_like() {
    let test_cases = vec![
        (1.0, "1"),
        (42.0, "42"),
        (-1.0, "-1"),
        (1000000.0, "1000000"),
    ];

    for (input, expected) in test_cases {
        let value = serde_json::json!({"x": input});
        let canon = canonicalize_json(&value).unwrap();
        assert_eq!(canon, format!(r#"{{"x":{}}}"#, expected), "Failed for {}", input);

        // Verify idempotence
        let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
        let canon2 = canonicalize_json(&reparsed).unwrap();
        assert_eq!(canon, canon2);
    }
}

#[test]
fn test_canonicalization_float_decimals() {
    let value = serde_json::json!({"x": 0.1, "y": 0.123456});
    let canon = canonicalize_json(&value).unwrap();

    // Verify it parses back correctly
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let x = reparsed["x"].as_f64().unwrap();
    let y = reparsed["y"].as_f64().unwrap();

    assert!((x - 0.1).abs() < f64::EPSILON);
    assert!((y - 0.123456).abs() < f64::EPSILON);

    // Verify idempotence
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

#[test]
fn test_canonicalization_float_large_values() {
    let value = serde_json::json!({"x": 1e10, "y": 1e15});
    let canon = canonicalize_json(&value).unwrap();

    // Verify idempotence
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

#[test]
fn test_canonicalization_float_nan_infinity() {
    // NaN and Infinity should be converted to null per JCS spec
    let n = serde_json::Number::from_f64(f64::NAN);
    let inf = serde_json::Number::from_f64(f64::INFINITY);
    let neg_inf = serde_json::Number::from_f64(f64::NEG_INFINITY);

    // These become None because serde_json doesn't allow NaN/Infinity
    assert!(n.is_none());
    assert!(inf.is_none());
    assert!(neg_inf.is_none());
}
```

### 4.3 String Edge Case Tests

```rust
#[test]
fn test_canonicalization_string_empty() {
    let value = serde_json::json!({"x": ""});
    let canon = canonicalize_json(&value).unwrap();
    assert_eq!(canon, r#"{"x":""}"#);
}

#[test]
fn test_canonicalization_string_escape_sequences() {
    let value = serde_json::json!({
        "newline": "a\nb",
        "tab": "a\tb",
        "quote": "a\"b",
        "backslash": "a\\b"
    });

    let canon = canonicalize_json(&value).unwrap();

    // Verify it contains proper escapes
    assert!(canon.contains(r#""newline":"a\nb""#));
    assert!(canon.contains(r#""tab":"a\tb""#));
    assert!(canon.contains(r#""quote":"a\"b""#));
    assert!(canon.contains(r#""backslash":"a\\b""#));

    // Verify round-trip
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

#[test]
fn test_canonicalization_string_unicode() {
    let value = serde_json::json!({
        "emoji": "\u{1F600}",
        "chinese": "\u{4e2d}\u{6587}",
        "japanese": "\u{3042}\u{3044}\u{3046}"
    });

    let canon = canonicalize_json(&value).unwrap();

    // Verify round-trip
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    assert_eq!(reparsed["emoji"].as_str().unwrap(), "\u{1F600}");

    // Verify idempotence
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}

#[test]
fn test_canonicalization_string_control_characters() {
    let value = serde_json::json!({"x": "\x00\x01\x1f"});
    let canon = canonicalize_json(&value).unwrap();

    // Should escape control characters as \uXXXX
    assert!(canon.contains("\\u0000"));
    assert!(canon.contains("\\u0001"));
    assert!(canon.contains("\\u001f"));

    // Verify round-trip
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();
    let canon2 = canonicalize_json(&reparsed).unwrap();
    assert_eq!(canon, canon2);
}
```

### 4.4 Object Key Ordering Tests

```rust
#[test]
fn test_canonicalization_key_ordering_simple() {
    let json1: serde_json::Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
    let json2: serde_json::Value = serde_json::from_str(r#"{"a": 2, "z": 1, "m": 3}"#).unwrap();
    let json3: serde_json::Value = serde_json::from_str(r#"{"m": 3, "z": 1, "a": 2}"#).unwrap();

    let canon1 = canonicalize_json(&json1).unwrap();
    let canon2 = canonicalize_json(&json2).unwrap();
    let canon3 = canonicalize_json(&json3).unwrap();

    // All should produce the same canonical form
    assert_eq!(canon1, canon2);
    assert_eq!(canon2, canon3);
    assert_eq!(canon1, r#"{"a":2,"m":3,"z":1}"#);
}

#[test]
fn test_canonicalization_key_ordering_nested() {
    let json: serde_json::Value = serde_json::from_str(r#"{
        "z": {"b": 1, "a": 2},
        "a": {"d": 3, "c": 4}
    }"#).unwrap();

    let canon = canonicalize_json(&json).unwrap();

    // Keys should be sorted at all levels
    assert_eq!(canon, r#"{"a":{"c":4,"d":3},"z":{"a":2,"b":1}}"#);
}
```

### 4.5 Array Preservation Tests

```rust
#[test]
fn test_canonicalization_preserves_array_order() {
    let value = serde_json::json!({"items": [3, 1, 2]});
    let canon = canonicalize_json(&value).unwrap();

    // Array order should be preserved
    assert_eq!(canon, r#"{"items":[3,1,2]}"#);
}

#[test]
fn test_canonicalization_preserves_object_array_order() {
    let value = serde_json::json!({
        "items": [
            {"id": "third", "value": 3},
            {"id": "first", "value": 1},
            {"id": "second", "value": 2}
        ]
    });

    let canon = canonicalize_json(&value).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&canon).unwrap();

    // Array order preserved (object keys sorted within each object)
    assert_eq!(reparsed["items"][0]["id"].as_str().unwrap(), "third");
    assert_eq!(reparsed["items"][1]["id"].as_str().unwrap(), "first");
    assert_eq!(reparsed["items"][2]["id"].as_str().unwrap(), "second");
}
```

### 4.6 Property-Based Tests (Optional)

```rust
// Add to Cargo.toml: proptest = "1.0"

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
        prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            any::<i64>().prop_map(|n| serde_json::json!(n)),
            any::<f64>()
                .prop_filter("must be finite", |f| f.is_finite())
                .prop_map(|n| serde_json::json!(n)),
            "[a-z]{0,20}".prop_map(|s| serde_json::json!(s)),
        ]
    }

    fn arb_json_object() -> impl Strategy<Value = serde_json::Value> {
        prop::collection::hash_map("[a-z]{1,10}", arb_json_value(), 0..10)
            .prop_map(|map| {
                serde_json::Value::Object(map.into_iter().collect())
            })
    }

    proptest! {
        #[test]
        fn canonicalization_is_idempotent(json in arb_json_object()) {
            let canon1 = canonicalize_json(&json).unwrap();
            let reparsed: serde_json::Value = serde_json::from_str(&canon1).unwrap();
            let canon2 = canonicalize_json(&reparsed).unwrap();
            prop_assert_eq!(canon1, canon2);
        }

        #[test]
        fn canonicalization_is_deterministic(json in arb_json_object()) {
            let canon1 = canonicalize_json(&json).unwrap();
            let canon2 = canonicalize_json(&json).unwrap();
            prop_assert_eq!(canon1, canon2);
        }

        #[test]
        fn canonical_json_is_valid_json(json in arb_json_object()) {
            let canon = canonicalize_json(&json).unwrap();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&canon);
            prop_assert!(parsed.is_ok());
        }
    }
}
```

---

## 5. Test Coverage Summary

| Category | Test Count | Location |
|----------|------------|----------|
| Budget Enforcement | ~15 tests | `validation/tests.rs`, `tests/budget_enforcement.rs` |
| Provenance | ~5 tests | `commands/generate.rs`, `tests/provenance.rs` |
| Caching | ~10 tests | `cache/mod.rs`, `tests/caching.rs` |
| Canonicalization | ~20 tests | `hash.rs`, `tests/canonicalization.rs` |
| **Total** | **~50 tests** | |

---

## 6. CI Integration

### New Test Jobs

```yaml
# Add to CI workflow

test-budgets:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run budget tests
      run: cargo test -p speccade-spec budget

test-canonicalization:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run canonicalization tests
      run: cargo test -p speccade-spec canonicalization

test-provenance:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run provenance tests
      run: cargo test -p speccade-cli provenance
```

### Budget Constant Duplication Check

```yaml
check-budget-duplicates:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Check for duplicate budget constants
      run: |
        # Find all MAX_ constants in backend crates
        backend_consts=$(grep -rh "const MAX_" crates/speccade-backend-*/src/ || true)
        # These should not duplicate spec crate constants
        if echo "$backend_consts" | grep -q "MAX_AUDIO_DURATION_SECONDS\|MAX_AUDIO_LAYERS"; then
          echo "ERROR: Backend crates contain duplicate budget constants"
          echo "These should be imported from speccade_spec::validation::budgets"
          exit 1
        fi
```
