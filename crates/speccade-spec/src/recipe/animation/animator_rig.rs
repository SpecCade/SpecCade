//! Animator rig configuration types for visual aids in Blender.

use serde::{Deserialize, Serialize};

// =============================================================================
// Widget Styles
// =============================================================================

/// Widget shape styles for bone visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetStyle {
    /// Wireframe circle (default, good for rotation controls).
    #[default]
    WireCircle,
    /// Wireframe cube (good for position controls).
    WireCube,
    /// Wireframe sphere (good for ball joints).
    WireSphere,
    /// Wireframe diamond (good for IK targets).
    WireDiamond,
    /// Custom mesh widget (requires external mesh reference).
    CustomMesh,
}

impl WidgetStyle {
    /// Returns the Blender object name for this widget style.
    pub fn blender_name(&self) -> &'static str {
        match self {
            WidgetStyle::WireCircle => "WGT_circle",
            WidgetStyle::WireCube => "WGT_cube",
            WidgetStyle::WireSphere => "WGT_sphere",
            WidgetStyle::WireDiamond => "WGT_diamond",
            WidgetStyle::CustomMesh => "WGT_custom",
        }
    }

    /// Returns all standard widget styles (excluding CustomMesh).
    pub fn standard_styles() -> &'static [WidgetStyle] {
        &[
            WidgetStyle::WireCircle,
            WidgetStyle::WireCube,
            WidgetStyle::WireSphere,
            WidgetStyle::WireDiamond,
        ]
    }
}

// =============================================================================
// Bone Collections
// =============================================================================

/// Bone collection definition for organizing bones in groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneCollection {
    /// Name of the collection (e.g., "IK Controls", "FK Controls").
    pub name: String,
    /// List of bone names in this collection.
    pub bones: Vec<String>,
    /// Whether this collection is visible by default.
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether bones in this collection are selectable.
    #[serde(default = "default_true")]
    pub selectable: bool,
}

fn default_true() -> bool {
    true
}

impl BoneCollection {
    /// Creates a new bone collection with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bones: Vec::new(),
            visible: true,
            selectable: true,
        }
    }

    /// Adds a bone to this collection.
    pub fn with_bone(mut self, bone: impl Into<String>) -> Self {
        self.bones.push(bone.into());
        self
    }

    /// Adds multiple bones to this collection.
    pub fn with_bones(mut self, bones: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.bones.extend(bones.into_iter().map(|b| b.into()));
        self
    }

    /// Sets the visibility of this collection.
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets the selectability of this collection.
    pub fn with_selectability(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Validates the bone collection.
    pub fn validate(&self) -> Result<(), AnimatorRigError> {
        if self.name.is_empty() {
            return Err(AnimatorRigError::EmptyCollectionName);
        }
        Ok(())
    }
}

/// Standard bone collection presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoneCollectionPreset {
    /// IK control bones (targets and poles).
    IkControls,
    /// FK control bones (direct rotation controls).
    FkControls,
    /// Deformation bones (actual skin deformers).
    Deform,
    /// Mechanism bones (helper bones, not for direct animation).
    Mechanism,
}

impl BoneCollectionPreset {
    /// Returns the default name for this preset.
    pub fn default_name(&self) -> &'static str {
        match self {
            BoneCollectionPreset::IkControls => "IK Controls",
            BoneCollectionPreset::FkControls => "FK Controls",
            BoneCollectionPreset::Deform => "Deform",
            BoneCollectionPreset::Mechanism => "Mechanism",
        }
    }

    /// Returns whether bones in this collection should be visible by default.
    pub fn default_visibility(&self) -> bool {
        match self {
            BoneCollectionPreset::IkControls => true,
            BoneCollectionPreset::FkControls => true,
            BoneCollectionPreset::Deform => false,
            BoneCollectionPreset::Mechanism => false,
        }
    }

    /// Returns whether bones in this collection should be selectable by default.
    pub fn default_selectability(&self) -> bool {
        match self {
            BoneCollectionPreset::IkControls => true,
            BoneCollectionPreset::FkControls => true,
            BoneCollectionPreset::Deform => false,
            BoneCollectionPreset::Mechanism => false,
        }
    }

    /// Creates a bone collection from this preset.
    pub fn to_collection(&self) -> BoneCollection {
        BoneCollection {
            name: self.default_name().to_string(),
            bones: Vec::new(),
            visible: self.default_visibility(),
            selectable: self.default_selectability(),
        }
    }
}

