//! Item card recipe types for `ui.item_card_v1`.
//!
//! Item cards are UI templates with customizable slots for icon, rarity indicator,
//! and background. Multiple rarity variants are packed into a single atlas.

use serde::{Deserialize, Serialize};

use super::DEFAULT_UI_PADDING;

/// Parameters for the `ui.item_card_v1` recipe.
///
/// Generates an atlas of item card templates for different rarity tiers.
/// Each variant includes background, border, and slot regions for runtime composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiItemCardV1Params {
    /// Card resolution [width, height] in pixels for each individual card.
    pub resolution: [u32; 2],

    /// Padding/gutter in pixels between variants in the atlas (for mip-safe borders).
    #[serde(default = "default_ui_padding")]
    pub padding: u32,

    /// List of rarity presets to generate.
    pub rarity_presets: Vec<RarityPreset>,

    /// Slot layout definitions for each card.
    pub slots: ItemCardSlots,

    /// Border width in pixels.
    #[serde(default = "default_border_width")]
    pub border_width: u32,

    /// Corner radius in pixels (for visual reference; solid fill in v1).
    #[serde(default = "default_corner_radius")]
    pub corner_radius: u32,
}

fn default_ui_padding() -> u32 {
    DEFAULT_UI_PADDING
}

fn default_border_width() -> u32 {
    2
}

fn default_corner_radius() -> u32 {
    8
}

/// A rarity tier preset defining visual appearance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RarityPreset {
    /// Rarity tier identifier.
    pub tier: RarityTier,

    /// Border color (RGBA, 0.0-1.0).
    pub border_color: [f64; 4],

    /// Background fill color (RGBA, 0.0-1.0).
    pub background_color: [f64; 4],

    /// Optional glow color (RGBA, 0.0-1.0).
    /// If specified, draws a glow effect around the border.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glow_color: Option<[f64; 4]>,
}

/// Rarity tier enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RarityTier {
    /// Common items (default, lowest tier).
    Common,
    /// Uncommon items.
    Uncommon,
    /// Rare items.
    Rare,
    /// Epic items.
    Epic,
    /// Legendary items (highest tier).
    Legendary,
}

impl RarityTier {
    /// Returns the tier as a string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            RarityTier::Common => "common",
            RarityTier::Uncommon => "uncommon",
            RarityTier::Rare => "rare",
            RarityTier::Epic => "epic",
            RarityTier::Legendary => "legendary",
        }
    }

    /// Returns the sort order for deterministic atlas packing.
    pub fn sort_order(&self) -> u8 {
        match self {
            RarityTier::Common => 0,
            RarityTier::Uncommon => 1,
            RarityTier::Rare => 2,
            RarityTier::Epic => 3,
            RarityTier::Legendary => 4,
        }
    }
}

impl std::fmt::Display for RarityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Slot layout definitions for an item card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemCardSlots {
    /// Icon slot region [x, y, width, height] in pixels.
    pub icon_region: [u32; 4],

    /// Rarity indicator region [x, y, width, height] in pixels.
    pub rarity_indicator_region: [u32; 4],

    /// Background region [x, y, width, height] in pixels.
    /// Typically the full card size starting at (0, 0).
    pub background_region: [u32; 4],
}

/// UV rectangle in normalized [0, 1] coordinates for atlas sampling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCardUv {
    /// Left edge U coordinate (0-1).
    pub u_min: f64,

    /// Top edge V coordinate (0-1).
    pub v_min: f64,

    /// Right edge U coordinate (0-1).
    pub u_max: f64,

    /// Bottom edge V coordinate (0-1).
    pub v_max: f64,
}

impl ItemCardUv {
    /// Creates UV coordinates from pixel positions and atlas dimensions.
    pub fn from_pixels(x: u32, y: u32, w: u32, h: u32, atlas_w: u32, atlas_h: u32) -> Self {
        Self {
            u_min: x as f64 / atlas_w as f64,
            v_min: y as f64 / atlas_h as f64,
            u_max: (x + w) as f64 / atlas_w as f64,
            v_max: (y + h) as f64 / atlas_h as f64,
        }
    }
}

/// Slot region in pixel coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotRegion {
    /// X offset within the card.
    pub x: u32,
    /// Y offset within the card.
    pub y: u32,
    /// Width of the slot.
    pub width: u32,
    /// Height of the slot.
    pub height: u32,
}

impl SlotRegion {
    /// Creates a slot region from a 4-element array.
    pub fn from_array(arr: [u32; 4]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            width: arr[2],
            height: arr[3],
        }
    }
}

/// Variant entry in the metadata output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCardVariant {
    /// Rarity tier identifier.
    pub tier: String,

    /// UV coordinates for this variant in the atlas.
    pub uv: ItemCardUv,

    /// Slot regions within this card variant.
    pub slots: ItemCardSlotRegions,
}

/// Slot regions for a card variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCardSlotRegions {
    /// Icon slot region.
    pub icon: SlotRegion,
    /// Rarity indicator region.
    pub rarity_indicator: SlotRegion,
    /// Background region.
    pub background: SlotRegion,
}

