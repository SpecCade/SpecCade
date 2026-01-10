# Future Generators + Production-Readiness Ideas

This is a forward-looking brainstorm document (not a spec contract).

## Matcap (`texture.matcap_v1`)

- Keywords: `light_dir`, `rim`, `toon_steps`, `roughness`, `metallic`, `fresnel`, `anisotropy`, `outline`, `ao`, `curvature`
- Algorithms: spherical mapping, stylized BRDF ramps, baked SH lighting, edge detection + curvature masks
- Enhancements: matcap-from-PBR (convert existing material maps), matcap library presets, color-grading LUTs

## Spritesheets (`sprite.sheet_v1`)

- Keywords: `palette`, `dither`, `outline`, `shading_ramp`, `hueshift`, `subpixel`, `silhouette`, `attachments`
- Algorithms: SDF-based shape synthesis → raster, palette quantization (median cut / k-means), ordered/Bayer dithering, edge-aware outlining
- Enhancements: multi-resolution export, normal/height for 2D lighting, per-sprite metadata (anchors, hitboxes)

## Sprite Animation (`sprite.animation_v1`)

- Keywords: `fps`, `events`, `loops`, `directions`, `blendspace`, `timing_curves`, `root_motion`, `constraints`
- Algorithms: parametric motion curves, pose-to-pose interpolation, smear frames, secondary motion (spring), procedural walk cycles
- Enhancements: export to common runtimes (JSON schemas), preview renders, automatic packing + trimming

## Texture/Material Expansion

- Trimsheets + atlases (packing, padding, mip-safe gutters), decals, splat maps for terrain, stylized hand-painted workflows.
- Smart masks: curvature/edge-wear, cavity, grime accumulation, slope-based masks.
- Additional noise/patterns: blue-noise masks, cracks, pores, fabric, stone stratification, moss growth, raindrop streaking.

## 2D VFX + UI Expansion

- VFX sprites: particles, smoke, fire, magic impacts, shockwaves, sparks, water splashes; plus flipbooks + emission masks.
- UI/icon generators: consistent icon sets, button frames, nine-slice panels, HUD elements, item cards.
- Font generation (bitmap/SDF): pixel fonts, outline/shadow variants, kerning tables, MSDF for crisp scaling.

## Mesh/Animation Expansion

- Modular kit generators (walls/doors/pipes), parametric LODs, collision mesh + navmesh hints.
- UV automation: texel density targets, seam heuristics, island packing, lightmap UVs.
- Baking: high→low normal/AO/curvature, vertex color baking, texture-space dilation.

## Audio/Music Expansion

- Batch SFX variation sets (seed sweeps + constraints), one-shot + loop pairing, loudness targets (LUFS), peak/true-peak limiting.
- Convolution reverb IR generation, foley layers, impulse/decay modeling, sample set export.
- Music: motif variation, pattern transformations, humanization with deterministic constraints, style presets.

## Pipeline “Powerhouse” Features

- Incremental builds (content-addressed cache keyed by canonical JSON + toolchain versions).
- Provenance + licensing: embed recipe + seed + commit hash into metadata, output manifests, SPDX-like license tagging.
- CI gates: `fmt`, `clippy`, determinism checks, golden hashes, performance budgets.
- Plugin API: external generators as WASM or subprocess backends with strict I/O contracts.
- Profiling + observability: per-stage timings, memory stats, reproducible performance runs.
- Quality controls: perceptual diffing (image/audio), duplicate detection, constraint solvers, property-based fuzzing for specs.