// =============================================================================
// Bone Colors
// =============================================================================

/// RGB color value for bone coloring (0.0-1.0 range).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoneColor {
    /// Red component (0.0-1.0).
    pub r: f64,
    /// Green component (0.0-1.0).
    pub g: f64,
    /// Blue component (0.0-1.0).
    pub b: f64,
}

impl BoneColor {
    /// Creates a new bone color from RGB values.
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    /// Standard blue color for left-side bones.
    pub fn left_blue() -> Self {
        Self::new(0.2, 0.4, 1.0)
    }

    /// Standard red color for right-side bones.
    pub fn right_red() -> Self {
        Self::new(1.0, 0.3, 0.3)
    }

    /// Standard yellow color for center bones.
    pub fn center_yellow() -> Self {
        Self::new(1.0, 0.9, 0.2)
    }

    /// White color (default/neutral).
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Returns the color as an array [R, G, B].
    pub fn as_array(&self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for BoneColor {
    fn default() -> Self {
        Self::white()
    }
}

/// Bone color scheme for automatic bone coloring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "scheme", rename_all = "snake_case", deny_unknown_fields)]
pub enum BoneColorScheme {
    /// Standard scheme: left=blue, right=red, center=yellow.
    #[default]
    Standard,
    /// Custom color mapping by bone side.
    Custom {
        /// Color for left-side bones (suffix `_l` or `_L`).
        left: BoneColor,
        /// Color for right-side bones (suffix `_r` or `_R`).
        right: BoneColor,
        /// Color for center bones (no side suffix).
        center: BoneColor,
    },
    /// Per-bone custom colors.
    PerBone {
        /// Map of bone name to color.
        colors: std::collections::HashMap<String, BoneColor>,
        /// Default color for bones not in the map.
        #[serde(default)]
        default: BoneColor,
    },
}

impl BoneColorScheme {
    /// Creates the standard color scheme (L=blue, R=red, center=yellow).
    pub fn standard() -> Self {
        BoneColorScheme::Standard
    }

    /// Creates a custom color scheme with specified colors.
    pub fn custom(left: BoneColor, right: BoneColor, center: BoneColor) -> Self {
        BoneColorScheme::Custom {
            left,
            right,
            center,
        }
    }

    /// Returns the color for a given bone name based on the scheme.
    pub fn color_for_bone(&self, bone_name: &str) -> BoneColor {
        match self {
            BoneColorScheme::Standard => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    BoneColor::left_blue()
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    BoneColor::right_red()
                } else {
                    BoneColor::center_yellow()
                }
            }
            BoneColorScheme::Custom {
                left,
                right,
                center,
            } => {
                if bone_name.ends_with("_l") || bone_name.ends_with("_L") {
                    *left
                } else if bone_name.ends_with("_r") || bone_name.ends_with("_R") {
                    *right
                } else {
                    *center
                }
            }
            BoneColorScheme::PerBone { colors, default } => {
                colors.get(bone_name).copied().unwrap_or(*default)
            }
        }
    }
}

// =============================================================================
// Armature Display
// =============================================================================

/// Armature display type in Blender viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArmatureDisplay {
    /// Octahedral bone shapes (default).
    #[default]
    Octahedral,
    /// Stick bone shapes.
    Stick,
    /// B-Bone (bendy bone) shapes.
    Bbone,
    /// Envelope shapes.
    Envelope,
    /// Wire shapes.
    Wire,
}

impl ArmatureDisplay {
    /// Returns the Blender enum name for this display type.
    pub fn blender_name(&self) -> &'static str {
        match self {
            ArmatureDisplay::Octahedral => "OCTAHEDRAL",
            ArmatureDisplay::Stick => "STICK",
            ArmatureDisplay::Bbone => "BBONE",
            ArmatureDisplay::Envelope => "ENVELOPE",
            ArmatureDisplay::Wire => "WIRE",
        }
    }
}

// =============================================================================
// Animator Rig Config
// =============================================================================