/// Metadata output for an item card atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCardMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Padding/gutter in pixels.
    pub padding: u32,

    /// Card width in pixels.
    pub card_width: u32,

    /// Card height in pixels.
    pub card_height: u32,

    /// Variants packed into this atlas.
    pub variants: Vec<ItemCardVariant>,
}

impl UiItemCardV1Params {
    /// Creates new item card params with the given card resolution.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            resolution: [width, height],
            padding: DEFAULT_UI_PADDING,
            rarity_presets: Vec::new(),
            slots: ItemCardSlots {
                icon_region: [8, 8, 64, 64],
                rarity_indicator_region: [8, 80, 112, 16],
                background_region: [0, 0, width, height],
            },
            border_width: default_border_width(),
            corner_radius: default_corner_radius(),
        }
    }

    /// Sets the padding between variants.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Adds a rarity preset to the card.
    pub fn with_rarity(mut self, preset: RarityPreset) -> Self {
        self.rarity_presets.push(preset);
        self
    }

    /// Sets the slot layout.
    pub fn with_slots(mut self, slots: ItemCardSlots) -> Self {
        self.slots = slots;
        self
    }

    /// Sets the border width.
    pub fn with_border_width(mut self, width: u32) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the corner radius.
    pub fn with_corner_radius(mut self, radius: u32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl RarityPreset {
    /// Creates a new rarity preset.
    pub fn new(tier: RarityTier, border_color: [f64; 4], background_color: [f64; 4]) -> Self {
        Self {
            tier,
            border_color,
            background_color,
            glow_color: None,
        }
    }

    /// Sets the glow color for this preset.
    pub fn with_glow(mut self, color: [f64; 4]) -> Self {
        self.glow_color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_card_params_builder() {
        let params = UiItemCardV1Params::new(128, 192)
            .with_padding(4)
            .with_border_width(3)
            .with_rarity(RarityPreset::new(
                RarityTier::Common,
                [0.5, 0.5, 0.5, 1.0],
                [0.2, 0.2, 0.2, 1.0],
            ))
            .with_rarity(
                RarityPreset::new(
                    RarityTier::Legendary,
                    [1.0, 0.8, 0.0, 1.0],
                    [0.3, 0.2, 0.0, 1.0],
                )
                .with_glow([1.0, 0.9, 0.5, 0.5]),
            );

        assert_eq!(params.resolution, [128, 192]);
        assert_eq!(params.padding, 4);
        assert_eq!(params.border_width, 3);
        assert_eq!(params.rarity_presets.len(), 2);
        assert_eq!(params.rarity_presets[0].tier, RarityTier::Common);
        assert_eq!(params.rarity_presets[1].tier, RarityTier::Legendary);
        assert!(params.rarity_presets[1].glow_color.is_some());
    }

    #[test]
    fn test_rarity_tier_as_str() {
        assert_eq!(RarityTier::Common.as_str(), "common");
        assert_eq!(RarityTier::Uncommon.as_str(), "uncommon");
        assert_eq!(RarityTier::Rare.as_str(), "rare");
        assert_eq!(RarityTier::Epic.as_str(), "epic");
        assert_eq!(RarityTier::Legendary.as_str(), "legendary");
    }

    #[test]
    fn test_rarity_tier_sort_order() {
        assert!(RarityTier::Common.sort_order() < RarityTier::Uncommon.sort_order());
        assert!(RarityTier::Uncommon.sort_order() < RarityTier::Rare.sort_order());
        assert!(RarityTier::Rare.sort_order() < RarityTier::Epic.sort_order());
        assert!(RarityTier::Epic.sort_order() < RarityTier::Legendary.sort_order());
    }

    #[test]
    fn test_item_card_uv_from_pixels() {
        let uv = ItemCardUv::from_pixels(0, 0, 128, 192, 512, 192);

        assert_eq!(uv.u_min, 0.0);
        assert_eq!(uv.v_min, 0.0);
        assert_eq!(uv.u_max, 0.25);
        assert_eq!(uv.v_max, 1.0);
    }

    #[test]
    fn test_slot_region_from_array() {
        let region = SlotRegion::from_array([8, 16, 64, 64]);

        assert_eq!(region.x, 8);
        assert_eq!(region.y, 16);
        assert_eq!(region.width, 64);
        assert_eq!(region.height, 64);
    }

    #[test]
    fn test_item_card_serde() {
        let params = UiItemCardV1Params::new(128, 192).with_rarity(RarityPreset::new(
            RarityTier::Rare,
            [0.0, 0.5, 1.0, 1.0],
            [0.1, 0.1, 0.2, 1.0],
        ));

        let json = serde_json::to_string(&params).unwrap();
        let parsed: UiItemCardV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(params, parsed);
    }

    #[test]
    fn test_item_card_serde_deny_unknown_fields() {
        let json = r#"{"resolution":[128,192],"padding":2,"rarity_presets":[],"slots":{"icon_region":[0,0,64,64],"rarity_indicator_region":[0,64,128,16],"background_region":[0,0,128,192]},"unknown_field":true}"#;
        let result: Result<UiItemCardV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
