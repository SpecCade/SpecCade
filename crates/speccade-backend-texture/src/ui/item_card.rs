//! Item card generation with deterministic atlas packing.
//!
//! This module implements an item card generator that packs multiple rarity
//! variants into an atlas using a deterministic horizontal shelf layout.

use speccade_spec::recipe::ui::{
    ItemCardMetadata, ItemCardSlotRegions, ItemCardUv, ItemCardVariant, SlotRegion,
    UiItemCardV1Params,
};
use thiserror::Error;

use super::gutter::validate_color;
use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during item card generation.
#[derive(Debug, Error)]
pub enum ItemCardError {
    /// No rarity presets defined.
    #[error("At least one rarity_preset must be defined")]
    NoRarityPresets,

    /// Card is too large to fit in atlas.
    #[error("Card resolution ({0}x{1}) with padding is too large for computed atlas")]
    CardTooLarge(u32, u32),

    /// Duplicate rarity tier.
    #[error("Duplicate rarity tier: '{0}'")]
    DuplicateRarityTier(String),

    /// Invalid resolution.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Invalid color value.
    #[error("{0}")]
    InvalidColor(String),

    /// Slot region extends beyond card bounds.
    #[error("Slot region '{0}' extends beyond card bounds: region ends at ({1}, {2}) but card is ({3}x{4})")]
    SlotOutOfBounds(String, u32, u32, u32, u32),

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),
}

/// Result of item card generation.
#[derive(Debug)]
pub struct ItemCardResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates and slot info for each variant.
    pub metadata: ItemCardMetadata,
}

