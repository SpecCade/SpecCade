//! Static mesh recipe types.

mod common;
mod modifiers;
mod primitives;
mod static_mesh;

pub use common::{MaterialSlot, MeshConstraints, MeshExportSettings};
pub use modifiers::{MeshModifier, UvProjection, UvProjectionMethod};
pub use primitives::MeshPrimitive;
pub use static_mesh::StaticMeshBlenderPrimitivesV1Params;
