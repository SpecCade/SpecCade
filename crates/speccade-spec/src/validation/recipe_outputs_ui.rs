//! UI and font recipe output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;

use super::recipe_outputs::validate_primary_output_present;

/// Validates outputs for `ui.nine_slice_v1` recipe.
pub(super) fn validate_ui_nine_slice_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_ui_nine_slice() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.nine_slice_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.nine_slice_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `ui.icon_set_v1` recipe.
pub(super) fn validate_ui_icon_set_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_ui_icon_set() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.icon_set_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.icon_set_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `ui.item_card_v1` recipe.
pub(super) fn validate_ui_item_card_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_ui_item_card() {
        Ok(params) => {
            if params.resolution[0] < 32 || params.resolution[1] < 32 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be at least 32x32, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if params.resolution[0] > 4096 || params.resolution[1] > 4096 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be at most 4096x4096, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if params.rarity_presets.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "at least one rarity_preset must be defined",
                    "recipe.params.rarity_presets",
                ));
            }
            let mut seen_tiers = std::collections::HashSet::new();
            for (i, preset) in params.rarity_presets.iter().enumerate() {
                if !seen_tiers.insert(preset.tier) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate rarity tier: '{}'", preset.tier),
                        format!("recipe.params.rarity_presets[{}].tier", i),
                    ));
                }
                for (j, &c) in preset.border_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("border_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.rarity_presets[{}].border_color[{}]", i, j),
                        ));
                    }
                }
                for (j, &c) in preset.background_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("background_color[{}] must be in [0, 1], got {}", j, c),
                            format!(
                                "recipe.params.rarity_presets[{}].background_color[{}]",
                                i, j
                            ),
                        ));
                    }
                }
                if let Some(ref glow) = preset.glow_color {
                    for (j, &c) in glow.iter().enumerate() {
                        if !(0.0..=1.0).contains(&c) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!("glow_color[{}] must be in [0, 1], got {}", j, c),
                                format!("recipe.params.rarity_presets[{}].glow_color[{}]", i, j),
                            ));
                        }
                    }
                }
            }
            let card_w = params.resolution[0];
            let card_h = params.resolution[1];
            let mut validate_slot = |name: &str, region: &[u32; 4], path: &str| {
                let end_x = region[0] + region[2];
                let end_y = region[1] + region[3];
                if end_x > card_w || end_y > card_h {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "{} slot region extends beyond card bounds: ends at ({}, {}) but card is ({}x{})",
                            name, end_x, end_y, card_w, card_h
                        ),
                        path,
                    ));
                }
            };
            validate_slot(
                "icon",
                &params.slots.icon_region,
                "recipe.params.slots.icon_region",
            );
            validate_slot(
                "rarity_indicator",
                &params.slots.rarity_indicator_region,
                "recipe.params.slots.rarity_indicator_region",
            );
            validate_slot(
                "background",
                &params.slots.background_region,
                "recipe.params.slots.background_region",
            );
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.item_card_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.item_card_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `ui.damage_number_v1` recipe.
pub(super) fn validate_ui_damage_number_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_ui_damage_number() {
        Ok(params) => {
            if params.glyph_size[0] < 8 || params.glyph_size[1] < 8 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "glyph_size must be at least 8x8, got [{}, {}]",
                        params.glyph_size[0], params.glyph_size[1]
                    ),
                    "recipe.params.glyph_size",
                ));
            }
            if params.glyph_size[0] > 128 || params.glyph_size[1] > 128 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "glyph_size must be at most 128x128, got [{}, {}]",
                        params.glyph_size[0], params.glyph_size[1]
                    ),
                    "recipe.params.glyph_size",
                ));
            }
            if params.outline_width < 1 || params.outline_width > 8 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "outline_width must be between 1 and 8, got {}",
                        params.outline_width
                    ),
                    "recipe.params.outline_width",
                ));
            }
            if params.styles.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "at least one style must be defined",
                    "recipe.params.styles",
                ));
            }
            let mut seen_types = std::collections::HashSet::new();
            for (i, style) in params.styles.iter().enumerate() {
                if !seen_types.insert(&style.style_type) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate style_type: '{}'", style.style_type),
                        format!("recipe.params.styles[{}].style_type", i),
                    ));
                }
                for (j, &c) in style.text_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("text_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.styles[{}].text_color[{}]", i, j),
                        ));
                    }
                }
                for (j, &c) in style.outline_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("outline_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.styles[{}].outline_color[{}]", i, j),
                        ));
                    }
                }
                if let Some(ref glow) = style.glow_color {
                    for (j, &c) in glow.iter().enumerate() {
                        if !(0.0..=1.0).contains(&c) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!("glow_color[{}] must be in [0, 1], got {}", j, c),
                                format!("recipe.params.styles[{}].glow_color[{}]", i, j),
                            ));
                        }
                    }
                }
                if let Some(scale) = style.scale {
                    if !(0.5..=2.0).contains(&scale) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("scale must be between 0.5 and 2.0, got {}", scale),
                            format!("recipe.params.styles[{}].scale", i),
                        ));
                    }
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.damage_number_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.damage_number_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `font.bitmap_v1` recipe.
pub(super) fn validate_font_bitmap_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_font_bitmap() {
        Ok(params) => {
            if params.charset[0] > params.charset[1] {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "charset start must be <= end, got [{}, {}]",
                        params.charset[0], params.charset[1]
                    ),
                    "recipe.params.charset",
                ));
            }
            match (params.glyph_size[0], params.glyph_size[1]) {
                (5, 7) | (8, 8) | (6, 9) => {}
                (w, h) => {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "unsupported glyph_size [{}, {}]; supported sizes: [5,7], [8,8], [6,9]",
                            w, h
                        ),
                        "recipe.params.glyph_size",
                    ));
                }
            }
            for (idx, c) in params.color.iter().enumerate() {
                if !(0.0..=1.0).contains(c) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("color[{}] must be in [0,1], got {}", idx, c),
                        format!("recipe.params.color[{}]", idx),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "font.bitmap_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "font.bitmap_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}