/// Generate an item card atlas from parameters.
///
/// # Arguments
/// * `params` - Item card parameters including resolution, rarity presets, and slots
/// * `_seed` - Deterministic seed (reserved for future procedural effects)
///
/// # Returns
/// An `ItemCardResult` containing the PNG data, hash, and variant metadata.
pub fn generate_item_card(
    params: &UiItemCardV1Params,
    _seed: u32,
) -> Result<ItemCardResult, ItemCardError> {
    let card_width = params.resolution[0];
    let card_height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if card_width == 0 || card_height == 0 {
        return Err(ItemCardError::InvalidParameter(
            "Card resolution must be non-zero".to_string(),
        ));
    }

    if card_width < 32 || card_height < 32 {
        return Err(ItemCardError::InvalidParameter(format!(
            "Card resolution must be at least 32x32, got {}x{}",
            card_width, card_height
        )));
    }

    if card_width > 4096 || card_height > 4096 {
        return Err(ItemCardError::InvalidParameter(format!(
            "Card resolution must be at most 4096x4096, got {}x{}",
            card_width, card_height
        )));
    }

    // Validate at least one rarity preset
    if params.rarity_presets.is_empty() {
        return Err(ItemCardError::NoRarityPresets);
    }

    // Validate slot regions are within card bounds
    validate_slot_region("icon", &params.slots.icon_region, card_width, card_height)?;
    validate_slot_region(
        "rarity_indicator",
        &params.slots.rarity_indicator_region,
        card_width,
        card_height,
    )?;
    validate_slot_region(
        "background",
        &params.slots.background_region,
        card_width,
        card_height,
    )?;

    // Validate colors and check for duplicate tiers
    let mut seen_tiers = std::collections::HashSet::new();
    for preset in &params.rarity_presets {
        if !seen_tiers.insert(preset.tier) {
            return Err(ItemCardError::DuplicateRarityTier(
                preset.tier.as_str().to_string(),
            ));
        }

        validate_color(&preset.border_color, &format!("{} border", preset.tier))
            .map_err(ItemCardError::InvalidColor)?;
        validate_color(
            &preset.background_color,
            &format!("{} background", preset.tier),
        )
        .map_err(ItemCardError::InvalidColor)?;

        if let Some(glow) = &preset.glow_color {
            validate_color(glow, &format!("{} glow", preset.tier))
                .map_err(ItemCardError::InvalidColor)?;
        }
    }

    // Sort rarity presets by tier for deterministic output
    let mut sorted_presets = params.rarity_presets.clone();
    sorted_presets.sort_by_key(|p| p.tier.sort_order());

    // Calculate atlas dimensions
    // Pack variants horizontally in a single row
    let variant_count = sorted_presets.len() as u32;
    let padded_card_width = card_width + padding * 2;
    let padded_card_height = card_height + padding * 2;

    let atlas_width = padded_card_width * variant_count;
    let atlas_height = padded_card_height;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(atlas_width, atlas_height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each variant
    let mut variants = Vec::with_capacity(sorted_presets.len());

    for (i, preset) in sorted_presets.iter().enumerate() {
        let card_x = (i as u32) * padded_card_width + padding;
        let card_y = padding;

        // Render card for this rarity preset
        render_item_card(
            &mut atlas,
            card_x,
            card_y,
            card_width,
            card_height,
            params,
            preset,
        );

        // Build variant metadata
        let uv = ItemCardUv::from_pixels(
            card_x,
            card_y,
            card_width,
            card_height,
            atlas_width,
            atlas_height,
        );

        let slots = ItemCardSlotRegions {
            icon: SlotRegion::from_array(params.slots.icon_region),
            rarity_indicator: SlotRegion::from_array(params.slots.rarity_indicator_region),
            background: SlotRegion::from_array(params.slots.background_region),
        };

        variants.push(ItemCardVariant {
            tier: preset.tier.as_str().to_string(),
            uv,
            slots,
        });
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    let metadata = ItemCardMetadata {
        atlas_width,
        atlas_height,
        padding,
        card_width,
        card_height,
        variants,
    };

    Ok(ItemCardResult {
        png_data,
        hash,
        metadata,
    })
}

/// Validates that a slot region is within card bounds.
fn validate_slot_region(
    name: &str,
    region: &[u32; 4],
    card_width: u32,
    card_height: u32,
) -> Result<(), ItemCardError> {
    let end_x = region[0] + region[2];
    let end_y = region[1] + region[3];

    if end_x > card_width || end_y > card_height {
        return Err(ItemCardError::SlotOutOfBounds(
            name.to_string(),
            end_x,
            end_y,
            card_width,
            card_height,
        ));
    }

    Ok(())
}

/// Renders a single item card variant into the atlas.
fn render_item_card(
    atlas: &mut TextureBuffer,
    card_x: u32,
    card_y: u32,
    card_width: u32,
    card_height: u32,
    params: &UiItemCardV1Params,
    preset: &speccade_spec::recipe::ui::RarityPreset,
) {
    let border_width = params.border_width;

    // 1. Render background fill (the entire card area)
    let bg_color = Color::rgba(
        preset.background_color[0],
        preset.background_color[1],
        preset.background_color[2],
        preset.background_color[3],
    );

    for dy in 0..card_height {
        for dx in 0..card_width {
            let px = card_x + dx;
            let py = card_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, bg_color);
            }
        }
    }

    // 2. Render glow if present (as an outer border effect)
    if let Some(glow) = &preset.glow_color {
        let glow_color = Color::rgba(glow[0], glow[1], glow[2], glow[3]);
        let glow_width = border_width + 2; // Glow extends beyond border

        // Top glow
        for dy in 0..glow_width.min(card_height) {
            for dx in 0..card_width {
                let px = card_x + dx;
                let py = card_y + dy;
                if px < atlas.width && py < atlas.height {
                    let existing = atlas.get(px, py);
                    atlas.set(px, py, blend_over(glow_color, existing));
                }
            }
        }

        // Bottom glow
        for dy in card_height.saturating_sub(glow_width)..card_height {
            for dx in 0..card_width {
                let px = card_x + dx;
                let py = card_y + dy;
                if px < atlas.width && py < atlas.height {
                    let existing = atlas.get(px, py);
                    atlas.set(px, py, blend_over(glow_color, existing));
                }
            }
        }

        // Left glow
        for dy in 0..card_height {
            for dx in 0..glow_width.min(card_width) {
                let px = card_x + dx;
                let py = card_y + dy;
                if px < atlas.width && py < atlas.height {
                    let existing = atlas.get(px, py);
                    atlas.set(px, py, blend_over(glow_color, existing));
                }
            }
        }

        // Right glow
        for dy in 0..card_height {
            for dx in card_width.saturating_sub(glow_width)..card_width {
                let px = card_x + dx;
                let py = card_y + dy;
                if px < atlas.width && py < atlas.height {
                    let existing = atlas.get(px, py);
                    atlas.set(px, py, blend_over(glow_color, existing));
                }
            }
        }
    }

    // 3. Render border
    let border_color = Color::rgba(
        preset.border_color[0],
        preset.border_color[1],
        preset.border_color[2],
        preset.border_color[3],
    );

    // Top border
    for dy in 0..border_width.min(card_height) {
        for dx in 0..card_width {
            let px = card_x + dx;
            let py = card_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, border_color);
            }
        }
    }

    // Bottom border
    for dy in card_height.saturating_sub(border_width)..card_height {
        for dx in 0..card_width {
            let px = card_x + dx;
            let py = card_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, border_color);
            }
        }
    }

    // Left border
    for dy in 0..card_height {
        for dx in 0..border_width.min(card_width) {
            let px = card_x + dx;
            let py = card_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, border_color);
            }
        }
    }

    // Right border
    for dy in 0..card_height {
        for dx in card_width.saturating_sub(border_width)..card_width {
            let px = card_x + dx;
            let py = card_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, border_color);
            }
        }
    }

    // 4. Re-render interior background (inside border)
    let inner_x = border_width;
    let inner_y = border_width;
    let inner_w = card_width.saturating_sub(border_width * 2);
    let inner_h = card_height.saturating_sub(border_width * 2);

    for dy in 0..inner_h {
        for dx in 0..inner_w {
            let px = card_x + inner_x + dx;
            let py = card_y + inner_y + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, bg_color);
            }
        }
    }

    // 5. Render rarity indicator region with a subtle tint of the border color
    let indicator = &params.slots.rarity_indicator_region;
    let indicator_color = Color::rgba(
        preset.border_color[0] * 0.6,
        preset.border_color[1] * 0.6,
        preset.border_color[2] * 0.6,
        preset.border_color[3] * 0.8,
    );

    for dy in 0..indicator[3] {
        for dx in 0..indicator[2] {
            let px = card_x + indicator[0] + dx;
            let py = card_y + indicator[1] + dy;
            if px < atlas.width && py < atlas.height {
                atlas.set(px, py, indicator_color);
            }
        }
    }
}

