# Font Spec Reference

This document describes the SpecCade font recipe types for generating bitmap and MSDF fonts.

## Overview

Font specs generate font atlases with glyph metrics for runtime text rendering. Two recipe kinds are supported:

- `font.bitmap_v1` - Bitmap pixel fonts with hardcoded patterns (Tier 1, deterministic)
- `font.msdf_v1` - Multi-channel signed distance field fonts (Tier 2, planned)

## Asset Type

All font recipes use `asset_type: font`.

## Recipe Kind: `font.bitmap_v1`

Generates bitmap pixel fonts using hardcoded glyph patterns. Glyphs are packed into a PNG atlas with deterministic shelf packing, and metrics are output as JSON.

### Output Files

- **Primary**: PNG atlas texture containing all glyphs
- **Metadata**: JSON file with glyph metrics (UVs, advance, baseline)

### Parameters

```json
{
  "kind": "font.bitmap_v1",
  "params": {
    "charset": [32, 126],
    "glyph_size": [5, 7],
    "padding": 2,
    "font_style": "monospace",
    "color": [1.0, 1.0, 1.0, 1.0]
  }
}
```

#### Parameter Details

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `charset` | `[u32; 2]` | Yes | - | Character range `[start, end]` inclusive. Must be ASCII (32-126). |
| `glyph_size` | `[u32; 2]` | Yes | - | Glyph dimensions `[width, height]`. Supported: `[5,7]`, `[8,8]`, `[6,9]`. |
| `padding` | `u32` | No | `2` | Padding in pixels between glyphs (for mip-safe borders). |
| `font_style` | `string` | No | `"monospace"` | Font style: `"monospace"` or `"proportional"`. |
| `color` | `[f64; 4]` | No | `[1.0, 1.0, 1.0, 1.0]` | Glyph color in RGBA (0.0-1.0). |

### Supported Glyph Sizes

- `[5, 7]` - Classic 5x7 pixel font (most compact)
- `[8, 8]` - Square 8x8 pixel font
- `[6, 9]` - Taller 6x9 pixel font

### Font Styles

- **`monospace`**: All glyphs have the same advance width (equal spacing).
- **`proportional`**: Glyphs have variable advance widths based on actual pixel content (tighter spacing).

### Metadata Output Format

The metadata JSON contains glyph metrics for runtime rendering:

```json
{
  "atlas_width": 256,
  "atlas_height": 128,
  "glyph_size": [5, 7],
  "padding": 2,
  "font_style": "monospace",
  "line_height": 9,
  "glyphs": [
    {
      "char_code": 65,
      "character": "A",
      "uv_min": [0.0, 0.0],
      "uv_max": [0.02, 0.05],
      "width": 5,
      "height": 7,
      "advance": 6,
      "baseline": 5
    }
  ]
}
```

#### Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `atlas_width` | `u32` | Atlas texture width in pixels. |
| `atlas_height` | `u32` | Atlas texture height in pixels. |
| `glyph_size` | `[u32; 2]` | Base glyph dimensions. |
| `padding` | `u32` | Padding between glyphs. |
| `font_style` | `string` | Font style (`"monospace"` or `"proportional"`). |
| `line_height` | `u32` | Recommended line spacing in pixels. |
| `glyphs` | `array` | Array of glyph metadata entries. |

#### Glyph Metadata Entry

| Field | Type | Description |
|-------|------|-------------|
| `char_code` | `u32` | ASCII character code (e.g., 65 for 'A'). |
| `character` | `string` | Character as a string (e.g., "A"). |
| `uv_min` | `[f64; 2]` | Top-left UV coordinates (normalized [0,1]). |
| `uv_max` | `[f64; 2]` | Bottom-right UV coordinates (normalized [0,1]). |
| `width` | `u32` | Glyph width in pixels. |
| `height` | `u32` | Glyph height in pixels. |
| `advance` | `u32` | Horizontal advance to next glyph in pixels. |
| `baseline` | `u32` | Distance from top to baseline in pixels. |

## Example Specs

### Minimal Monospace Font

```json
{
  "spec_version": 1,
  "asset_id": "mono-font-01",
  "asset_type": "font",
  "license": "CC0-1.0",
  "seed": 100,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "fonts/mono_atlas.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "fonts/mono_metrics.json"
    }
  ],
  "recipe": {
    "kind": "font.bitmap_v1",
    "params": {
      "charset": [32, 126],
      "glyph_size": [5, 7]
    }
  }
}
```

### Proportional Font with Custom Color

```json
{
  "spec_version": 1,
  "asset_id": "prop-font-01",
  "asset_type": "font",
  "license": "CC0-1.0",
  "seed": 200,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "fonts/prop_atlas.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "fonts/prop_metrics.json"
    }
  ],
  "recipe": {
    "kind": "font.bitmap_v1",
    "params": {
      "charset": [65, 90],
      "glyph_size": [8, 8],
      "padding": 1,
      "font_style": "proportional",
      "color": [0.9, 0.9, 1.0, 1.0]
    }
  }
}
```

## Determinism

`font.bitmap_v1` is a **Tier 1** backend with full deterministic guarantees:

- Same spec + same seed = byte-identical PNG atlas
- Same spec + same seed = identical JSON metadata
- Uses deterministic shelf packing algorithm
- Hardcoded glyph patterns (no external font files)

## Validation Rules

- `charset[0]` must be <= `charset[1]`
- `charset[1]` must be <= 126 (printable ASCII range)
- `glyph_size` must be one of the supported sizes: `[5,7]`, `[8,8]`, `[6,9]`
- `padding` must be >= 0
- `color` values must be in range [0.0, 1.0]
- Must have at least one `primary` output of format `png`
- Metadata outputs must be format `json`

## Usage in Runtime

The metadata JSON can be loaded at runtime to render text:

```rust
// Pseudocode for text rendering
let metrics: FontBitmapMetadata = load_json("fonts/mono_metrics.json");
let atlas: Texture = load_texture("fonts/mono_atlas.png");

for char in "Hello".chars() {
    let glyph = metrics.glyphs.iter()
        .find(|g| g.character == char.to_string())
        .unwrap();

    // Render quad with atlas texture and glyph UVs
    render_quad(
        position,
        glyph.width,
        glyph.height,
        atlas,
        glyph.uv_min,
        glyph.uv_max
    );

    // Advance cursor
    position.x += glyph.advance;
}
```

## Future Extensions

- **`font.msdf_v1`**: Multi-channel signed distance field fonts for scalable vector rendering
- **Variable font support**: Parameterized font weights and styles
- **Kerning tables**: Advanced glyph pair spacing
- **Ligatures**: Combined glyph rendering
- **Unicode support**: Extended character sets beyond ASCII