/// Configuration for animator rig visual aids.
///
/// This configuration controls how the rig appears to animators in Blender,
/// including bone collections, custom shapes, and color coding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimatorRigConfig {
    /// Whether to organize bones into collections.
    #[serde(default = "default_true")]
    pub collections: bool,
    /// Whether to add custom bone shapes (widgets).
    #[serde(default = "default_true")]
    pub shapes: bool,
    /// Whether to apply color coding to bones.
    #[serde(default = "default_true")]
    pub colors: bool,
    /// Armature display type in viewport.
    #[serde(default)]
    pub display: ArmatureDisplay,
    /// Widget style for control bones.
    #[serde(default)]
    pub widget_style: WidgetStyle,
    /// Custom bone collections (in addition to or replacing defaults).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bone_collections: Vec<BoneCollection>,
    /// Bone color scheme.
    #[serde(default)]
    pub bone_colors: BoneColorScheme,
}

impl Default for AnimatorRigConfig {
    fn default() -> Self {
        Self {
            collections: true,
            shapes: true,
            colors: true,
            display: ArmatureDisplay::default(),
            widget_style: WidgetStyle::default(),
            bone_collections: Vec::new(),
            bone_colors: BoneColorScheme::default(),
        }
    }
}

impl AnimatorRigConfig {
    /// Creates a new animator rig config with all features enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a minimal config with no visual aids.
    pub fn minimal() -> Self {
        Self {
            collections: false,
            shapes: false,
            colors: false,
            display: ArmatureDisplay::Octahedral,
            widget_style: WidgetStyle::WireCircle,
            bone_collections: Vec::new(),
            bone_colors: BoneColorScheme::Standard,
        }
    }

    /// Sets whether to organize bones into collections.
    pub fn with_collections(mut self, enabled: bool) -> Self {
        self.collections = enabled;
        self
    }

    /// Sets whether to add custom bone shapes.
    pub fn with_shapes(mut self, enabled: bool) -> Self {
        self.shapes = enabled;
        self
    }

    /// Sets whether to apply color coding.
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colors = enabled;
        self
    }

    /// Sets the armature display type.
    pub fn with_display(mut self, display: ArmatureDisplay) -> Self {
        self.display = display;
        self
    }

    /// Sets the widget style for control bones.
    pub fn with_widget_style(mut self, style: WidgetStyle) -> Self {
        self.widget_style = style;
        self
    }

    /// Adds a bone collection.
    pub fn with_bone_collection(mut self, collection: BoneCollection) -> Self {
        self.bone_collections.push(collection);
        self
    }

    /// Sets the bone color scheme.
    pub fn with_bone_colors(mut self, scheme: BoneColorScheme) -> Self {
        self.bone_colors = scheme;
        self
    }

    /// Validates the animator rig configuration.
    pub fn validate(&self) -> Result<(), AnimatorRigError> {
        for collection in &self.bone_collections {
            collection.validate()?;
        }
        Ok(())
    }

    /// Creates default bone collections for a humanoid rig.
    pub fn default_humanoid_collections() -> Vec<BoneCollection> {
        vec![
            BoneCollectionPreset::IkControls
                .to_collection()
                .with_bones([
                    "ik_foot_l",
                    "ik_foot_r",
                    "ik_hand_l",
                    "ik_hand_r",
                    "pole_knee_l",
                    "pole_knee_r",
                    "pole_elbow_l",
                    "pole_elbow_r",
                ]),
            BoneCollectionPreset::FkControls
                .to_collection()
                .with_bones([
                    "root",
                    "hips",
                    "spine",
                    "chest",
                    "neck",
                    "head",
                    "shoulder_l",
                    "shoulder_r",
                ]),
            BoneCollectionPreset::Deform.to_collection().with_bones([
                "upper_arm_l",
                "lower_arm_l",
                "hand_l",
                "upper_arm_r",
                "lower_arm_r",
                "hand_r",
                "upper_leg_l",
                "lower_leg_l",
                "foot_l",
                "upper_leg_r",
                "lower_leg_r",
                "foot_r",
            ]),
            BoneCollectionPreset::Mechanism.to_collection(),
        ]
    }
}

// =============================================================================
// Errors
// =============================================================================

/// Errors that can occur when validating animator rig configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimatorRigError {
    /// Bone collection name is empty.
    EmptyCollectionName,
    /// Duplicate collection name.
    DuplicateCollectionName(String),
    /// Invalid widget style for bone type.
    InvalidWidgetStyle { bone: String, style: String },
}

