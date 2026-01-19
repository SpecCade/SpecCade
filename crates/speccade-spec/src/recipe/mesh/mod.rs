//! Static mesh recipe types.

mod common;
mod modifiers;
mod primitives;
mod static_mesh;

pub use common::{BakeType, BakingSettings, CollisionMeshSettings, CollisionType, MaterialSlot, MeshConstraints, MeshExportSettings, NavmeshSettings, NormalsPreset, NormalsSettings};
pub use modifiers::{MeshModifier, UvProjection, UvProjectionMethod};
pub use primitives::MeshPrimitive;
pub use static_mesh::{
    LodChainSettings, LodDecimateMethod, LodLevel, StaticMeshBlenderPrimitivesV1Params,
};
