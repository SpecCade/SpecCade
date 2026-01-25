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
    let lod_levels = result.metrics.lod_levels.clone().map(|levels| {
        levels
            .into_iter()
            .map(|level| speccade_spec::StaticMeshLodLevelMetrics {
                lod_level: level.lod_level,
                target_tris: level.target_tris,
                simplification_ratio: level.simplification_ratio,
                vertex_count: level.vertex_count,
                face_count: level.face_count,
                edge_count: level.edge_count,
                triangle_count: level.triangle_count,
                quad_count: level.quad_count,
                quad_percentage: level.quad_percentage,
                manifold: level.manifold,
                non_manifold_edge_count: level.non_manifold_edge_count,
                degenerate_face_count: level.degenerate_face_count,
                zero_area_face_count: level.zero_area_face_count,
                uv_island_count: level.uv_island_count,
                uv_coverage: level.uv_coverage,
                uv_overlap_percentage: level.uv_overlap_percentage,
                has_uv_map: level.has_uv_map,
                uv_layer_count: level.uv_layer_count,
                texel_density: level.texel_density,
                bounding_box: level
                    .bounding_box
                    .as_ref()
                    .map(|bb| speccade_spec::BoundingBox {
                        min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                        max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                    }),
                bounds_min: level.bounds_min,
                bounds_max: level.bounds_max,
                material_slot_count: level.material_slot_count,
            })
            .collect()
    });

    let collision_mesh =
        result
            .metrics
            .collision_mesh
            .clone()
            .map(|m| speccade_spec::CollisionMeshMetrics {
                vertex_count: m.vertex_count,
                face_count: m.face_count,
                triangle_count: m.triangle_count,
                bounding_box: speccade_spec::CollisionBoundingBox {
                    min: m.bounding_box.min,
                    max: m.bounding_box.max,
                },
                collision_type: m.collision_type,
            });

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
            zero_area_face_count: result.metrics.zero_area_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            has_uv_map: result.metrics.has_uv_map,
            uv_layer_count: result.metrics.uv_layer_count,
            texel_density: result.metrics.texel_density,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            lod_count: result.metrics.lod_count,
            lod_levels,
            collision_mesh,
            collision_mesh_path: result.metrics.collision_mesh_path.clone(),
            navmesh: result
                .metrics
                .navmesh
                .clone()
                .map(|m| speccade_spec::NavmeshMetrics {
                    walkable_face_count: m.walkable_face_count,
                    non_walkable_face_count: m.non_walkable_face_count,
                    walkable_percentage: m.walkable_percentage,
                    stair_candidates: m.stair_candidates,
                }),
            baking: result
                .metrics
                .baking
                .clone()
                .map(|b| speccade_spec::BakingMetrics {
                    baked_maps: b
                        .baked_maps
                        .into_iter()
                        .map(|m| speccade_spec::BakedMapInfo {
                            bake_type: m.bake_type,
                            path: m.path,
                            resolution: m.resolution,
                        })
                        .collect(),
                    ray_distance: b.ray_distance,
                    margin: b.margin,
                }),
            bone_count: None,
            max_bone_influences: None,
            unweighted_vertex_count: None,
            weight_normalization_percentage: None,
            max_weight_deviation: None,
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
            zero_area_face_count: result.metrics.zero_area_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            has_uv_map: result.metrics.has_uv_map,
            uv_layer_count: result.metrics.uv_layer_count,
            texel_density: result.metrics.texel_density,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            lod_count: None,
            lod_levels: None,
            collision_mesh: None,
            collision_mesh_path: None,
            navmesh: None,
            baking: None,
            bone_count: result.metrics.bone_count,
            max_bone_influences: result.metrics.max_bone_influences,
            unweighted_vertex_count: result.metrics.unweighted_vertex_count,
            weight_normalization_percentage: result.metrics.weight_normalization_percentage,
            max_weight_deviation: result.metrics.max_weight_deviation,
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
        zero_area_face_count: None,
        uv_island_count: None,
        uv_coverage: None,
        uv_overlap_percentage: None,
        has_uv_map: None,
        uv_layer_count: None,
        texel_density: None,
        bounding_box: None,
        bounds_min: None,
        bounds_max: None,
        lod_count: None,
        lod_levels: None,
        collision_mesh: None,
        collision_mesh_path: None,
        navmesh: None,
        baking: None,
        bone_count: result.metrics.bone_count,
        max_bone_influences: None,
        unweighted_vertex_count: None,
        weight_normalization_percentage: None,
        max_weight_deviation: None,
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