impl std::fmt::Display for AnimatorRigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimatorRigError::EmptyCollectionName => {
                write!(f, "Bone collection name cannot be empty")
            }
            AnimatorRigError::DuplicateCollectionName(name) => {
                write!(f, "Duplicate bone collection name: {}", name)
            }
            AnimatorRigError::InvalidWidgetStyle { bone, style } => {
                write!(f, "Invalid widget style '{}' for bone '{}'", style, bone)
            }
        }
    }
}

impl std::error::Error for AnimatorRigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_style_serde() {
        // Test all variants serialize correctly
        let styles = [
            (WidgetStyle::WireCircle, "\"wire_circle\""),
            (WidgetStyle::WireCube, "\"wire_cube\""),
            (WidgetStyle::WireSphere, "\"wire_sphere\""),
            (WidgetStyle::WireDiamond, "\"wire_diamond\""),
            (WidgetStyle::CustomMesh, "\"custom_mesh\""),
        ];
        for (style, expected) in styles {
            let json = serde_json::to_string(&style).unwrap();
            assert_eq!(json, expected);
        }

        // Test default
        assert_eq!(WidgetStyle::default(), WidgetStyle::WireCircle);

        // Test blender_name
        assert_eq!(WidgetStyle::WireCircle.blender_name(), "WGT_circle");
        assert_eq!(WidgetStyle::WireDiamond.blender_name(), "WGT_diamond");

