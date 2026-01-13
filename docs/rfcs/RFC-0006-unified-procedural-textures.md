# RFC-0006: Unified Procedural Textures (`texture.procedural_v1`)

- **Status:** Draft
- **Author:** SpecCade Team
- **Created:** 2026-01-13
- **Target Version:** SpecCade v1 (pre-release breaking change)
- **Depends on:** RFC-0001 (Canonical Spec Architecture)
- **Supersedes:** RFC-0002 (Channel Packing), RFC-0005 (Texture Graph IR)

## Summary

This RFC proposes a **hard break** unification of SpecCade’s texture generation into a single, map-agnostic recipe kind:

- `texture.procedural_v1`

The unified recipe is a deterministic, named-node **DAG** that can generate **any** texture outputs the author wants, by name, without prescribing semantics like “albedo”, “normal”, or “ORM”.

This RFC explicitly removes the following recipe kinds from the canonical surface area:

- `texture.material_v1`
- `texture.normal_v1`
- `texture.packed_v1`
- `texture.graph_v1`

Instead, those concepts become **user-level workflows** built from the same primitives:

- A “material” is a reusable *graph pattern* that emits multiple named outputs.
- A “normal map” is just a named output derived from a height node (`normal_from_height`).
- “Packing” is just constructing an RGBA node (e.g. `compose_rgba`) and outputting it as PNG.

## 1. Motivation

Spec authors and tool users currently have to answer unnecessary questions:

- “Is this a *material* or a *normal*?”
- “Do I need *packed* or *graph*?”
- “Why can I name nodes in one place, but pack channels in another place?”

This fragmentation is a UX smell: the project is fundamentally a **procedural texturing tool** and should present one mental model:

> “Build a graph. Name your intermediate results. Declare which named results to write to disk.”

This is analogous to the audio unification strategy (single “audio” backend for both SFX and instruments): one recipe contract, many workflows.

## 2. Goals

- **Single texture recipe kind:** `texture.procedural_v1` is the only supported texture generator recipe.
- **Map-agnostic and unopinionated:** no reserved output names or semantics; outputs are arbitrary user-defined PNGs.
- **Composable DAG:** authors can build multi-stage graphs with named intermediate nodes.
- **Deterministic (Tier 1):** same spec + seed => byte-identical PNG output.
- **Supports “packing” without special cases:** packing is expressed as graph composition (RGBA construction), not as a separate recipe kind.
- **Clear migration story:** since SpecCade is pre-release, we accept a breaking change with a mechanical migration plan.

## 3. Non-goals (v1)

- Preserving `texture.material_v1` / `texture.normal_v1` / `texture.packed_v1` schemas.
- Shipping a full node editor UI.
- Providing every possible image processing operation in v1.
- Cross-spec dependency (e.g. “use the output of another spec as an input”).

## 4. Proposed Canonical Spec Shape

### 4.1 Recipe Kind

`texture.procedural_v1`

### 4.2 Parameters (`recipe.params`)

The parameters are a named-node DAG (identical in spirit to RFC-0005’s graph, but promoted as the single canonical texture recipe).

```json
{
  "resolution": [256, 256],
  "tileable": true,
  "nodes": [
    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
    { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.55 },
    { "id": "albedo", "type": "color_ramp", "input": "n", "ramp": ["#0b1020", "#e94560", "#f39c12"] },
    { "id": "packed", "type": "compose_rgba", "r": "mask", "g": "n", "b": "mask" }
  ]
}
```

### 4.3 Outputs (`outputs[]`)

Outputs are explicit bindings from file paths to named nodes.

```json
{
  "outputs": [
    { "kind": "primary", "format": "png", "path": "albedo.png", "source": "albedo" },
    { "kind": "primary", "format": "png", "path": "mask.png", "source": "mask" },
    { "kind": "primary", "format": "png", "path": "packed_orm.png", "source": "packed" }
  ]
}
```

Rules:

- Every `primary` output **must** have `format: "png"`.
- Every `primary` output **must** set `source` to a node `id`.
- The output bytes are produced by encoding the referenced node value:
  - grayscale node ⇒ grayscale PNG
  - color node ⇒ RGBA PNG

In this unified model, there is no special “packed output kind”. A “packed texture” is simply an RGBA PNG output whose bytes were produced by a packing-shaped subgraph (typically `compose_rgba`).

### 4.4 Why “packing” does not need special handling

Channel packing is not a distinct generator; it is just **RGBA construction**:

- build 3–4 grayscale nodes (or derive them)
- use `compose_rgba` to combine them into a color node
- output that node as a PNG

RFC-0002 introduced `texture.packed_v1` because packing needed a dedicated schema *before* we had a general-purpose graph IR. In the unified model, packing is simply one graph pattern among many.

## 5. Node Model

Each node produces exactly one value of one of these types:

- **Grayscale** (single channel, normalized `[0, 1]`)
- **Color** (RGBA, normalized `[0, 1]`)

Nodes refer to other nodes by `id`. The graph must be a DAG.

### 5.1 v1 Node Ops (initial set)

This RFC starts with the RFC-0005 v1 op set, with minor clarifications to treat it as the canonical texture tool.