/// Generate sprite from mesh using the Blender backend
pub(super) fn generate_blender_sprite_from_mesh(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result =
        speccade_backend_blender::mesh_to_sprite::generate(spec, out_root).map_err(|e| {
            DispatchError::BackendError(format!("Mesh-to-sprite generation failed: {}", e))
        })?;

    // Get primary output (atlas PNG)
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Png {
        return Err(DispatchError::BackendError(format!(
            "sprite.render_from_mesh_v1 requires primary output format 'png', got '{}'",
            primary_output.format
        )));
    }

    let mut outputs = vec![];

    // Primary atlas output
    let primary_metrics = speccade_spec::OutputMetrics::default();
    outputs.push(OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Png,
        PathBuf::from(&primary_output.path),
        primary_metrics,
    ));

    // Metadata output if specified
    if let Some(metadata_output) = spec.outputs.iter().find(|o| o.kind == OutputKind::Metadata) {
        if metadata_output.format == OutputFormat::Json {
            let metadata_metrics = speccade_spec::OutputMetrics::default();
            outputs.push(OutputResult::tier2(
                OutputKind::Metadata,
                OutputFormat::Json,
                PathBuf::from(&metadata_output.path),
                metadata_metrics,
            ));
        }
    }

    // Log generation metrics for debugging
    if let Some(atlas_dims) = result.metrics.atlas_dimensions {
        eprintln!(
            "[mesh_to_sprite] Generated sprite atlas: {}x{} with {} frames",
            atlas_dims[0],
            atlas_dims[1],
            result.metrics.frame_count.unwrap_or(0)
        );
    }

    Ok(outputs)
}

/// Generate modular kit mesh using the Blender backend
pub(super) fn generate_blender_modular_kit(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result = speccade_backend_blender::modular_kit::generate(spec, out_root).map_err(|e| {
        DispatchError::BackendError(format!("Modular kit generation failed: {}", e))
    })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "static_mesh.modular_kit_v1 requires primary output format 'glb', got '{}'",
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
            zero_area_face_count: result.metrics.zero_area_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            has_uv_map: result.metrics.has_uv_map,
            uv_layer_count: result.metrics.uv_layer_count,
            texel_density: result.metrics.texel_density,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            lod_count: None,
            lod_levels: None,
            collision_mesh: None,
            collision_mesh_path: None,
            navmesh: None,
            baking: None,
            bone_count: None,
            max_bone_influences: None,
            unweighted_vertex_count: None,
            weight_normalization_percentage: None,
            max_weight_deviation: None,
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

/// Generate organic sculpt mesh using the Blender backend
pub(super) fn generate_blender_organic_sculpt(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result =
        speccade_backend_blender::organic_sculpt::generate(spec, out_root).map_err(|e| {
            DispatchError::BackendError(format!("Organic sculpt generation failed: {}", e))
        })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "static_mesh.organic_sculpt_v1 requires primary output format 'glb', got '{}'",
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
            zero_area_face_count: result.metrics.zero_area_face_count,
            uv_island_count: result.metrics.uv_island_count,
            uv_coverage: result.metrics.uv_coverage,
            uv_overlap_percentage: result.metrics.uv_overlap_percentage,
            has_uv_map: result.metrics.has_uv_map,
            uv_layer_count: result.metrics.uv_layer_count,
            texel_density: result.metrics.texel_density,
            bounding_box: result.metrics.bounding_box.as_ref().map(|bb| {
                speccade_spec::BoundingBox {
                    min: [bb.min[0] as f32, bb.min[1] as f32, bb.min[2] as f32],
                    max: [bb.max[0] as f32, bb.max[1] as f32, bb.max[2] as f32],
                }
            }),
            bounds_min: result.metrics.bounds_min,
            bounds_max: result.metrics.bounds_max,
            lod_count: None,
            lod_levels: None,
            collision_mesh: None,
            collision_mesh_path: None,
            navmesh: None,
            baking: None,
            bone_count: None,
            max_bone_influences: None,
            unweighted_vertex_count: None,
            weight_normalization_percentage: None,
            max_weight_deviation: None,
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

/// Generate rigged animation using the Blender backend
pub(super) fn generate_blender_rigged_animation(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let result =
        speccade_backend_blender::rigged_animation::generate(spec, out_root).map_err(|e| {
            DispatchError::BackendError(format!("Rigged animation generation failed: {}", e))
        })?;

    // Get primary output path
    let primary_output = spec
        .outputs
        .iter()
        .find(|o| o.kind == OutputKind::Primary)
        .ok_or_else(|| DispatchError::BackendError("No primary output specified".to_string()))?;
    if primary_output.format != OutputFormat::Glb {
        return Err(DispatchError::BackendError(format!(
            "skeletal_animation.blender_rigged_v1 requires primary output format 'glb', got '{}'",
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
        zero_area_face_count: None,
        uv_island_count: None,
        uv_coverage: None,
        uv_overlap_percentage: None,
        has_uv_map: None,
        uv_layer_count: None,
        texel_density: None,
        bounding_box: None,
        bounds_min: None,
        bounds_max: None,
        lod_count: None,
        lod_levels: None,
        collision_mesh: None,
        collision_mesh_path: None,
        navmesh: None,
        baking: None,
        bone_count: result.metrics.bone_count,
        max_bone_influences: None,
        unweighted_vertex_count: None,
        weight_normalization_percentage: None,
        max_weight_deviation: None,
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
