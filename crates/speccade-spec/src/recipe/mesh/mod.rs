//! Static mesh recipe types.

mod common;
mod modifiers;
mod modular_kit;
mod organic_sculpt;
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
pub use organic_sculpt::{
    DisplacementNoise, MetaballSource, StaticMeshOrganicSculptV1Params, MAX_METABALLS,
    MAX_REMESH_VOXEL_SIZE, MAX_SMOOTH_ITERATIONS, MIN_REMESH_VOXEL_SIZE,
};
pub use primitives::MeshPrimitive;
pub use static_mesh::{
    LodChainSettings, LodDecimateMethod, LodLevel, StaticMeshBlenderPrimitivesV1Params,
};
