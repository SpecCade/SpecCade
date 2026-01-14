//! Bone collection types for organizing bones in groups.

use serde::{Deserialize, Serialize};

use super::error::AnimatorRigError;

fn default_true() -> bool {
    true
}

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
