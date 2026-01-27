# SpecCade Starlark Standard Library - Core Functions

[← Back to Index](stdlib-reference.md)

## Core Functions

These are the fundamental functions for creating specs and defining outputs.

### spec()

Creates a complete spec dictionary with all required fields.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case identifier |
| asset_type | str | Yes | - | "audio", "music", "texture", "sprite", "vfx", "ui", "font", "static_mesh", "skeletal_mesh", "skeletal_animation" |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| outputs | list | Yes | - | List of output specifications |
| recipe | dict | No | None | Optional recipe specification |
| description | str | No | None | Optional description |
| tags | list | No | None | Optional style tags (stored as `style_tags` in the output spec) |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Returns:** Dict matching the Spec IR structure with `spec_version: 1`.

**Example:**
```starlark
spec(
    asset_id = "laser-blast-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/laser.wav", "wav")]
)
```

### output()

Creates an output specification.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| path | str | Yes | - | Output file path |
| format | str | Yes | - | Output format ("wav", "png", "glb", etc.) |
| kind | str | No | "primary" | "primary", "preview", or "metadata" |
| source | str | No | None | Optional output source selector (recipe-specific) |

**Returns:** Dict matching the Output IR structure.

**Example:**
```starlark
output("sounds/laser.wav", "wav")
output("textures/preview.png", "png", "preview")
output("textures/laser.report.json", "json", "metadata")
```

Validation rules for `metadata`, `preview`, and `source` are recipe-specific; use `speccade validate` for the authoritative rules.

---

[← Back to Index](stdlib-reference.md)
