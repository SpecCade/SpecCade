//! Texture backend dispatch handler

use super::output_helpers::{
    get_metadata_outputs, get_primary_outputs, write_metadata_outputs, write_primary_png_outputs,
};
use super::{write_output_bytes, DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec, StageTiming};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Generate trimsheet atlas outputs using the texture backend.
pub(super) fn generate_texture_trimsheet(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_trimsheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture trimsheet params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Trimsheet generation failed: {}", e)))?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "texture.trimsheet_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "texture.trimsheet_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate trimsheet atlas outputs with profiling instrumentation.
pub(super) fn generate_texture_trimsheet_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_trimsheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture trimsheet params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: pack_and_render
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Trimsheet generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "texture.trimsheet_v1")?;
    let metadata_outputs = get_metadata_outputs(spec, "texture.trimsheet_v1")?;

    let mut outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate procedural texture outputs using the texture backend.
pub(super) fn generate_texture_procedural(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;

    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| DispatchError::BackendError(format!("PNG encoding failed: {}", e)))?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }

    Ok(outputs)
}

/// Generate procedural texture outputs with profiling instrumentation.
pub(super) fn generate_texture_procedural_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: render_graph
    let render_start = Instant::now();
    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "render_graph",
        render_start.elapsed().as_millis() as u64,
    ));

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    // Stage: encode_outputs
    let encode_start = Instant::now();
    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| DispatchError::BackendError(format!("PNG encoding failed: {}", e)))?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }
    stages.push(StageTiming::new(
        "encode_outputs",
        encode_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate decal texture outputs using the texture backend.
///
/// Decals output:
/// - RGBA albedo texture (primary, with alpha composited)
/// - Optional normal map (secondary)
/// - Optional roughness map (secondary)
/// - Metadata JSON sidecar (metadata)
pub(super) fn generate_texture_decal(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_texture_decal()
        .map_err(|e| DispatchError::BackendError(format!("Invalid texture decal params: {}", e)))?;

    let result = speccade_backend_texture::generate_decal(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Decal generation failed: {}", e)))?;
    write_texture_decal_outputs(spec, out_root, &result)
}

/// Generate decal texture outputs with profiling instrumentation.
pub(super) fn generate_texture_decal_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe
        .as_texture_decal()
        .map_err(|e| DispatchError::BackendError(format!("Invalid texture decal params: {}", e)))?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_decal
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_decal(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Decal generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "generate_decal",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let outputs = write_texture_decal_outputs(spec, out_root, &result)?;

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

fn write_texture_decal_outputs(
    spec: &Spec,
    out_root: &Path,
    result: &speccade_backend_texture::DecalResult,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe_kind = "texture.decal_v1";

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, recipe_kind)?;
    let mut outputs = Vec::new();
    let mut wrote_albedo = false;

    for (output_index, output_spec) in &primary_outputs {
        let source = output_spec.source.as_deref().unwrap_or("");
        match source {
            "" | "albedo" => {
                wrote_albedo = true;
                write_output_bytes(out_root, &output_spec.path, &result.albedo.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    result.albedo.hash.clone(),
                ));
            }
            "normal" => {
                let normal = result.normal.as_ref().ok_or_else(|| {
                    DispatchError::BackendError(format!(
                        "{} output requested normal map but recipe did not generate one (outputs[{}].source)",
                        recipe_kind, output_index
                    ))
                })?;
                write_output_bytes(out_root, &output_spec.path, &normal.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    normal.hash.clone(),
                ));
            }
            "roughness" => {
                let roughness = result.roughness.as_ref().ok_or_else(|| {
                    DispatchError::BackendError(format!(
                        "{} output requested roughness map but recipe did not generate one (outputs[{}].source)",
                        recipe_kind, output_index
                    ))
                })?;
                write_output_bytes(out_root, &output_spec.path, &roughness.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    roughness.hash.clone(),
                ));
            }
            other => {
                return Err(DispatchError::BackendError(format!(
                    "{} primary outputs have unknown source '{}' (outputs[{}].source); expected '', 'albedo', 'normal', or 'roughness'",
                    recipe_kind, other, output_index
                )));
            }
        }
    }

    if !wrote_albedo {
        return Err(DispatchError::BackendError(format!(
            "{} requires at least one albedo output (primary output with empty source or source 'albedo')",
            recipe_kind
        )));
    }

    let metadata_outputs = get_metadata_outputs(spec, recipe_kind)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}

