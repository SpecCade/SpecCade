# Simple Subprocess Extension

A reference implementation of a SpecCade subprocess extension.

## Overview

This extension demonstrates the I/O contract for subprocess-based SpecCade extensions. It generates simple gradient textures with deterministic output (Tier 1).

## Recipe Kind

This extension handles: `texture.simple_gradient_v1`

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `width` | integer | 256 | Texture width in pixels (1-4096) |
| `height` | integer | 256 | Texture height in pixels (1-4096) |
| `start_color` | [r,g,b,a] | [0,0,0,255] | Starting color (RGBA) |
| `end_color` | [r,g,b,a] | [255,255,255,255] | Ending color (RGBA) |
| `direction` | string | "horizontal" | Gradient direction: "horizontal" or "vertical" |
| `noise_amount` | float | 0.0 | Amount of noise to add (0.0-1.0) |

## Building

```bash
cd examples/extensions/simple-subprocess
cargo build --release
```

The binary will be at `target/release/simple-subprocess-extension`.

## Usage

### Direct invocation

```bash
simple-subprocess-extension --spec input.spec.json --out ./output --seed 42
```

### Example spec

```json
{
  "spec_version": 1,
  "asset_id": "gradient-test-01",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/gradient.png"
    }
  ],
  "recipe": {
    "kind": "texture.simple_gradient_v1",
    "params": {
      "width": 512,
      "height": 512,
      "start_color": [255, 0, 0, 255],
      "end_color": [0, 0, 255, 255],
      "direction": "horizontal"
    }
  }
}
```

### Integration with SpecCade

1. Build the extension
2. Copy it to an extensions directory
3. Copy `manifest.json` alongside it
4. SpecCade will discover and use it for `texture.simple_gradient_v1` specs

## Output

### Generated files

- `{output_path}` - The generated PNG texture

### Output manifest

The extension writes `manifest.json`:

```json
{
  "manifest_version": 1,
  "success": true,
  "output_files": [
    {
      "path": "textures/gradient.png",
      "hash": "...",
      "size": 1234,
      "kind": "primary",
      "format": "png"
    }
  ],
  "determinism_report": {
    "input_hash": "...",
    "output_hash": "...",
    "tier": 1,
    "determinism": "byte_identical",
    "seed": 42,
    "deterministic": true
  },
  "errors": [],
  "warnings": [],
  "duration_ms": 50,
  "extension_version": "1.0.0"
}
```

## Determinism

This extension is Tier 1 (byte-identical). Given the same input spec and seed, it produces identical output bytes.

Determinism is achieved by:
- Using PCG64 PRNG seeded from the provided seed
- Using deterministic PNG compression settings
- Avoiding any system-dependent operations

## Error Handling

The extension handles errors gracefully:

- Invalid arguments: Exit code 2
- Invalid spec: Exit code 3
- Generation failure: Exit code 4

Error details are written to `manifest.json` and stderr.

## Testing Determinism

```bash
# Generate twice with same seed
simple-subprocess-extension --spec test.spec.json --out ./out1 --seed 42
simple-subprocess-extension --spec test.spec.json --out ./out2 --seed 42

# Compare hashes
b3sum out1/textures/gradient.png out2/textures/gradient.png
# Should show identical hashes
```

## License

MIT OR Apache-2.0
