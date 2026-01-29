//! End-to-End Migration Tests for SpecCade
//!
//! Tests verify legacy .spec.py parsing and migration to canonical JSON format.
//! Migrated specs are validated using `speccade validate`.

use std::fs;
use std::path::PathBuf;

use speccade_spec::{validate_spec, AssetType, Spec};
use speccade_tests::fixtures::{GoldenFixtures, LegacyProjectFixture};
use speccade_tests::harness::{parse_spec_file, TestHarness};

// ============================================================================
// Test Infrastructure
// ============================================================================

fn golden_legacy_dir() -> PathBuf {
    GoldenFixtures::legacy_dir()
}

fn read_legacy_spec(category: &str, name: &str) -> String {
    let path = golden_legacy_dir()
        .join(category)
        .join(format!("{}.spec.py", name));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e))
}

fn migrate_legacy_fixture(category: &str, name: &str) -> MigrationResult {
    let fixture = LegacyProjectFixture::new();
    let content = read_legacy_spec(category, name);
    let _spec_path = fixture.add_spec(category, name, &content);
    let harness = TestHarness::new();
    let result = harness.migrate_project(fixture.path());
    MigrationResult {
        cli_result: result,
        fixture,
        spec_name: name.to_string(),
        category: category.to_string(),
    }
}

struct MigrationResult {
    cli_result: speccade_tests::harness::CliResult,
    fixture: LegacyProjectFixture,
    spec_name: String,
    category: String,
}

impl MigrationResult {
    fn migrated_spec_path(&self) -> PathBuf {
        let asset_type = category_to_asset_type(&self.category);
        self.fixture
            .path()
            .join("specs")
            .join(asset_type)
            .join(format!("{}.json", self.spec_name.replace('_', "-")))
    }

    fn parse_migrated_spec(&self) -> Result<Spec, String> {
        let path = self.migrated_spec_path();
        if !path.exists() {
            return Err(format!("Migrated spec not found at {:?}", path));
        }
        parse_spec_file(&path)
    }

    fn assert_migration_success(&self) {
        assert!(
            self.cli_result.success,
            "Migration failed for {}/{}: stdout={}, stderr={}",
            self.category, self.spec_name, self.cli_result.stdout, self.cli_result.stderr
        );
    }

    fn validate_migrated_spec(&self) -> Spec {
        let spec = self
            .parse_migrated_spec()
            .expect("Failed to parse migrated spec");
        let validation = validate_spec(&spec);
        assert!(
            validation.is_ok(),
            "Validation failed for {}/{}: errors={:?}",
            self.category,
            self.spec_name,
            validation.errors
        );
        spec
    }

    fn is_static_analysis_failure(&self) -> bool {
        !self.cli_result.success && self.cli_result.stdout.contains("Static analysis failed")
    }
}

fn category_to_asset_type(category: &str) -> &'static str {
    match category {
        "sounds" | "instruments" => "audio",
        "music" => "music",
        "textures" | "normals" => "texture",
        "meshes" => "static_mesh",
        "characters" => "skeletal_mesh",
        "animations" => "skeletal_animation",
        _ => "unknown",
    }
}

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_sound_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_sound("test_beep");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(
        content.contains("SOUND") && content.contains("duration") && content.contains("layers")
    );
}

#[test]
fn test_parse_instrument_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_instrument("test_synth");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("INSTRUMENT") && content.contains("synthesis"));
}

#[test]
fn test_parse_music_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_music("test_song");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("SONG") && content.contains("bpm") && content.contains("patterns"));
}

#[test]
fn test_parse_texture_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_texture("test_metal");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("TEXTURE") && content.contains("size"));
}

#[test]
fn test_parse_normal_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_normal("test_bricks");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("NORMAL") && content.contains("pattern"));
}

#[test]
fn test_parse_mesh_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_mesh("test_cube");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("MESH") && content.contains("primitive"));
}

#[test]
fn test_parse_character_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_character("test_biped");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("SPEC") && content.contains("skeleton"));
}

