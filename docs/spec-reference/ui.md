# UI Asset Recipes

This document describes the UI asset recipe kinds for generating nine-slice panels, icon sets, and item cards.

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

---

### `ui.item_card_v1`

Generates item card templates with multiple rarity variants packed into a single atlas.

**AssetType**: `ui`

**Outputs**:
- Primary (PNG): Atlas texture containing all rarity variants
- Metadata (JSON): UV coordinates, slot regions, and variant metadata

**Parameters**:

```json
{
  "resolution": [128, 192],
  "padding": 2,
  "border_width": 3,
  "corner_radius": 8,
  "slots": {
    "icon_region": [16, 16, 96, 96],
    "rarity_indicator_region": [16, 120, 96, 16],
    "background_region": [0, 0, 128, 192]
  },
  "rarity_presets": [
    {
      "tier": "common",
      "border_color": [0.5, 0.5, 0.5, 1.0],
      "background_color": [0.15, 0.15, 0.15, 1.0]
    },
    {
      "tier": "legendary",
      "border_color": [1.0, 0.8, 0.2, 1.0],
      "background_color": [0.25, 0.2, 0.1, 1.0],
      "glow_color": [1.0, 0.9, 0.4, 0.4]
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resolution` | `[u32; 2]` | Yes | Card resolution `[width, height]` in pixels (min 32x32, max 4096x4096) |
| `padding` | `u32` | No | Padding/gutter between variants in pixels (default: 2) |
| `border_width` | `u32` | No | Border thickness in pixels (default: 2) |
| `corner_radius` | `u32` | No | Corner radius in pixels (default: 8, visual reference only in v1) |
| `slots` | `ItemCardSlots` | Yes | Slot layout definitions |
| `rarity_presets` | `Vec<RarityPreset>` | Yes | List of rarity presets (at least 1 required) |

**`ItemCardSlots` Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `icon_region` | `[u32; 4]` | Yes | Icon slot `[x, y, width, height]` in pixels |
| `rarity_indicator_region` | `[u32; 4]` | Yes | Rarity indicator `[x, y, width, height]` in pixels |
| `background_region` | `[u32; 4]` | Yes | Background `[x, y, width, height]` in pixels |

**`RarityPreset` Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tier` | `RarityTier` | Yes | One of: `common`, `uncommon`, `rare`, `epic`, `legendary` |
| `border_color` | `[f64; 4]` | Yes | Border fill color (RGBA, 0.0-1.0) |
| `background_color` | `[f64; 4]` | Yes | Background fill color (RGBA, 0.0-1.0) |
| `glow_color` | `[f64; 4]` | No | Optional glow effect color (RGBA, 0.0-1.0) |

**Metadata Output**:

The metadata JSON contains UV coordinates and slot regions for each rarity variant:

```json
{
  "atlas_width": 660,
  "atlas_height": 196,
  "padding": 2,
  "card_width": 128,
  "card_height": 192,
  "variants": [
    {
      "tier": "common",
      "uv": {
        "u_min": 0.003,
        "v_min": 0.010,
        "u_max": 0.197,
        "v_max": 0.990
      },
      "slots": {
        "icon": {"x": 16, "y": 16, "width": 96, "height": 96},
        "rarity_indicator": {"x": 16, "y": 120, "width": 96, "height": 16},
        "background": {"x": 0, "y": 0, "width": 128, "height": 192}
      }
    }
  ]
}
```

**Example Spec**:

```json
{
  "spec_version": 1,
  "asset_id": "ui-item-cards-rpg",
  "asset_type": "ui",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "ui/item_cards.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "ui/item_cards.json"
    }
  ],
  "recipe": {
    "kind": "ui.item_card_v1",
    "params": {
      "resolution": [128, 192],
      "padding": 2,
      "border_width": 3,
      "corner_radius": 8,
      "slots": {
        "icon_region": [16, 16, 96, 96],
        "rarity_indicator_region": [16, 120, 96, 16],
        "background_region": [0, 0, 128, 192]
      },
      "rarity_presets": [
        {
          "tier": "common",
          "border_color": [0.5, 0.5, 0.5, 1.0],
          "background_color": [0.15, 0.15, 0.15, 1.0]
        },
        {
          "tier": "uncommon",
          "border_color": [0.2, 0.8, 0.2, 1.0],
          "background_color": [0.1, 0.2, 0.1, 1.0]
        },
        {
          "tier": "rare",
          "border_color": [0.2, 0.5, 1.0, 1.0],
          "background_color": [0.1, 0.15, 0.25, 1.0]
        },
        {
          "tier": "epic",
          "border_color": [0.7, 0.3, 0.9, 1.0],
          "background_color": [0.2, 0.1, 0.25, 1.0]
        },
        {
          "tier": "legendary",
          "border_color": [1.0, 0.8, 0.2, 1.0],
          "background_color": [0.25, 0.2, 0.1, 1.0],
          "glow_color": [1.0, 0.9, 0.4, 0.4]
        }
      ]
    }
  }
}
```

## Determinism

All UI recipe kinds are Tier 1 (Rust-only, byte-identical):
- Nine-slice generation produces identical PNG and metadata for the same spec hash/seed
- Icon set packing uses deterministic shelf algorithm (sorted by height, width, id)
- Item card generation uses deterministic horizontal packing (sorted by rarity tier)
- All outputs are byte-identical across platforms and runs

## Version History

- **v1** (2026-01): Initial implementation
  - Nine-slice panels with solid color regions
  - Icon sets with shelf packing algorithm
  - Item card templates with rarity variants
  - Metadata outputs with UV coordinates
