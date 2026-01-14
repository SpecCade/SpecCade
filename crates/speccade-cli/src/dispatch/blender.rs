//! Blender backend dispatch handlers

use super::DispatchError;
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
use std::path::{Path, PathBuf};

/// Generate static mesh using the Blender backend
pub(super) fn generate_blender_static_mesh(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::static_mesh::generate(spec, out_root).map_err(|e| {
        DispatchError::BackendError(format!("Static mesh generation failed: {}", e))
    })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "static_mesh.blender_primitives_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

    // Convert metrics to OutputMetrics
    let metrics =
        speccade_spec::OutputMetrics {
            triangle_count: result.metrics.triangle_count,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            uv_island_count: result.metrics.uv_island_count,
            bone_count: None,
            material_slot_count: result.metrics.material_slot_count,
            max_bone_influences: None,
            animation_frame_count: None,
            animation_duration_seconds: None,
        };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}

/// Generate skeletal mesh using the Blender backend
pub(super) fn generate_blender_skeletal_mesh(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result =
        speccade_backend_blender::skeletal_mesh::generate(spec, out_root).map_err(|e| {
            DispatchError::BackendError(format!("Skeletal mesh generation failed: {}", e))
        })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "skeletal_mesh.blender_rigged_mesh_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

    // Convert metrics to OutputMetrics
    let metrics =
        speccade_spec::OutputMetrics {
            triangle_count: result.metrics.triangle_count,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            uv_island_count: result.metrics.uv_island_count,
            bone_count: result.metrics.bone_count,
            material_slot_count: result.metrics.material_slot_count,
            max_bone_influences: result.metrics.max_bone_influences,
            animation_frame_count: None,
            animation_duration_seconds: None,
        };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}

/// Generate animation using the Blender backend
pub(super) fn generate_blender_animation(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::animation::generate(spec, out_root)
        .map_err(|e| DispatchError::BackendError(format!("Animation generation failed: {}", e)))?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "skeletal_animation.blender_clip_v1 requires primary output format 'glb', got '{}'",
            primary_output.format
        )));
    }

    // Convert metrics to OutputMetrics
    let metrics = speccade_spec::OutputMetrics {
        triangle_count: None,
        bounding_box: None,
        uv_island_count: None,
        bone_count: result.metrics.bone_count,
        material_slot_count: None,
        max_bone_influences: None,
        animation_frame_count: result.metrics.animation_frame_count,
        animation_duration_seconds: result.metrics.animation_duration_seconds.map(|d| d as f32),
    };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}
