//! Damage number recipe types for `ui.damage_number_v1`.
//!
//! Damage number sprites are UI elements for displaying combat feedback (damage, healing, etc.)
//! with customizable styles. Multiple style variants are packed into a single atlas.

use serde::{Deserialize, Serialize};

use super::DEFAULT_UI_PADDING;

/// Parameters for the `ui.damage_number_v1` recipe.
///
/// Generates an atlas of damage number digit sprites with multiple style variants.
/// Each style has its own color scheme for text, outline, and optional glow effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiDamageNumberV1Params {
    /// Base glyph size [width, height] in pixels for each digit.
    /// Minimum: 8x8, Maximum: 128x128
    pub glyph_size: [u32; 2],

    /// Outline thickness in pixels (1-8).
    pub outline_width: u32,

    /// Padding/spacing between glyphs in the atlas.
    #[serde(default = "default_padding")]
    pub padding: u32,

    /// List of style variants to generate.
    pub styles: Vec<DamageNumberStyle>,
}

fn default_padding() -> u32 {
    DEFAULT_UI_PADDING
}

/// A damage number style variant defining visual appearance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageNumberStyle {
    /// Style type identifier.
    pub style_type: DamageNumberStyleType,

    /// Text fill color (RGBA, 0.0-1.0).
    pub text_color: [f64; 4],

    /// Outline color (RGBA, 0.0-1.0).
    pub outline_color: [f64; 4],

    /// Optional glow color (RGBA, 0.0-1.0).
    /// If specified, draws a glow effect below the outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glow_color: Option<[f64; 4]>,

    /// Optional scale multiplier (default: 1.0).
    /// Critical hits might use 1.25 for emphasis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

/// Style type enumeration for damage numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageNumberStyleType {
    /// Normal damage (default style).
    Normal,
    /// Critical hit damage (emphasized).
    Critical,
    /// Healing (positive effect).
    Healing,
}

impl DamageNumberStyleType {
    /// Returns the style type as a string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            DamageNumberStyleType::Normal => "normal",
            DamageNumberStyleType::Critical => "critical",
            DamageNumberStyleType::Healing => "healing",
        }
    }

    /// Returns the sort order for deterministic atlas packing.
    pub fn sort_order(&self) -> u8 {
        match self {
            DamageNumberStyleType::Normal => 0,
            DamageNumberStyleType::Critical => 1,
            DamageNumberStyleType::Healing => 2,
        }
    }
}

impl std::fmt::Display for DamageNumberStyleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Glyph entry in the metadata output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageNumberGlyph {
    /// Character represented (e.g., "0", "1", "+", "-", ".").
    pub char_code: String,

    /// UV coordinates [u_min, v_min, u_max, v_max] in normalized [0, 1] space.
    pub uv: [f64; 4],

    /// Glyph width in pixels.
    pub width: u32,

    /// Glyph height in pixels.
    pub height: u32,
}

/// Style entry in the metadata output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageNumberStyleMetadata {
    /// Style type identifier.
    pub style: String,

    /// UV offset for this style's region in the atlas [u, v].
    pub uv_offset: [f64; 2],

    /// Scale factor applied to this style.
    pub scale: f64,

    /// Glyph entries for this style.
    pub glyphs: Vec<DamageNumberGlyph>,
}

/// Metadata output for a damage number atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageNumberMetadata {
    /// Atlas dimensions [width, height] in pixels.
    pub atlas_size: [u32; 2],

    /// Base glyph size [width, height] in pixels.
    pub glyph_size: [u32; 2],

    /// Outline width in pixels.
    pub outline_width: u32,

    /// Padding between glyphs in pixels.
    pub padding: u32,

    /// Style variants with their glyph data.
    pub styles: Vec<DamageNumberStyleMetadata>,
}

impl UiDamageNumberV1Params {
    /// Creates new damage number params with the given glyph size.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            glyph_size: [width, height],
            outline_width: 1,
            padding: DEFAULT_UI_PADDING,
            styles: Vec::new(),
        }
    }

    /// Sets the outline width.
    pub fn with_outline_width(mut self, width: u32) -> Self {
        self.outline_width = width;
        self
    }

    /// Sets the padding between glyphs.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Adds a style variant.
    pub fn with_style(mut self, style: DamageNumberStyle) -> Self {
        self.styles.push(style);
        self
    }
}

impl DamageNumberStyle {
    /// Creates a new damage number style.
    pub fn new(
        style_type: DamageNumberStyleType,
        text_color: [f64; 4],
        outline_color: [f64; 4],
    ) -> Self {
        Self {
            style_type,
            text_color,
            outline_color,
            glow_color: None,
            scale: None,
        }
    }

    /// Sets the glow color for this style.
    pub fn with_glow(mut self, color: [f64; 4]) -> Self {
        self.glow_color = Some(color);
        self
    }

    /// Sets the scale multiplier for this style.
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_number_params_builder() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_padding(4)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Normal,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ))
            .with_style(
                DamageNumberStyle::new(
                    DamageNumberStyleType::Critical,
                    [1.0, 0.9, 0.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                )
                .with_scale(1.25)
                .with_glow([1.0, 0.5, 0.0, 0.5]),
            );

        assert_eq!(params.glyph_size, [16, 24]);
        assert_eq!(params.outline_width, 2);
        assert_eq!(params.padding, 4);
        assert_eq!(params.styles.len(), 2);
        assert_eq!(params.styles[0].style_type, DamageNumberStyleType::Normal);
        assert_eq!(params.styles[1].style_type, DamageNumberStyleType::Critical);
        assert!(params.styles[1].glow_color.is_some());
        assert_eq!(params.styles[1].scale, Some(1.25));
    }

    #[test]
    fn test_style_type_as_str() {
        assert_eq!(DamageNumberStyleType::Normal.as_str(), "normal");
        assert_eq!(DamageNumberStyleType::Critical.as_str(), "critical");
        assert_eq!(DamageNumberStyleType::Healing.as_str(), "healing");
    }

    #[test]
    fn test_style_type_sort_order() {
        assert!(
            DamageNumberStyleType::Normal.sort_order()
                < DamageNumberStyleType::Critical.sort_order()
        );
        assert!(
            DamageNumberStyleType::Critical.sort_order()
                < DamageNumberStyleType::Healing.sort_order()
        );
    }

    #[test]
    fn test_damage_number_serde() {
        let params = UiDamageNumberV1Params::new(16, 24)
            .with_outline_width(2)
            .with_style(DamageNumberStyle::new(
                DamageNumberStyleType::Healing,
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.5, 0.0, 1.0],
            ));

        let json = serde_json::to_string(&params).unwrap();
        let parsed: UiDamageNumberV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(params, parsed);
    }

    #[test]
    fn test_damage_number_serde_deny_unknown_fields() {
        let json = r#"{"glyph_size":[16,24],"outline_width":2,"styles":[],"unknown_field":true}"#;
        let result: Result<UiDamageNumberV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