        // Test standard_styles
        let standard = WidgetStyle::standard_styles();
        assert_eq!(standard.len(), 4);
        assert!(!standard.contains(&WidgetStyle::CustomMesh));
    }

    #[test]
    fn test_bone_collection_serde() {
        let collection = BoneCollection::new("IK Controls")
            .with_bone("ik_foot_l")
            .with_bone("ik_foot_r")
            .with_visibility(true)
            .with_selectability(true);

        let json = serde_json::to_string(&collection).unwrap();
        assert!(json.contains("IK Controls"));
        assert!(json.contains("ik_foot_l"));
        assert!(json.contains("ik_foot_r"));

        let parsed: BoneCollection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "IK Controls");
        assert_eq!(parsed.bones.len(), 2);
        assert!(parsed.visible);
        assert!(parsed.selectable);
    }

    #[test]
    fn test_bone_collection_with_bones() {
        let collection =
            BoneCollection::new("Deform").with_bones(["arm_l", "arm_r", "leg_l", "leg_r"]);

        assert_eq!(collection.bones.len(), 4);
        assert!(collection.bones.contains(&"arm_l".to_string()));
    }

    #[test]
    fn test_bone_collection_validation() {
        // Valid collection
        let valid = BoneCollection::new("Test");
        assert!(valid.validate().is_ok());

        // Invalid - empty name
        let invalid = BoneCollection::new("");
        assert_eq!(
            invalid.validate(),
            Err(AnimatorRigError::EmptyCollectionName)
        );
    }

    #[test]
    fn test_bone_collection_preset() {
        // Test all presets
        let presets = [
            (BoneCollectionPreset::IkControls, "IK Controls", true, true),
            (BoneCollectionPreset::FkControls, "FK Controls", true, true),
            (BoneCollectionPreset::Deform, "Deform", false, false),
            (BoneCollectionPreset::Mechanism, "Mechanism", false, false),
        ];

        for (preset, name, visible, selectable) in presets {
            assert_eq!(preset.default_name(), name);
            assert_eq!(preset.default_visibility(), visible);
            assert_eq!(preset.default_selectability(), selectable);

            let collection = preset.to_collection();
            assert_eq!(collection.name, name);
            assert_eq!(collection.visible, visible);
            assert_eq!(collection.selectable, selectable);
        }
    }

    #[test]
    fn test_bone_color() {
        // Test constructors
        let color = BoneColor::new(0.5, 0.7, 0.9);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.7);
        assert_eq!(color.b, 0.9);

        // Test clamping
        let clamped = BoneColor::new(1.5, -0.5, 0.5);
        assert_eq!(clamped.r, 1.0);
        assert_eq!(clamped.g, 0.0);
        assert_eq!(clamped.b, 0.5);

        // Test standard colors
        let blue = BoneColor::left_blue();
        assert!(blue.b > blue.r);
        assert!(blue.b > blue.g);

        let red = BoneColor::right_red();
        assert!(red.r > red.g);
        assert!(red.r > red.b);

        let yellow = BoneColor::center_yellow();
        assert!(yellow.r > 0.9);
        assert!(yellow.g > 0.8);

        // Test as_array
        let arr = color.as_array();
        assert_eq!(arr, [0.5, 0.7, 0.9]);

        // Test default
        let default = BoneColor::default();
        assert_eq!(default, BoneColor::white());
    }

    #[test]
    fn test_bone_color_serde() {
        let color = BoneColor::new(0.2, 0.4, 1.0);
        let json = serde_json::to_string(&color).unwrap();
        assert!(json.contains("0.2"));
        assert!(json.contains("0.4"));
        assert!(json.contains("1.0"));

        let parsed: BoneColor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.r, 0.2);
        assert_eq!(parsed.g, 0.4);
        assert_eq!(parsed.b, 1.0);
    }

    #[test]
    fn test_bone_color_scheme_standard() {
        let scheme = BoneColorScheme::Standard;

        // Left bones (suffix _l)
        let left_color = scheme.color_for_bone("arm_l");
        assert_eq!(left_color, BoneColor::left_blue());

        // Right bones (suffix _r)
        let right_color = scheme.color_for_bone("arm_r");
        assert_eq!(right_color, BoneColor::right_red());

        // Center bones (no suffix)
        let center_color = scheme.color_for_bone("spine");
        assert_eq!(center_color, BoneColor::center_yellow());

        // Test uppercase suffix
        let left_upper = scheme.color_for_bone("arm_L");
        assert_eq!(left_upper, BoneColor::left_blue());
    }

    #[test]
    fn test_bone_color_scheme_custom() {
        let scheme = BoneColorScheme::custom(
            BoneColor::new(0.0, 1.0, 0.0), // green for left
            BoneColor::new(1.0, 0.0, 1.0), // magenta for right
            BoneColor::new(0.5, 0.5, 0.5), // gray for center
        );

        let left = scheme.color_for_bone("leg_l");
        assert_eq!(left.g, 1.0);

        let right = scheme.color_for_bone("leg_r");
        assert_eq!(right.r, 1.0);
        assert_eq!(right.b, 1.0);

        let center = scheme.color_for_bone("head");
        assert_eq!(center.r, 0.5);
    }

    #[test]
    fn test_bone_color_scheme_per_bone() {
        let mut colors = std::collections::HashMap::new();
        colors.insert("special_bone".to_string(), BoneColor::new(1.0, 0.5, 0.0));

        let scheme = BoneColorScheme::PerBone {
            colors,
            default: BoneColor::white(),
        };

        let special = scheme.color_for_bone("special_bone");
        assert_eq!(special.r, 1.0);
        assert_eq!(special.g, 0.5);

        let other = scheme.color_for_bone("other_bone");
        assert_eq!(other, BoneColor::white());
    }

    #[test]
    fn test_bone_color_scheme_serde() {
        // Standard scheme
        let standard = BoneColorScheme::Standard;
        let json = serde_json::to_string(&standard).unwrap();
        assert!(json.contains("standard"));

        let parsed: BoneColorScheme = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, BoneColorScheme::Standard));

        // Custom scheme
        let custom = BoneColorScheme::custom(
            BoneColor::left_blue(),
            BoneColor::right_red(),
            BoneColor::center_yellow(),
        );
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("custom"));
        assert!(json.contains("left"));
        assert!(json.contains("right"));
        assert!(json.contains("center"));
    }

    #[test]
    fn test_armature_display_serde() {
        let displays = [
            (ArmatureDisplay::Octahedral, "\"OCTAHEDRAL\""),
            (ArmatureDisplay::Stick, "\"STICK\""),
            (ArmatureDisplay::Bbone, "\"BBONE\""),
            (ArmatureDisplay::Envelope, "\"ENVELOPE\""),
            (ArmatureDisplay::Wire, "\"WIRE\""),
        ];

        for (display, expected) in displays {
            let json = serde_json::to_string(&display).unwrap();
            assert_eq!(json, expected);
            assert_eq!(display.blender_name(), expected.trim_matches('"'));
        }

        // Test default
        assert_eq!(ArmatureDisplay::default(), ArmatureDisplay::Octahedral);
    }

    #[test]
    fn test_animator_rig_config_default() {
        let config = AnimatorRigConfig::new();

        // All features enabled by default
        assert!(config.collections);
        assert!(config.shapes);
        assert!(config.colors);
        assert_eq!(config.display, ArmatureDisplay::Octahedral);
        assert_eq!(config.widget_style, WidgetStyle::WireCircle);
        assert!(config.bone_collections.is_empty());
        assert!(matches!(config.bone_colors, BoneColorScheme::Standard));
    }

    #[test]
    fn test_animator_rig_config_minimal() {
        let config = AnimatorRigConfig::minimal();

        // All features disabled
        assert!(!config.collections);
        assert!(!config.shapes);
        assert!(!config.colors);
    }

    #[test]
    fn test_animator_rig_config_builder() {
        let config = AnimatorRigConfig::new()
            .with_collections(true)
            .with_shapes(true)
            .with_colors(false)
            .with_display(ArmatureDisplay::Stick)
            .with_widget_style(WidgetStyle::WireDiamond)
            .with_bone_collection(BoneCollection::new("Custom"))
            .with_bone_colors(BoneColorScheme::Standard);

        assert!(config.collections);
        assert!(config.shapes);
        assert!(!config.colors);
        assert_eq!(config.display, ArmatureDisplay::Stick);
        assert_eq!(config.widget_style, WidgetStyle::WireDiamond);
        assert_eq!(config.bone_collections.len(), 1);
    }

    #[test]
    fn test_animator_rig_config_serde() {
        let config = AnimatorRigConfig::new()
            .with_display(ArmatureDisplay::Bbone)
            .with_widget_style(WidgetStyle::WireSphere)
            .with_bone_collection(
                BoneCollection::new("Test Collection").with_bones(["bone1", "bone2"]),
            );

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("BBONE"));
        assert!(json.contains("wire_sphere"));
        assert!(json.contains("Test Collection"));
        assert!(json.contains("bone1"));

        let parsed: AnimatorRigConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.display, ArmatureDisplay::Bbone);
        assert_eq!(parsed.widget_style, WidgetStyle::WireSphere);
        assert_eq!(parsed.bone_collections.len(), 1);
    }

    #[test]
    fn test_animator_rig_config_validation() {
        // Valid config
        let valid = AnimatorRigConfig::new().with_bone_collection(BoneCollection::new("Valid"));
        assert!(valid.validate().is_ok());

        // Invalid - empty collection name
        let invalid = AnimatorRigConfig::new().with_bone_collection(BoneCollection::new(""));
        assert_eq!(
            invalid.validate(),
            Err(AnimatorRigError::EmptyCollectionName)
        );
    }

    #[test]
    fn test_animator_rig_config_default_humanoid_collections() {
        let collections = AnimatorRigConfig::default_humanoid_collections();

        assert_eq!(collections.len(), 4);

        // Check IK Controls
        let ik = &collections[0];
        assert_eq!(ik.name, "IK Controls");
        assert!(ik.bones.contains(&"ik_foot_l".to_string()));
        assert!(ik.visible);

        // Check FK Controls
        let fk = &collections[1];
        assert_eq!(fk.name, "FK Controls");
        assert!(fk.bones.contains(&"root".to_string()));

        // Check Deform
        let deform = &collections[2];
        assert_eq!(deform.name, "Deform");
        assert!(!deform.visible);
        assert!(!deform.selectable);

        // Check Mechanism
        let mechanism = &collections[3];
        assert_eq!(mechanism.name, "Mechanism");
        assert!(!mechanism.visible);
    }

    #[test]
    fn test_animator_rig_error_display() {
        assert_eq!(
            AnimatorRigError::EmptyCollectionName.to_string(),
            "Bone collection name cannot be empty"
        );
        assert_eq!(
            AnimatorRigError::DuplicateCollectionName("Test".to_string()).to_string(),
            "Duplicate bone collection name: Test"
        );
        assert_eq!(
            AnimatorRigError::InvalidWidgetStyle {
                bone: "arm".to_string(),
                style: "invalid".to_string()
            }
            .to_string(),
            "Invalid widget style 'invalid' for bone 'arm'"
        );
    }
}
