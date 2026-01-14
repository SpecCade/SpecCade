//! Error types for animator rig configuration.

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
