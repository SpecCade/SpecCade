//! Main animator rig configuration types.

use serde::{Deserialize, Serialize};

use super::bone_collection::{BoneCollection, BoneCollectionPreset};
use super::bone_color::BoneColorScheme;
use super::error::AnimatorRigError;
use super::widget::WidgetStyle;

fn default_true() -> bool {
    true
}

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