/// Generate splat set texture outputs using the texture backend.
///
/// Splat sets output:
/// - Per-layer textures (albedo, normal, roughness PNGs)
/// - Splat mask textures (RGBA, up to 4 layers per mask)
/// - Optional macro variation texture
/// - Metadata JSON sidecar
pub(super) fn generate_texture_splat_set(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_splat_set().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture splat set params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_splat_set(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Splat set generation failed: {}", e)))?;

    let mut outputs = Vec::new();

    // Find primary outputs for layer textures
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    // Match primary outputs by source field or path patterns
    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.splat_set_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_deref().unwrap_or("");

        // Parse source like "grass.albedo", "grass.normal", "mask0", "macro"
        if source.contains('.') {
            // Layer texture (e.g., "grass.albedo")
            let parts: Vec<&str> = source.split('.').collect();
            if parts.len() == 2 {
                let layer_id = parts[0];
                let map_type = parts[1];

                if let Some(layer_output) = result.layer_outputs.iter().find(|l| l.id == layer_id) {
                    let (png_data, hash) = match map_type {
                        "albedo" => (&layer_output.albedo.png_data, &layer_output.albedo.hash),
                        "normal" => (&layer_output.normal.png_data, &layer_output.normal.hash),
                        "roughness" => (
                            &layer_output.roughness.png_data,
                            &layer_output.roughness.hash,
                        ),
                        _ => continue,
                    };

                    write_output_bytes(out_root, &output_spec.path, png_data)?;
                    outputs.push(OutputResult::tier1(
                        output_spec.kind,
                        OutputFormat::Png,
                        PathBuf::from(&output_spec.path),
                        hash.clone(),
                    ));
                }
            }
        } else if let Some(stripped) = source.strip_prefix("mask") {
            // Splat mask (e.g., "mask0", "mask1")
            if let Ok(mask_idx) = stripped.parse::<usize>() {
                if mask_idx < result.splat_masks.len() {
                    let mask = &result.splat_masks[mask_idx];
                    write_output_bytes(out_root, &output_spec.path, &mask.png_data)?;
                    outputs.push(OutputResult::tier1(
                        output_spec.kind,
                        OutputFormat::Png,
                        PathBuf::from(&output_spec.path),
                        mask.hash.clone(),
                    ));
                }
            }
        } else if source == "macro" {
            // Macro variation texture
            if let Some(ref macro_tex) = result.macro_variation {
                write_output_bytes(out_root, &output_spec.path, &macro_tex.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    macro_tex.hash.clone(),
                ));
            }
        }
    }

    // Find metadata output (JSON sidecar)
    let metadata_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Metadata)
        .collect();

    for (output_index, output_spec) in &metadata_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "texture.splat_set_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        let metadata_hash = blake3::hash(metadata_json.as_bytes()).to_hex().to_string();

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Json,
            PathBuf::from(&output_spec.path),
            metadata_hash,
        ));
    }

    Ok(outputs)
}

/// Generate splat set texture outputs with profiling instrumentation.
pub(super) fn generate_texture_splat_set_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_splat_set().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture splat set params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_splat_set
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_splat_set(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Splat set generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "generate_splat_set",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary outputs for layer textures
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.splat_set_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_deref().unwrap_or("");

        if source.contains('.') {
            let parts: Vec<&str> = source.split('.').collect();
            if parts.len() == 2 {
                let layer_id = parts[0];
                let map_type = parts[1];

                if let Some(layer_output) = result.layer_outputs.iter().find(|l| l.id == layer_id) {
                    let (png_data, hash) = match map_type {
                        "albedo" => (&layer_output.albedo.png_data, &layer_output.albedo.hash),
                        "normal" => (&layer_output.normal.png_data, &layer_output.normal.hash),
                        "roughness" => (
                            &layer_output.roughness.png_data,
                            &layer_output.roughness.hash,
                        ),
                        _ => continue,
                    };

                    write_output_bytes(out_root, &output_spec.path, png_data)?;
                    outputs.push(OutputResult::tier1(
                        output_spec.kind,
                        OutputFormat::Png,
                        PathBuf::from(&output_spec.path),
                        hash.clone(),
                    ));
                }
            }
        } else if let Some(stripped) = source.strip_prefix("mask") {
            if let Ok(mask_idx) = stripped.parse::<usize>() {
                if mask_idx < result.splat_masks.len() {
                    let mask = &result.splat_masks[mask_idx];
                    write_output_bytes(out_root, &output_spec.path, &mask.png_data)?;
                    outputs.push(OutputResult::tier1(
                        output_spec.kind,
                        OutputFormat::Png,
                        PathBuf::from(&output_spec.path),
                        mask.hash.clone(),
                    ));
                }
            }
        } else if source == "macro" {
            if let Some(ref macro_tex) = result.macro_variation {
                write_output_bytes(out_root, &output_spec.path, &macro_tex.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    macro_tex.hash.clone(),
                ));
            }
        }
    }

    // Find metadata output
    let metadata_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Metadata)
        .collect();

    for (output_index, output_spec) in &metadata_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "texture.splat_set_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        let metadata_hash = blake3::hash(metadata_json.as_bytes()).to_hex().to_string();

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Json,
            PathBuf::from(&output_spec.path),
            metadata_hash,
        ));
    }

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate matcap texture outputs using the texture backend.
pub(super) fn generate_texture_matcap(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_matcap().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture matcap params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_matcap(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Matcap generation failed: {}", e)))?;

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "texture.matcap_v1")?;

    write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)
}

