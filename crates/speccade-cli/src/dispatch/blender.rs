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
            vertex_count: result.metrics.vertex_count,
            face_count: result.metrics.face_count,
            edge_count: result.metrics.edge_count,
            triangle_count: result.metrics.triangle_count,
            quad_count: result.metrics.quad_count,
            quad_percentage: result.metrics.quad_percentage,
            manifold: result.metrics.manifold,
            non_manifold_edge_count: result.metrics.non_manifold_edge_count,
            degenerate_face_count: result.metrics.degenerate_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            bone_count: None,
            max_bone_influences: None,
            unweighted_vertex_count: None,
            weight_normalization_percentage: None,
            material_slot_count: result.metrics.material_slot_count,
            animation_frame_count: None,
            animation_duration_seconds: None,
            hinge_axis_violations: None,
            range_violations: None,
            velocity_spikes: None,
            root_motion_delta: None,
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
            vertex_count: result.metrics.vertex_count,
            face_count: result.metrics.face_count,
            edge_count: result.metrics.edge_count,
            triangle_count: result.metrics.triangle_count,
            quad_count: result.metrics.quad_count,
            quad_percentage: result.metrics.quad_percentage,
            manifold: result.metrics.manifold,
            non_manifold_edge_count: result.metrics.non_manifold_edge_count,
            degenerate_face_count: result.metrics.degenerate_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            bone_count: result.metrics.bone_count,
            max_bone_influences: result.metrics.max_bone_influences,
            unweighted_vertex_count: None, // TODO: Add to BlenderMetrics when Python reports this
            weight_normalization_percentage: None, // TODO: Add to BlenderMetrics when Python reports this
            material_slot_count: result.metrics.material_slot_count,
            animation_frame_count: None,
            animation_duration_seconds: None,
            hinge_axis_violations: None,
            range_violations: None,
            velocity_spikes: None,
            root_motion_delta: None,
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
        vertex_count: None,
        face_count: None,
        edge_count: None,
        triangle_count: None,
        quad_count: None,
        quad_percentage: None,
        manifold: None,
        non_manifold_edge_count: None,
        degenerate_face_count: None,
        uv_island_count: None,
        uv_coverage: None,
        uv_overlap_percentage: None,
        bounding_box: None,
        bounds_min: None,
        bounds_max: None,
        bone_count: result.metrics.bone_count,
        max_bone_influences: None,
        unweighted_vertex_count: None,
        weight_normalization_percentage: None,
        material_slot_count: None,
        animation_frame_count: result.metrics.animation_frame_count,
        animation_duration_seconds: result.metrics.animation_duration_seconds.map(|d| d as f32),
        // Motion verification metrics (MESHVER-005)
        hinge_axis_violations: result.metrics.hinge_axis_violations,
        range_violations: result.metrics.range_violations,
        velocity_spikes: result.metrics.velocity_spikes,
        root_motion_delta: result
            .metrics
            .root_motion_delta
            .map(|d| [d[0] as f32, d[1] as f32, d[2] as f32]),
    };

    Ok(vec![OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from(&primary_output.path),
        metrics,
    )])
}
