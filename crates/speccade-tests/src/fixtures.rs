//! Test fixture utilities for creating synthetic spec trees.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A test fixture representing a project with legacy .studio/specs structure.
pub struct LegacyProjectFixture {
    pub root: TempDir,
    pub specs_dir: PathBuf,
}

impl LegacyProjectFixture {
    /// Create a new empty legacy project fixture.
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp dir");
        let specs_dir = root.path().join(".studio").join("specs");
        fs::create_dir_all(&specs_dir).expect("Failed to create .studio/specs");
        Self { root, specs_dir }
    }

    /// Get the project root path.
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    /// Add a legacy spec file to the project.
    ///
    /// # Arguments
    /// * `category` - The category (sounds, textures, meshes, etc.)
    /// * `name` - The filename without extension
    /// * `content` - The Python spec content
    pub fn add_spec(&self, category: &str, name: &str, content: &str) -> PathBuf {
        let category_dir = self.specs_dir.join(category);
        fs::create_dir_all(&category_dir).expect("Failed to create category dir");

        let spec_path = category_dir.join(format!("{}.spec.py", name));
        fs::write(&spec_path, content).expect("Failed to write spec file");
        spec_path
    }

    /// Add a minimal sound spec.
    pub fn add_sound(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
SOUND = {{
    "name": "{}",
    "duration": 0.3,
    "sample_rate": 44100,
    "layers": [
        {{
            "type": "sine",
            "freq": 440,
            "amplitude": 0.8,
            "envelope": {{
                "attack": 0.01,
                "decay": 0.05,
                "sustain": 0.6,
                "release": 0.15
            }}
        }}
    ]
}}
"#,
            name
        );
        self.add_spec("sounds", name, &content)
    }

    /// Add a minimal instrument spec.
    pub fn add_instrument(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
INSTRUMENT = {{
    "name": "{}",
    "synthesis": "subtractive",
    "oscillators": [
        {{
            "waveform": "sawtooth",
            "detune": 0.0
        }}
    ],
    "filter": {{
        "type": "lowpass",
        "cutoff": 2000.0,
        "resonance": 0.5
    }},
    "envelope": {{
        "attack": 0.01,
        "decay": 0.1,
        "sustain": 0.7,
        "release": 0.2
    }}
}}
"#,
            name
        );
        self.add_spec("instruments", name, &content)
    }

    /// Add a minimal music spec.
    pub fn add_music(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
SONG = {{
    "name": "{}",
    "format": "xm",
    "bpm": 120,
    "rows_per_beat": 4,
    "channels": 4,
    "instruments": [],
    "patterns": [],
    "arrangement": []
}}
"#,
            name
        );
        self.add_spec("music", name, &content)
    }

    /// Add a minimal texture spec.
    pub fn add_texture(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
TEXTURE = {{
    "name": "{}",
    "size": [256, 256],
    "layers": [
        {{
            "type": "solid",
            "color": 0.5
        }}
    ]
}}
"#,
            name
        );
        self.add_spec("textures", name, &content)
    }

    /// Add a minimal normal map spec.
    pub fn add_normal(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
NORMAL = {{
    "name": "{}",
    "size": [256, 256],
    "pattern": {{
        "type": "bricks",
        "brick_width": 64,
        "brick_height": 32,
        "mortar_depth": 0.1
    }}
}}
"#,
            name
        );
        self.add_spec("normals", name, &content)
    }

    /// Add a minimal mesh spec.
    pub fn add_mesh(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
MESH = {{
    "name": "{}",
    "primitive": "cube",
    "params": {{
        "size": 1.0
    }},
    "modifiers": []
}}
"#,
            name
        );
        self.add_spec("meshes", name, &content)
    }

    /// Add a minimal character spec.
    pub fn add_character(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
SPEC = {{
    "name": "{}",
    "tri_budget": 500,
    "skeleton": [
        {{"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.5]}}
    ],
    "parts": {{}}
}}
"#,
            name
        );
        self.add_spec("characters", name, &content)
    }

    /// Add a minimal animation spec.
    pub fn add_animation(&self, name: &str) -> PathBuf {
        let content = format!(
            r#"# Legacy .spec.py - test fixture
ANIMATION = {{
    "name": "{}",
    "fps": 30,
    "frame_count": 60,
    "loop": True,
    "poses": []
}}
"#,
            name
        );
        self.add_spec("animations", name, &content)
    }
}

impl Default for LegacyProjectFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Paths to golden test fixtures in the repository.
pub struct GoldenFixtures;

impl GoldenFixtures {
    /// Get the path to the golden legacy specs directory.
    pub fn legacy_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("legacy")
    }

    /// Get the path to the golden speccade specs directory.
    pub fn speccade_specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("specs")
    }

    /// Get the path to the expected hashes directory.
    pub fn expected_hashes_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("expected")
            .join("hashes")
    }

    /// Get the path to the expected metrics directory.
    pub fn expected_metrics_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("expected")
            .join("metrics")
    }

    /// Check if golden fixtures exist.
    pub fn exists() -> bool {
        Self::legacy_dir().exists() && Self::speccade_specs_dir().exists()
    }

    /// List all legacy spec files in a category.
    pub fn list_legacy_specs(category: &str) -> Vec<PathBuf> {
        let category_dir = Self::legacy_dir().join(category);
        if !category_dir.exists() {
            return Vec::new();
        }

        fs::read_dir(&category_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|e| e == "py")
                    .unwrap_or(false)
            })
            .collect()
    }

    /// List all canonical spec files in a category.
    pub fn list_speccade_specs(asset_type: &str) -> Vec<PathBuf> {
        let type_dir = Self::speccade_specs_dir().join(asset_type);
        if !type_dir.exists() {
            return Vec::new();
        }

        fs::read_dir(&type_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|e| e == "json")
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_project_fixture_creation() {
        let fixture = LegacyProjectFixture::new();
        assert!(fixture.path().exists());
        assert!(fixture.specs_dir.exists());
    }

    #[test]
    fn test_add_sound_spec() {
        let fixture = LegacyProjectFixture::new();
        let path = fixture.add_sound("test_beep");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("SOUND"));
        assert!(content.contains("test_beep"));
    }

    #[test]
    fn test_add_multiple_categories() {
        let fixture = LegacyProjectFixture::new();
        fixture.add_sound("beep");
        fixture.add_texture("metal");
        fixture.add_mesh("cube");

        assert!(fixture.specs_dir.join("sounds").join("beep.spec.py").exists());
        assert!(fixture.specs_dir.join("textures").join("metal.spec.py").exists());
        assert!(fixture.specs_dir.join("meshes").join("cube.spec.py").exists());
    }

    #[test]
    fn test_golden_fixtures_paths() {
        // These paths should exist in the repo
        let legacy = GoldenFixtures::legacy_dir();
        let speccade = GoldenFixtures::speccade_specs_dir();

        // Print paths for debugging
        println!("Legacy dir: {:?}", legacy);
        println!("Speccade specs dir: {:?}", speccade);
    }
}
