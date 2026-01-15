# Procedural Texture Nodes

Texture specs use a node graph system. Each node has an `id`, `type`, and type-specific parameters. Nodes connect via `input` references.

## Spec Structure

```json
{
  "recipe": {
    "kind": "texture.procedural_v1",
    "params": {
      "resolution": [512, 512],
      "tileable": true,
      "nodes": [ ... ]
    }
  }
}
```

**Output maps** - set in `outputs[].source`:
- `albedo` - Base color/diffuse
- `normal` - Normal map
- `roughness` - Surface roughness
- `metallic` - Metallic mask
- `ao` - Ambient occlusion
- `emissive` - Emission map

## Noise Nodes

### Perlin Noise
Classic gradient noise.

```json
{
  "id": "noise1",
  "type": "noise",
  "noise": {
    "type": "perlin",
    "scale": 4.0,           // Feature scale
    "octaves": 4,           // FBM octaves
    "persistence": 0.5,     // Amplitude falloff
    "lacunarity": 2.0       // Frequency multiplier
  }
}
```

### Simplex Noise
Improved noise with fewer artifacts.

```json
{
  "id": "noise1",
  "type": "noise",
  "noise": {
    "type": "simplex",
    "scale": 4.0,
    "octaves": 4,
    "persistence": 0.5
  }
}
```

### Worley/Voronoi Noise
Cellular patterns.

```json
{
  "id": "cells",
  "type": "noise",
  "noise": {
    "type": "worley",
    "scale": 8.0,
    "distance_function": "euclidean",  // euclidean, manhattan, chebyshev
    "return_type": "f1"      // f1, f2, f2_minus_f1
  }
}
```

### FBM
Fractal Brownian Motion (multi-octave noise).

```json
{
  "id": "fbm",
  "type": "noise",
  "noise": {
    "type": "fbm",
    "base_noise": "perlin",
    "scale": 4.0,
    "octaves": 6,
    "persistence": 0.5,
    "lacunarity": 2.0
  }
}
```

## Pattern Nodes

### Brick
Brick/tile pattern.

```json
{
  "id": "bricks",
  "type": "brick",
  "width": 4,               // Bricks per row
  "height": 8,              // Rows
  "mortar_size": 0.05,      // Gap width
  "offset": 0.5,            // Row offset (0.5 = standard)
  "variation": 0.1          // Random size variation
}
```

### Checkerboard
Regular checker pattern.

```json
{
  "id": "checker",
  "type": "checkerboard",
  "scale": 8                // Squares per dimension
}
```

### Stripes
Linear stripe pattern.

```json
{
  "id": "stripes",
  "type": "stripes",
  "count": 10,              // Number of stripes
  "angle": 0.0,             // Rotation in degrees
  "softness": 0.1           // Edge blur
}
```

### Gradient
Color gradients.

```json
{
  "id": "grad",
  "type": "gradient",
  "gradient_type": "linear", // linear, radial, angular
  "angle": 45.0,            // For linear
  "center": [0.5, 0.5]      // For radial
}
```

## Procedural Patterns

### Wood Grain
Wood texture simulation.

```json
{
  "id": "wood",
  "type": "wood_grain",
  "ring_scale": 20.0,       // Ring density
  "grain_scale": 100.0,     // Grain detail
  "turbulence": 0.3,        // Ring distortion
  "color_variation": 0.2
}
```

### Scratches
Surface wear scratches.

```json
{
  "id": "scratches",
  "type": "scratches",
  "density": 0.3,           // Scratch count
  "length": 0.4,            // Average length
  "width": 0.01,            // Line width
  "angle_variation": 30.0   // Direction randomness
}
```

### Edge Wear
Weathered edges effect.

```json
{
  "id": "wear",
  "type": "edge_wear",
  "input": "height_map",    // Height to detect edges
  "amount": 0.5,
  "noise_scale": 8.0
}
```

## Processing Nodes

### Color Ramp
Map grayscale to colors.

```json
{
  "id": "colored",
  "type": "color_ramp",
  "input": "noise1",
  "stops": [
    { "position": 0.0, "color": [0.2, 0.15, 0.1, 1.0] },
    { "position": 0.5, "color": [0.5, 0.4, 0.3, 1.0] },
    { "position": 1.0, "color": [0.8, 0.7, 0.6, 1.0] }
  ],
  "interpolation": "smooth" // linear, smooth, constant
}
```

### Threshold
Binary cutoff.

```json
{
  "id": "mask",
  "type": "threshold",
  "input": "noise1",
  "threshold": 0.5,
  "softness": 0.1           // Edge blur
}
```

### Levels
Brightness/contrast adjustment.

```json
{
  "id": "adjusted",
  "type": "levels",
  "input": "noise1",
  "black_point": 0.1,
  "white_point": 0.9,
  "gamma": 1.2
}
```

### Invert
Invert values.

```json
{
  "id": "inverted",
  "type": "invert",
  "input": "noise1"
}
```

### Blend
Combine two inputs.

```json
{
  "id": "combined",
  "type": "blend",
  "input_a": "noise1",
  "input_b": "noise2",
  "mode": "overlay",        // normal, multiply, screen, overlay, add, subtract
  "factor": 0.5             // Blend amount
}
```

## Material Map Nodes

### Normal From Height
Generate normal map from height.

```json
{
  "id": "normal",
  "type": "normal_from_height",
  "input": "height_node",
  "strength": 1.0           // Normal intensity
}
```

### AO From Height
Generate ambient occlusion from height.

```json
{
  "id": "ao",
  "type": "ao_from_height",
  "input": "height_node",
  "radius": 4,              // Sample radius
  "intensity": 1.0
}
```

### Channel Pack
Pack channels into single output (e.g., ORM map).

```json
{
  "id": "orm",
  "type": "channel_pack",
  "r": "ao_node",
  "g": "roughness_node",
  "b": "metallic_node"
}
```

## Example: Stone Material

```json
{
  "recipe": {
    "kind": "texture.procedural_v1",
    "params": {
      "resolution": [1024, 1024],
      "tileable": true,
      "nodes": [
        { "id": "base_noise", "type": "noise", "noise": { "type": "perlin", "scale": 3.0, "octaves": 4 }},
        { "id": "detail", "type": "noise", "noise": { "type": "worley", "scale": 10.0 }},
        { "id": "height", "type": "blend", "input_a": "base_noise", "input_b": "detail", "mode": "multiply", "factor": 0.7 },
        { "id": "albedo", "type": "color_ramp", "input": "height", "stops": [
          { "position": 0.0, "color": [0.3, 0.3, 0.32, 1.0] },
          { "position": 0.5, "color": [0.5, 0.48, 0.45, 1.0] },
          { "position": 1.0, "color": [0.65, 0.62, 0.58, 1.0] }
        ]},
        { "id": "roughness", "type": "levels", "input": "height", "black_point": 0.3, "white_point": 0.8 },
        { "id": "normal", "type": "normal_from_height", "input": "height", "strength": 1.2 },
        { "id": "ao", "type": "ao_from_height", "input": "height", "radius": 6 }
      ]
    }
  },
  "outputs": [
    { "kind": "primary", "format": "png", "path": "stone_albedo.png", "source": "albedo" },
    { "kind": "primary", "format": "png", "path": "stone_normal.png", "source": "normal" },
    { "kind": "primary", "format": "png", "path": "stone_roughness.png", "source": "roughness" },
    { "kind": "primary", "format": "png", "path": "stone_ao.png", "source": "ao" }
  ]
}
```