Grayscale primitives:

- `constant { value }`
- `noise { noise }` (reuses `NoiseConfig`)
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

- `color_ramp { input, ramp: ["#RRGGBB", ...] }` (grayscale → color)
- `palette { input, palette: ["#RRGGBB", ...] }` (color → color)
- `to_grayscale { input }` (color → grayscale luminance)
- `compose_rgba { r, g, b, a? }` (grayscale ×4 → color)
- `normal_from_height { input, strength }` (grayscale → color normal map)

### 5.2 Tileability

`tileable: true` is a contract-level intent: the backend should produce seamless edges **where the op supports it**.

In v1:

- `noise` must be tileable when `tileable: true`.
- Deterministic patterns like `checkerboard` and `stripes` are naturally tileable when their parameters divide resolution cleanly; otherwise the wrap behavior is implementation-defined but must remain deterministic.
- Future ops should explicitly document tileable behavior.

## 6. Determinism

- Node-local seeds are derived from `(spec.seed, node.id)` to remain stable across node reorderings.
- PNG encoding uses the deterministic encoder (fixed compression/filter settings; no timestamps).

## 7. Validation (Spec-Level)

`texture.procedural_v1` validation must reject:

- empty `nodes`
- duplicate node ids
- references to missing node ids
- cycles (graph must be a DAG)
- obvious type mismatches (e.g. `palette` taking grayscale input) where practical
- missing `outputs[].source` for primary outputs
- `outputs[].source` pointing to missing node ids
- primary outputs whose `format` is not `png`

## 8. Breaking Changes

Adopting this RFC is a hard break:

- Remove recipe kinds:
  - `texture.material_v1`
  - `texture.normal_v1`
  - `texture.packed_v1`
  - `texture.graph_v1`
- Introduce recipe kind:
  - `texture.procedural_v1`
- Remove `outputs[].kind = "packed"` (packing becomes an RGBA graph pattern; all texture outputs are `primary` PNGs).
- Update documentation, schema, CLI dispatch, tests, and golden fixtures accordingly.

## 9. Migration Strategy

Because we are pre-release, we prioritize simplicity and clarity over compatibility.

### 9.1 Mechanical translations

- `texture.graph_v1` → `texture.procedural_v1`
  - direct rename of recipe kind (params and node ops remain the same)
- `texture.normal_v1` → `texture.procedural_v1`
  - express the old “height/pattern” as grayscale nodes
  - generate normal via `normal_from_height`
  - output the normal node
- `texture.packed_v1` → `texture.procedural_v1`
  - express each `maps[key]` as a grayscale node (or subgraph)
  - output a packed RGBA node via `compose_rgba`
- `texture.material_v1` → `texture.procedural_v1`
  - rewrite as a graph that emits multiple named outputs (albedo/roughness/metallic/normal/etc.)
  - note: any removed convenience “layer keywords” must be recreated either as graph templates or future ops

### 9.2 Optional tooling

- A `speccade migrate` mode can be added to perform the mechanical renames and provide best-effort graph rewrites for the older texture kinds, but it is not required to adopt this RFC.

## 10. Implementation Plan (Repository Work)

### 10.1 `speccade-spec`

- Add `texture.procedural_v1` recipe parsing and typed params.
- Update validation to treat `texture.procedural_v1` as the only supported texture recipe kind.
- Update JSON schema to:
  - include `texture.procedural_v1`
  - require/allow `outputs[].source` (already used for graph-style binding)
  - remove old texture recipe kinds
  - remove `outputs[].kind = "packed"` and the corresponding `channels` field path from the editor schema

### 10.2 `speccade-backend-texture`

- Rename/route the current graph backend as the canonical procedural generator.
- Ensure `tileable` semantics are documented and tested for `noise`.

### 10.3 `speccade-cli`

- Dispatch `texture.procedural_v1` to the procedural generator.
- Remove dispatch and availability checks for removed texture recipe kinds.

### 10.4 Golden fixtures + tests

- Rewrite all `golden/speccade/specs/texture/*.json` to use `texture.procedural_v1`.
- Update any test harness logic keyed on removed kinds.
- Maintain Tier 1 determinism checks via golden hashes for representative procedural specs.

### 10.5 Documentation

- Update:
  - `docs/spec-reference/texture.md`
  - `docs/SPEC_REFERENCE.md`
  - `PARITY_MATRIX.md`
  - any references in RFC-0001 and other docs

## 11. Alternatives Considered

### 11.1 Keep wrappers (`material_v1`, `normal_v1`, `packed_v1`) as “sugar”

Pros:

- less migration churn
- friendlier onboarding

Cons:

- perpetuates conceptual fragmentation (“which one do I use?”)
- creates multiple validation and behavior surfaces to maintain

### 11.2 Add a “bridge” without removing old kinds

Pros:

- minimal breakage

Cons:

- still leaves the user with multiple mental models and recipe choices

## 12. Future Work

- Add more ops (blur/warp/morphology/blend modes/UV transforms).
- Add graph libraries/templates (reusable subgraphs) without introducing new recipe kinds.
- Optional “component extract” / “swizzle” ops if we need richer packing workflows beyond grayscale-to-RGBA.