/// Simple over blending for glow effect.
fn blend_over(src: Color, dst: Color) -> Color {
    let src_a = src.a;
    let dst_a = dst.a;
    let out_a = src_a + dst_a * (1.0 - src_a);

    if out_a < 0.0001 {
        return Color::rgba(0.0, 0.0, 0.0, 0.0);
    }

    Color::rgba(
        (src.r * src_a + dst.r * dst_a * (1.0 - src_a)) / out_a,
        (src.g * src_a + dst.g * dst_a * (1.0 - src_a)) / out_a,
        (src.b * src_a + dst.b * dst_a * (1.0 - src_a)) / out_a,
        out_a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::ui::{ItemCardSlots, RarityPreset, RarityTier};

    fn make_basic_params() -> UiItemCardV1Params {
        UiItemCardV1Params::new(128, 192)
            .with_rarity(RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            ))
            .with_slots(ItemCardSlots {
                icon_region: [16, 16, 64, 64],
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            })
    }

    #[test]
    fn test_generate_single_variant() {
        let params = make_basic_params();
        let result = generate_item_card(&params, 42).unwrap();

        assert!(!result.png_data.is_empty());
        assert_eq!(result.metadata.variants.len(), 1);
        assert_eq!(result.metadata.variants[0].tier, "common");
        assert_eq!(result.metadata.card_width, 128);
        assert_eq!(result.metadata.card_height, 192);
    }

    #[test]
    fn test_generate_multiple_variants() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_rarity(RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            ))
            .with_rarity(RarityPreset::new(
                RarityTier::Rare,
                [0.0, 0.5, 1.0, 1.0],
                [0.1, 0.1, 0.2, 1.0],
            ))
            .with_rarity(
                RarityPreset::new(
                    RarityTier::Legendary,
                    [1.0, 0.8, 0.0, 1.0],
                    [0.3, 0.2, 0.0, 1.0],
                )
                .with_glow([1.0, 0.9, 0.5, 0.5]),
            )
            .with_slots(ItemCardSlots {
                icon_region: [16, 16, 64, 64],
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            });

        let result = generate_item_card(&params, 42).unwrap();

        assert_eq!(result.metadata.variants.len(), 3);
        // Variants should be sorted by tier
        assert_eq!(result.metadata.variants[0].tier, "common");
        assert_eq!(result.metadata.variants[1].tier, "rare");
        assert_eq!(result.metadata.variants[2].tier, "legendary");
    }

    #[test]
    fn test_determinism() {
        let params = make_basic_params();

        let result1 = generate_item_card(&params, 42).unwrap();
        let result2 = generate_item_card(&params, 42).unwrap();

        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);
    }

    #[test]
    fn test_no_rarity_presets() {
        let params = UiItemCardV1Params::new(128, 192).with_slots(ItemCardSlots {
            icon_region: [16, 16, 64, 64],
            rarity_indicator_region: [16, 90, 96, 16],
            background_region: [0, 0, 128, 192],
        });

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::NoRarityPresets));
    }

    #[test]
    fn test_duplicate_rarity_tier() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_rarity(RarityPreset::new(
                RarityTier::Rare,
                [0.0, 0.5, 1.0, 1.0],
                [0.1, 0.1, 0.2, 1.0],
            ))
            .with_rarity(RarityPreset::new(
                RarityTier::Rare,
                [0.0, 0.7, 1.0, 1.0],
                [0.1, 0.1, 0.3, 1.0],
            ))
            .with_slots(ItemCardSlots {
                icon_region: [16, 16, 64, 64],
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            });

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::DuplicateRarityTier(_)));
    }

    #[test]
    fn test_invalid_color() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_rarity(RarityPreset::new(
                RarityTier::Common,
                [1.5, 0.0, 0.0, 1.0], // Invalid: > 1.0
                [0.2, 0.2, 0.2, 1.0],
            ))
            .with_slots(ItemCardSlots {
                icon_region: [16, 16, 64, 64],
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            });

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::InvalidColor(_)));
    }

    #[test]
    fn test_slot_out_of_bounds() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_rarity(RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            ))
            .with_slots(ItemCardSlots {
                icon_region: [100, 16, 64, 64], // Extends beyond card width
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            });

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::SlotOutOfBounds(..)));
    }

    #[test]
    fn test_resolution_too_small() {
        let params = UiItemCardV1Params {
            resolution: [16, 16],
            padding: 2,
            rarity_presets: vec![RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            )],
            slots: ItemCardSlots {
                icon_region: [0, 0, 8, 8],
                rarity_indicator_region: [0, 8, 16, 4],
                background_region: [0, 0, 16, 16],
            },
            border_width: 2,
            corner_radius: 4,
        };

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::InvalidParameter(_)));
    }

    #[test]
    fn test_resolution_too_large() {
        let params = UiItemCardV1Params {
            resolution: [8192, 8192],
            padding: 2,
            rarity_presets: vec![RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            )],
            slots: ItemCardSlots {
                icon_region: [0, 0, 64, 64],
                rarity_indicator_region: [0, 64, 128, 16],
                background_region: [0, 0, 8192, 8192],
            },
            border_width: 2,
            corner_radius: 4,
        };

        let err = generate_item_card(&params, 42).unwrap_err();
        assert!(matches!(err, ItemCardError::InvalidParameter(_)));
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = make_basic_params();
        let result = generate_item_card(&params, 42).unwrap();

        let uv = &result.metadata.variants[0].uv;
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);
        assert!(uv.u_max > uv.u_min);
        assert!(uv.v_max > uv.v_min);
    }

    #[test]
    fn test_glow_effect() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_rarity(
                RarityPreset::new(
                    RarityTier::Legendary,
                    [1.0, 0.8, 0.0, 1.0],
                    [0.3, 0.2, 0.0, 1.0],
                )
                .with_glow([1.0, 0.9, 0.5, 0.5]),
            )
            .with_slots(ItemCardSlots {
                icon_region: [16, 16, 64, 64],
                rarity_indicator_region: [16, 90, 96, 16],
                background_region: [0, 0, 128, 192],
            });

        let result = generate_item_card(&params, 42).unwrap();
        assert!(!result.png_data.is_empty());
        assert_eq!(result.metadata.variants.len(), 1);
    }
}
