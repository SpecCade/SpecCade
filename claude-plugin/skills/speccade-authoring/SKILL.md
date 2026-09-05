---
name: speccade-authoring
description: >-
  Authoring SpecCade specs for deterministic asset generation. Covers audio
  synthesis (16+ types), texture procedural nodes, tracker music (XM/IT format),
  mesh generation, and the Starlark/JSON spec format. Use when creating, editing,
  or debugging sound, texture, music, or 3D mesh specs.
license: MIT
compatibility: Requires speccade CLI installed.
metadata:
  author: nethercore-systems
  version: "1.0.1"
---

# SpecCade Authoring

Author assets **with** SpecCade; do not change its implementation merely to make a recipe pass. Prefer Starlark helpers to hand-maintained JSON. JSON is supported when requested.

## Authority and first inspection

- Current Rust types/validation in `crates/speccade-spec/src/recipe/` define the contract.
- Run `speccade --version`, `speccade doctor`, and relevant `--help` against the executable you will use. A binary in a dirty developer checkout may not match published source.
- `speccade stdlib dump --format json` exposes current helper names, argument names and defaults. Do not guess them from a related synthesizer or Blender API.
- Reuse the nearest checked-in `specs/` or `packs/preset_library_v1/` example. Validate it at this revision before adapting it. Editor schemas and older prose are secondary.

## Small authoring loop

1. Fix the intended asset, duration/dimensions, target format, seed and output path. Keep the editable `.star` source.
2. Evaluate the helpers, then validate the complete spec before generation.
3. Generate into an explicit output root, read the report and decode the actual output. Never treat a schema pass as backend success.
4. Import/pack and exercise the asset in the target application. Listening/viewing decides quality; format and budget checks do not.

For **Nethercore ZX**, these commands use its `nethercore` budget. Other targets should choose their own budget rather than inheriting ZX restrictions:

```bash
speccade eval --spec asset.star --pretty
speccade validate --spec asset.star --budget nethercore --json
speccade generate --spec asset.star --out-root ./output --budget nethercore --json
```

`nethercore` and `zx-8bit` are distinct presets. No bulk generation until one representative asset works end to end.

## Minimal ZX audio starting point

Adapted from `specs/audio/audio_filter_bandpass.star`, with target sample rate explicit:

```starlark
spec(
    asset_id = "zx-bandpass-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/bandpass.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.5,
            "sample_rate": 22050,
            "layers": [audio_layer(
                synthesis = noise_burst("white"),
                envelope = envelope(0.05, 0.3, 0.7, 0.4),
                volume = 0.7,
                filter = bandpass(1000, 4.0, 4000),
            )],
        },
    },
)
```

Verify generated WAV metadata: mono, signed 16-bit PCM, 22,050 Hz. The current Nethercore raw WAV packer does not safely infer/convert an arbitrary source format.

## Routing

- Audio synthesis/filtering: [audio-synthesis](references/audio-synthesis.md), [effects](references/audio-effects.md).
- PNG/procedural materials: [texture nodes](references/texture-nodes.md). Each output's `source` selects its terminal node ID.
- Tracker music: [tracker](references/music-tracker.md), [compose IR](references/music-compose-ir.md).
- GLB meshes/rigs: [Blender](references/mesh-blender.md); canonical character guidance is `docs/spec-reference/character.md` and `crates/speccade-spec/src/recipe/character/`.
- Spec/report conventions: [spec format](references/spec-format.md).
- **ZX integration:** [Nethercore boundary](references/nethercore-zx-integration.md): raw inputs, stable manifest IDs, tracker sample extraction, WAV and GLB limits, runtime gates.

## Determinism and delivery

- Tier 1 Rust backends: compare bytes for the same validated source/spec/seed and pinned implementation/dependencies. Determinism is not a promise across versions.
- Tier 2 Blender backends: pin Blender; validate triangle/bone/frame counts, bounds and expected animation. Do not demand byte-identical GLB.
- Do not silently replace a failed backend or asset with a placeholder. Keep errors attached to the real attempted recipe; make one directed correction.
- Preserve authored sources and useful reports. Avoid retaining duplicate generated batches. Deliver only verified artifacts; keep sensory acceptance separate.
