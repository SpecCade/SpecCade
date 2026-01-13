# RFC-0005: Texture Graph IR (`texture.graph_v1`)

## Summary

Introduce a **map-agnostic**, **named-node** texture authoring IR for SpecCade: `texture.graph_v1`.

Unlike `texture.material_v1` (fixed PBR map set with opinionated semantics), `texture.graph_v1` is a generic
graph of deterministic image operations where:

- Authors define **named nodes** (intermediate maps).
- Nodes can reference other nodes as inputs (DAG).
- Outputs explicitly reference which node to write (no filename conventions).

This unlocks “IR-style” texture authoring similar in spirit to the music Pattern IR work: an explicit, stable,
deterministic intermediate representation that can grow over time without baking in game-specific conventions.

## Goals

- **No forced map semantics**: authors name their own maps (masks, ramps, channels, etc.).
- **Explicit routing**: map data flows are declared in the graph, not inferred from output filenames.
- **Deterministic**: same spec + seed -> byte-identical PNG output (Tier 1).
- **Composable**: simple primitives + composition ops cover many workflows (masks, ramps, packing, normals, etc.).
- **Incrementally extensible**: new node ops can be added without breaking existing graphs.

## Non-goals (v1)

- A full node editor UI.
- Shipping every image op under the sun.
- Perfect tileable noise for all noise algorithms.
- Replacing `texture.material_v1` (it remains useful for “one recipe -> PBR set” generation).

## Proposed Canonical Spec Shape

### Recipe Kind

`texture.graph_v1`

### Params (`recipe.params`)

```json
{
  "resolution": [256, 256],
  "tileable": true,
  "nodes": [
    { "id": "base", "type": "constant", "value": 0.2 },
    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
    { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.55 },
    { "id": "albedo", "type": "color_ramp", "input": "n", "ramp": ["#000000", "#ff6600"] },
    { "id": "rgba", "type": "compose_rgba", "r": "mask", "g": "n", "b": "base" }
  ]
}
```

### Output Binding (Explicit)

To avoid naming conventions, outputs declare a source node. This proposes adding an **optional** field
to `outputs[]`:

```json
{ "kind": "primary", "format": "png", "path": "my_mask.png", "source": "mask" }
```

For `texture.graph_v1`:

- Each `primary` PNG output **must** declare `source`.
- `source` **must** match a node `id`.

This field is ignored by existing recipes (it is only interpreted by `texture.graph_v1` initially).

## Node Model

Nodes are evaluated deterministically and may produce either:

- a grayscale image (single channel in `[0,1]`)
- a color image (RGBA in `[0,1]`)

Nodes reference inputs by node id, enabling arbitrary “map -> map” flows.

### v1 Node Ops (initial set)

Grayscale primitives:

- `constant { value }`
- `noise { noise }` (reuses existing `NoiseConfig`)
- `gradient { direction, ... }`
- `stripes { direction, stripe_width, color1, color2 }`
- `checkerboard { tile_size, color1, color2 }`

Grayscale ops:

- `invert { input }`
- `clamp { input, min, max }`
- `add { a, b }`
- `multiply { a, b }`
- `lerp { a, b, t }`
- `threshold { input, threshold }`

Color ops:

- `color_ramp { input, ramp: ["#RRGGBB", ...] }` (maps grayscale -> color)
- `palette { input, palette: ["#RRGGBB", ...] }` (quantizes color)
- `to_grayscale { input }` (explicit luminance conversion)
- `compose_rgba { r, g, b, a? }` (builds RGBA from grayscale nodes)
- `normal_from_height { input, strength }` (grayscale -> RGB normal)

## Determinism

- Node-local seeds are derived from the spec seed + node id (stable across reorderings).
- PNG encoding uses the existing deterministic encoder.

## Validation (Spec-Level)

`texture.graph_v1` validation should reject:

- duplicate node ids
- references to missing input node ids
- outputs that reference missing nodes
- obvious type mismatches (e.g. `palette` on a grayscale input) where practical
- cycles (graph must be a DAG)

## Backwards Compatibility

- Existing specs remain valid.
- Adding `outputs[].source` is backward compatible because it is optional; existing JSON specs will continue to parse.

## Future Work

- More ops (blur, warp, distance, morphology, blend modes, UV transforms, etc.).
- Better tileability controls per node.
- Explicit type annotations (if needed for stronger validation).
- A generic “graph” output kind or richer `source` descriptors (e.g. select a channel, output color type).
- Bridges: `texture.material_v1` as a node inside graphs, or graphs feeding `texture.packed_v1`.