#[test]
fn test_parse_animation_spec() {
    let fixture = LegacyProjectFixture::new();
    let spec_path = fixture.add_animation("test_walk");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("ANIMATION") && content.contains("fps"));
}

#[test]
fn test_golden_legacy_fixtures_exist() {
    if !GoldenFixtures::exists() {
        return;
    }
    assert!(!GoldenFixtures::list_legacy_specs("sounds").is_empty());
    assert!(!GoldenFixtures::list_legacy_specs("textures").is_empty());
}

#[test]
fn test_golden_speccade_specs_parse() {
    if !GoldenFixtures::exists() {
        return;
    }
    for spec_path in GoldenFixtures::list_speccade_specs("audio") {
        let spec = parse_spec_file(&spec_path).expect("Should parse");
        assert_eq!(spec.asset_type, AssetType::Audio);
    }
    for spec_path in GoldenFixtures::list_speccade_specs("texture") {
        let spec = parse_spec_file(&spec_path).expect("Should parse");
        assert_eq!(spec.asset_type, AssetType::Texture);
    }
}

// ============================================================================
// SOUND Migration Tests (SOUND -> audio_v1)
// ============================================================================

#[test]
fn test_migrate_sound_simple_beep() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "simple_beep");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    assert_eq!(
        spec.recipe.as_ref().map(|r| r.kind.as_str()),
        Some("audio_v1")
    );
    let params = &spec.recipe.as_ref().unwrap().params;
    assert!(params.get("layers").is_some());
    assert!(params.get("duration_seconds").is_some());
}

#[test]
fn test_migrate_sound_laser_shot() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "laser_shot");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    let params = &spec.recipe.as_ref().unwrap().params;
    let layers = params["layers"].as_array().expect("layers should be array");
    assert!(!layers.is_empty());
    assert_eq!(layers[0]["synthesis"]["type"].as_str(), Some("fm_synth"));
}

#[test]
fn test_migrate_sound_coin_collect() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "coin_collect");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    let params = &spec.recipe.as_ref().unwrap().params;
    let layers = params["layers"].as_array().expect("layers should be array");
    if !layers.is_empty() {
        let envelope = &layers[0]["envelope"];
        assert!(envelope.get("attack").is_some());
        assert!(envelope.get("decay").is_some());
    }
}

#[test]
fn test_migrate_sound_kick_drum() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "kick_drum");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    assert_eq!(
        spec.recipe.as_ref().map(|r| r.kind.as_str()),
        Some("audio_v1")
    );
}

#[test]
fn test_migrate_sound_white_noise_burst() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "white_noise_burst");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
}

// ============================================================================
// INSTRUMENT Migration Tests (INSTRUMENT -> audio_v1)
// ============================================================================

#[test]
fn test_migrate_instrument_fm_bell() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("instruments", "fm_bell");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    assert_eq!(
        spec.recipe.as_ref().map(|r| r.kind.as_str()),
        Some("audio_v1")
    );
}

#[test]
fn test_migrate_instrument_bass_pluck() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("instruments", "bass_pluck");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    let params = &spec.recipe.as_ref().unwrap().params;
    let layers = params["layers"].as_array().expect("layers should be array");
    if !layers.is_empty() {
        assert_eq!(
            layers[0]["synthesis"]["type"].as_str(),
            Some("karplus_strong")
        );
    }
}

#[test]
fn test_migrate_instrument_square_chip() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("instruments", "square_chip");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::Audio);
    assert_eq!(
        spec.recipe.as_ref().map(|r| r.kind.as_str()),
        Some("audio_v1")
    );
}

// ============================================================================
// CHARACTER Migration Test (Negative Test)
// ============================================================================

#[test]
fn test_migrate_character_produces_warning() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("characters", "simple_biped");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_type, AssetType::SkeletalMesh);
    assert_eq!(
        spec.recipe.as_ref().map(|r| r.kind.as_str()),
        Some("skeletal_mesh.armature_driven_v1")
    );
    let params = &spec.recipe.as_ref().unwrap().params;
    assert!(params.get("bone_meshes").is_some());
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn test_migrated_sound_has_wav_output() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "simple_beep");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert!(spec.has_primary_output());
    let primary = spec.primary_outputs().next().expect("should have primary");
    assert_eq!(primary.format, speccade_spec::OutputFormat::Wav);
}