/// Generate matcap texture outputs with profiling instrumentation.
pub(super) fn generate_texture_matcap_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_matcap().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture matcap params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_matcap
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_matcap(&params, spec.seed)
        .map_err(|e| DispatchError::BackendError(format!("Matcap generation failed: {}", e)))?;
    stages.push(StageTiming::new(
        "generate_matcap",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, "texture.matcap_v1")?;

    let outputs =
        write_primary_png_outputs(out_root, &primary_outputs, &result.png_data, &result.hash)?;

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

/// Generate material preset texture outputs using the texture backend.
///
/// Material presets output:
/// - Albedo texture (primary PNG)
/// - Roughness texture (primary PNG)
/// - Metallic texture (primary PNG)
/// - Normal map texture (primary PNG)
/// - Metadata JSON sidecar (metadata)
pub(super) fn generate_texture_material_preset(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_material_preset().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture material preset params: {}", e))
    })?;

    let result =
        speccade_backend_texture::material_preset::generate_material_preset(&params, spec.seed)
            .map_err(|e| {
                DispatchError::BackendError(format!("Material preset generation failed: {}", e))
            })?;

    write_texture_material_preset_outputs(spec, out_root, &result)
}

/// Generate material preset texture outputs with profiling instrumentation.
pub(super) fn generate_texture_material_preset_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_material_preset().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture material preset params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_material_preset
    let render_start = Instant::now();
    let result =
        speccade_backend_texture::material_preset::generate_material_preset(&params, spec.seed)
            .map_err(|e| {
                DispatchError::BackendError(format!("Material preset generation failed: {}", e))
            })?;
    stages.push(StageTiming::new(
        "generate_material_preset",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let outputs = write_texture_material_preset_outputs(spec, out_root, &result)?;

    stages.push(StageTiming::new(
        "write_outputs",
        write_start.elapsed().as_millis() as u64,
    ));

    Ok(DispatchResult::with_stages(outputs, stages))
}

fn write_texture_material_preset_outputs(
    spec: &Spec,
    out_root: &Path,
    result: &speccade_backend_texture::material_preset::MaterialPresetResult,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe_kind = "texture.material_preset_v1";

    let primary_outputs = get_primary_outputs(spec, OutputFormat::Png, recipe_kind)?;
    let mut outputs = Vec::new();

    for (output_index, output_spec) in &primary_outputs {
        let source = output_spec.source.as_deref().unwrap_or("");
        match source {
            "albedo" => {
                write_output_bytes(out_root, &output_spec.path, &result.albedo.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    result.albedo.hash.clone(),
                ));
            }
            "roughness" => {
                write_output_bytes(out_root, &output_spec.path, &result.roughness.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    result.roughness.hash.clone(),
                ));
            }
            "metallic" => {
                write_output_bytes(out_root, &output_spec.path, &result.metallic.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    result.metallic.hash.clone(),
                ));
            }
            "normal" => {
                write_output_bytes(out_root, &output_spec.path, &result.normal.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    result.normal.hash.clone(),
                ));
            }
            other => {
                return Err(DispatchError::BackendError(format!(
                    "{} primary output has unknown source '{}' (outputs[{}].source); expected 'albedo', 'roughness', 'metallic', or 'normal'",
                    recipe_kind, other, output_index
                )));
            }
        }
    }

    let metadata_outputs = get_metadata_outputs(spec, recipe_kind)?;
    outputs.extend(write_metadata_outputs(
        out_root,
        &metadata_outputs,
        &result.metadata,
    )?);

    Ok(outputs)
}
