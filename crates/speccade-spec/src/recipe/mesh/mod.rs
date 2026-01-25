//! Static mesh recipe types.

mod boolean_kit;
mod common;
mod modifiers;
mod modular_kit;
mod organic_sculpt;
mod primitives;
mod shrinkwrap;
mod static_mesh;

pub use boolean_kit::{
    BooleanCleanup, BooleanOperation, BooleanOperationType, BooleanSolver, MeshReference,
    MeshSource, PrimitiveMesh, StaticMeshBooleanKitV1Params, DEFAULT_MERGE_DISTANCE,
    MAX_BOOLEAN_OPERATIONS,
};
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
pub use shrinkwrap::{
    ShrinkwrapMode, ShrinkwrapValidation, StaticMeshShrinkwrapV1Params,
    DEFAULT_OFFSET, DEFAULT_SMOOTH_ITERATIONS as SHRINKWRAP_DEFAULT_SMOOTH_ITERATIONS,
    MAX_OFFSET, MAX_SMOOTH_ITERATIONS as SHRINKWRAP_MAX_SMOOTH_ITERATIONS, MIN_OFFSET,
};
pub use static_mesh::{
    LodChainSettings, LodDecimateMethod, LodLevel, StaticMeshBlenderPrimitivesV1Params,
};
