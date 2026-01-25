//! Static mesh recipe types.

mod common;
mod modifiers;
mod modular_kit;
mod primitives;
mod static_mesh;

pub use common::{
    BakeType, BakingSettings, CollisionMeshSettings, CollisionType, MaterialSlot, MeshConstraints,
    MeshExportSettings, NavmeshSettings, NormalsPreset, NormalsSettings,
};
pub use modifiers::{MeshModifier, UvProjection, UvProjectionMethod};
pub use modular_kit::{
    CutoutType, DoorKitParams, HingeSide, ModularKitType, PipeKitParams, PipeSegment,
    StaticMeshModularKitV1Params, WallCutout, WallKitParams, MAX_PIPE_SEGMENTS, MAX_WALL_CUTOUTS,
};
pub use primitives::MeshPrimitive;
pub use static_mesh::{
    LodChainSettings, LodDecimateMethod, LodLevel, StaticMeshBlenderPrimitivesV1Params,
};
