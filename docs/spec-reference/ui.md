# UI Asset Recipes

This document describes the UI asset recipe kinds for generating nine-slice panels and icon sets.

## Recipe Kinds

### `ui.nine_slice_v1`

Generates nine-slice panel textures with corner/edge/center regions for scalable UI elements.

**AssetType**: `ui`

**Outputs**:
- Primary (PNG): Atlas texture containing all nine regions
- Metadata (JSON): UV coordinates and dimensions for each region

**Parameters**:

```json
{
  "resolution": [256, 256],
  "padding": 2,
  "regions": {
    "corner_size": [16, 16],
    "top_left": [0.2, 0.2, 0.2, 1.0],
    "top_right": [0.2, 0.2, 0.2, 1.0],
    "bottom_left": [0.2, 0.2, 0.2, 1.0],
    "bottom_right": [0.2, 0.2, 0.2, 1.0],
    "top_edge": [0.3, 0.3, 0.3, 1.0],
    "bottom_edge": [0.3, 0.3, 0.3, 1.0],
    "left_edge": [0.3, 0.3, 0.3, 1.0],
    "right_edge": [0.3, 0.3, 0.3, 1.0],
    "center": [0.9, 0.9, 0.9, 1.0],
    "edge_width": 16,
    "edge_height": 16
  },
  "background_color": [0.0, 0.0, 0.0, 0.0]
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resolution` | `[u32; 2]` | Yes | Atlas resolution `[width, height]` in pixels |
| `padding` | `u32` | No | Padding/gutter between regions in pixels (default: 2) |
| `regions` | `NineSliceRegions` | Yes | Nine-slice region definitions |
| `background_color` | `[f64; 4]` | No | Optional background fill color (RGBA, 0.0-1.0) |

**`NineSliceRegions` Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `corner_size` | `[u32; 2]` | Yes | Corner dimensions `[width, height]` in pixels |
| `top_left` | `[f64; 4]` | Yes | Top-left corner fill color (RGBA, 0.0-1.0) |
| `top_right` | `[f64; 4]` | Yes | Top-right corner fill color (RGBA, 0.0-1.0) |
| `bottom_left` | `[f64; 4]` | Yes | Bottom-left corner fill color (RGBA, 0.0-1.0) |
| `bottom_right` | `[f64; 4]` | Yes | Bottom-right corner fill color (RGBA, 0.0-1.0) |
| `top_edge` | `[f64; 4]` | Yes | Top edge fill color (RGBA, 0.0-1.0) |
| `bottom_edge` | `[f64; 4]` | Yes | Bottom edge fill color (RGBA, 0.0-1.0) |
| `left_edge` | `[f64; 4]` | Yes | Left edge fill color (RGBA, 0.0-1.0) |
| `right_edge` | `[f64; 4]` | Yes | Right edge fill color (RGBA, 0.0-1.0) |
| `center` | `[f64; 4]` | Yes | Center fill color (RGBA, 0.0-1.0) |
| `edge_width` | `u32` | No | Edge slice width in pixels (default: `corner_size[0]`) |
| `edge_height` | `u32` | No | Edge slice height in pixels (default: `corner_size[1]`) |

**Metadata Output**:

The metadata JSON contains UV coordinates for all nine regions:

```json
{
  "atlas_width": 256,
  "atlas_height": 256,
  "padding": 2,
  "regions": {
    "top_left": {
      "u_min": 0.0078125,
      "v_min": 0.0078125,
      "u_max": 0.0703125,
      "v_max": 0.0703125,
      "width": 16,
      "height": 16
    },
    // ... other regions
  }
}
```

**Example Spec**:

```json
{
  "spec_version": 1,
  "asset_id": "ui-panel-simple",
  "asset_type": "ui",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "ui/panel.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "ui/panel_meta.json"
    }
  ],
  "recipe": {
    "kind": "ui.nine_slice_v1",
    "params": {
      "resolution": [256, 256],
      "padding": 2,
      "regions": {
        "corner_size": [16, 16],
        "top_left": [0.2, 0.2, 0.2, 1.0],
        "top_right": [0.2, 0.2, 0.2, 1.0],
        "bottom_left": [0.2, 0.2, 0.2, 1.0],
        "bottom_right": [0.2, 0.2, 0.2, 1.0],
        "top_edge": [0.3, 0.3, 0.3, 1.0],
        "bottom_edge": [0.3, 0.3, 0.3, 1.0],
        "left_edge": [0.3, 0.3, 0.3, 1.0],
        "right_edge": [0.3, 0.3, 0.3, 1.0],
        "center": [0.9, 0.9, 0.9, 1.0]
      }
    }
  }
}
```

---

### `ui.icon_set_v1`

Packs icon frames into a sprite atlas with deterministic shelf packing.

**AssetType**: `ui`

**Outputs**:
- Primary (PNG): Icon atlas texture
- Metadata (JSON): UV coordinates and metadata for each icon

**Parameters**:

```json
{
  "resolution": [512, 512],
  "padding": 2,
  "icons": [
    {
      "id": "close",
      "width": 32,
      "height": 32,
      "color": [1.0, 0.0, 0.0, 1.0],
      "category": "action"
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resolution` | `[u32; 2]` | Yes | Atlas resolution `[width, height]` in pixels |
| `padding` | `u32` | No | Padding/gutter between icons in pixels (default: 2) |
| `icons` | `Vec<IconEntry>` | Yes | List of icon entries to pack |

**`IconEntry` Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` | Yes | Unique identifier for this icon |
| `width` | `u32` | Yes | Icon width in pixels |
| `height` | `u32` | Yes | Icon height in pixels |
| `color` | `[f64; 4]` | Yes | Icon fill color (RGBA, 0.0-1.0) |
| `category` | `String` | No | Optional semantic category (e.g., "action", "status") |

**Metadata Output**:

The metadata JSON contains UV coordinates for each packed icon:

```json
{
  "atlas_width": 512,
  "atlas_height": 512,
  "padding": 2,
  "icons": [
    {
      "id": "close",
      "u_min": 0.0078125,
      "v_min": 0.0078125,
      "u_max": 0.0703125,
      "v_max": 0.0703125,
      "width": 32,
      "height": 32,
      "category": "action"
    }
  ]
}
```

**Example Spec**:

```json
{
  "spec_version": 1,
  "asset_id": "ui-icons-basic",
  "asset_type": "ui",
  "license": "CC0-1.0",
  "seed": 123,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "ui/icons.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "ui/icons_meta.json"
    }
  ],
  "recipe": {
    "kind": "ui.icon_set_v1",
    "params": {
      "resolution": [512, 512],
      "padding": 2,
      "icons": [
        {
          "id": "close",
          "width": 32,
          "height": 32,
          "color": [1.0, 0.0, 0.0, 1.0],
          "category": "action"
        },
        {
          "id": "settings",
          "width": 48,
          "height": 48,
          "color": [0.5, 0.5, 0.5, 1.0],
          "category": "system"
        }
      ]
    }
  }
}
```

## Determinism

Both UI recipe kinds are Tier 1 (Rust-only, byte-identical):
- Nine-slice generation produces identical PNG and metadata for the same spec hash/seed
- Icon set packing uses deterministic shelf algorithm (sorted by height, width, id)
- All outputs are byte-identical across platforms and runs

## Version History

- **v1** (2026-01): Initial implementation
  - Nine-slice panels with solid color regions
  - Icon sets with shelf packing algorithm
  - Metadata outputs with UV coordinates