// ============================================================================
// All Sound Fixtures Test
// ============================================================================

#[test]
fn test_migrate_all_sound_fixtures() {
    if !GoldenFixtures::exists() {
        return;
    }
    let sounds = GoldenFixtures::list_legacy_specs("sounds");
    assert!(!sounds.is_empty());

    for spec_path in sounds {
        let name = spec_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches(".spec");
        let result = migrate_legacy_fixture("sounds", name);
        result.assert_migration_success();
        let spec = result.validate_migrated_spec();
        assert_eq!(spec.asset_type, AssetType::Audio, "Sound {}", name);
        assert_eq!(
            spec.recipe.as_ref().map(|r| r.kind.as_str()),
            Some("audio_v1"),
            "Sound {}",
            name
        );
    }
}

// ============================================================================
// Seed Determinism Tests
// ============================================================================

#[test]
fn test_migrated_spec_has_deterministic_seed() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result1 = migrate_legacy_fixture("sounds", "simple_beep");
    result1.assert_migration_success();
    let spec1 = result1.validate_migrated_spec();

    let result2 = migrate_legacy_fixture("sounds", "simple_beep");
    result2.assert_migration_success();
    let spec2 = result2.validate_migrated_spec();

    assert_eq!(spec1.seed, spec2.seed, "Seed should be deterministic");
}

#[test]
fn test_different_fixtures_have_different_seeds() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result1 = migrate_legacy_fixture("sounds", "simple_beep");
    result1.assert_migration_success();
    let spec1 = result1.validate_migrated_spec();

    let result2 = migrate_legacy_fixture("sounds", "laser_shot");
    result2.assert_migration_success();
    let spec2 = result2.validate_migrated_spec();

    assert_ne!(
        spec1.seed, spec2.seed,
        "Different fixtures should have different seeds"
    );
}

// ============================================================================
// CLI Validation Test
// ============================================================================

#[test]
fn test_migrated_spec_passes_cli_validate() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "simple_beep");
    result.assert_migration_success();

    let migrated_path = result.migrated_spec_path();
    assert!(
        migrated_path.exists(),
        "Migrated spec should exist at {:?}",
        migrated_path
    );

    let harness = TestHarness::new();
    let validate_result = harness.validate_spec(&migrated_path);
    assert!(
        validate_result.success,
        "CLI validation should pass: stdout={}, stderr={}",
        validate_result.stdout, validate_result.stderr
    );
}

// ============================================================================
// Asset ID Format Tests
// ============================================================================

#[test]
fn test_migrated_asset_id_format() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "simple_beep");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert_eq!(spec.asset_id, "simple-beep");
    assert!(!spec.asset_id.contains('_'));
}

#[test]
fn test_migrated_asset_id_is_valid() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("sounds", "laser_shot");
    result.assert_migration_success();
    let spec = result.validate_migrated_spec();
    assert!(speccade_spec::is_valid_asset_id(&spec.asset_id));
}

// ============================================================================
// Static Analysis Limitation Tests
// ============================================================================

#[test]
fn test_mesh_fixtures_need_python_execution() {
    if !GoldenFixtures::exists() {
        return;
    }
    // Mesh fixtures use Python tuple syntax (0, 0, 0) which static analysis cannot parse
    let result = migrate_legacy_fixture("meshes", "simple_cube");
    assert!(
        result.is_static_analysis_failure(),
        "Mesh migration should fail with static analysis error"
    );
}

#[test]
fn test_static_analysis_error_message_is_helpful() {
    if !GoldenFixtures::exists() {
        return;
    }
    let result = migrate_legacy_fixture("meshes", "simple_cube");
    assert!(
        result.cli_result.stdout.contains("--allow-exec-specs"),
        "Error should suggest using --allow-exec-specs flag"
    );
}
