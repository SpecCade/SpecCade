//! Texture backend dispatch handler

use super::{write_output_bytes, DispatchError, DispatchResult};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec, StageTiming};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Generate spritesheet atlas outputs using the texture backend.
pub(super) fn generate_sprite_sheet(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_sheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite sheet params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_sprite_sheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Sprite sheet generation failed: {}", e))
    })?;

    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "sprite.sheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "sprite.sheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
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
                "sprite.sheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate spritesheet atlas outputs with profiling instrumentation.
pub(super) fn generate_sprite_sheet_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_sheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite sheet params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: pack_and_render
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_sprite_sheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Sprite sheet generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "sprite.sheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "sprite.sheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
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
                "sprite.sheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate sprite animation metadata.
pub(super) fn generate_sprite_animation(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_animation().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite animation params: {}", e))
    })?;

    // Generate metadata
    let metadata = params.to_metadata();

    let mut outputs = Vec::new();

    // Find primary output for the animation JSON
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "sprite.animation_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "sprite.animation_v1 primary outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize animation metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate sprite animation metadata with profiling instrumentation.
pub(super) fn generate_sprite_animation_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_sprite_animation().map_err(|e| {
        DispatchError::BackendError(format!("Invalid sprite animation params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_metadata
    let gen_start = Instant::now();
    let metadata = params.to_metadata();
    stages.push(StageTiming::new(
        "generate_metadata",
        gen_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary output for the animation JSON
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "sprite.animation_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Json {
            return Err(DispatchError::BackendError(format!(
                "sprite.animation_v1 primary outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize animation metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate trimsheet atlas outputs using the texture backend.
pub(super) fn generate_texture_trimsheet(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_trimsheet().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture trimsheet params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Trimsheet generation failed: {}", e))
    })?;

    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.trimsheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
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
                "texture.trimsheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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
    let result = speccade_backend_texture::generate_trimsheet(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Trimsheet generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "pack_and_render",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary output for the atlas PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.trimsheet_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.trimsheet_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.hash.clone(),
        ));
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
                "texture.trimsheet_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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
    let params = recipe.as_texture_decal().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture decal params: {}", e))
    })?;

    let result = speccade_backend_texture::generate_decal(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Decal generation failed: {}", e))
    })?;

    let mut outputs = Vec::new();

    // Find primary output for the albedo PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.decal_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.decal_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.albedo.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.albedo.hash.clone(),
        ));
    }

    // Find additional primary outputs for normal and roughness maps (using source field)
    for (output_index, output_spec) in spec.outputs.iter().enumerate() {
        // Skip the first primary output (albedo) and metadata
        if output_spec.kind != OutputKind::Primary {
            continue;
        }
        // Use source field to identify normal/roughness outputs
        let source = output_spec.source.as_deref().unwrap_or("");
        if source.is_empty() && !output_spec.path.contains("normal") && !output_spec.path.contains("roughness") {
            // This is the main albedo output, already handled above
            continue;
        }

        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.decal_v1 outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        if source == "normal" || output_spec.path.contains("normal") {
            if let Some(ref normal) = result.normal {
                write_output_bytes(out_root, &output_spec.path, &normal.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    normal.hash.clone(),
                ));
            }
        } else if source == "roughness" || output_spec.path.contains("roughness") {
            if let Some(ref roughness) = result.roughness {
                write_output_bytes(out_root, &output_spec.path, &roughness.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    roughness.hash.clone(),
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
                "texture.decal_v1 metadata outputs must have format 'json' (outputs[{}].format)",
                output_index
            )));
        }

        let metadata_json = serde_json::to_string_pretty(&result.metadata).map_err(|e| {
            DispatchError::BackendError(format!("Failed to serialize metadata: {}", e))
        })?;

        write_output_bytes(out_root, &output_spec.path, metadata_json.as_bytes())?;

        // Calculate hash of the JSON metadata
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

/// Generate decal texture outputs with profiling instrumentation.
pub(super) fn generate_texture_decal_profiled(
    spec: &Spec,
    out_root: &Path,
) -> Result<DispatchResult, DispatchError> {
    let mut stages = Vec::new();

    // Stage: parse_params
    let parse_start = Instant::now();
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_decal().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture decal params: {}", e))
    })?;
    stages.push(StageTiming::new(
        "parse_params",
        parse_start.elapsed().as_millis() as u64,
    ));

    // Stage: generate_decal
    let render_start = Instant::now();
    let result = speccade_backend_texture::generate_decal(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Decal generation failed: {}", e))
    })?;
    stages.push(StageTiming::new(
        "generate_decal",
        render_start.elapsed().as_millis() as u64,
    ));

    // Stage: write_outputs
    let write_start = Instant::now();
    let mut outputs = Vec::new();

    // Find primary output for the albedo PNG
    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.decal_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    for (output_index, output_spec) in &primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.decal_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        write_output_bytes(out_root, &output_spec.path, &result.albedo.png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            result.albedo.hash.clone(),
        ));
    }

    // Find additional primary outputs for normal and roughness maps (using source field)
    for (output_index, output_spec) in spec.outputs.iter().enumerate() {
        if output_spec.kind != OutputKind::Primary {
            continue;
        }
        let source = output_spec.source.as_deref().unwrap_or("");
        if source.is_empty() && !output_spec.path.contains("normal") && !output_spec.path.contains("roughness") {
            continue;
        }

        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.decal_v1 outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        if source == "normal" || output_spec.path.contains("normal") {
            if let Some(ref normal) = result.normal {
                write_output_bytes(out_root, &output_spec.path, &normal.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    normal.hash.clone(),
                ));
            }
        } else if source == "roughness" || output_spec.path.contains("roughness") {
            if let Some(ref roughness) = result.roughness {
                write_output_bytes(out_root, &output_spec.path, &roughness.png_data)?;
                outputs.push(OutputResult::tier1(
                    output_spec.kind,
                    OutputFormat::Png,
                    PathBuf::from(&output_spec.path),
                    roughness.hash.clone(),
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
                "texture.decal_v1 metadata outputs must have format 'json' (outputs[{}].format)",
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

    let result = speccade_backend_texture::generate_splat_set(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Splat set generation failed: {}", e))
    })?;

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
                        "roughness" => (&layer_output.roughness.png_data, &layer_output.roughness.hash),
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
        } else if source.starts_with("mask") {
            // Splat mask (e.g., "mask0", "mask1")
            if let Ok(mask_idx) = source[4..].parse::<usize>() {
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
    let result = speccade_backend_texture::generate_splat_set(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Splat set generation failed: {}", e))
    })?;
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
                        "roughness" => (&layer_output.roughness.png_data, &layer_output.roughness.hash),
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
        } else if source.starts_with("mask") {
            if let Ok(mask_idx) = source[4..].parse::<usize>() {
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
