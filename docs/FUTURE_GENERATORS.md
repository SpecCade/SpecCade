# Future Generators + Production-Readiness Ideas

This is a forward-looking brainstorm document (not a spec contract).

Legend (informal triage tags used below):

- `[Q]` = quick win / big payoff relative to cost
- `[I]` = isolated improvement / low-risk “pure add”
- `[G]` = significant gap-filler / capability unlock

## Matcap (`texture.matcap_v1`)

- Keywords: `light_dir`, `rim`, `toon_steps`, `roughness`, `metallic`, `fresnel`, `anisotropy`, `outline`, `ao`, `curvature`
- Algorithms: spherical mapping, stylized BRDF ramps, baked SH lighting, edge detection + curvature masks
- Ideas / Enhancements:
  - `[Q]` `toon_ramp` / `toon_steps` with optional per-step tinting (fast stylized variety)
  - `[Q]` `outline` via Sobel/edge detect on normal/height with thickness + threshold controls
  - `[Q]` Curvature/cavity masks (derived from sphere normal) as first-class outputs/drivers for wear/grime
  - `[I]` `matcap_preset` library + “preset + overrides” pattern for stable art direction
  - `[I]` Optional post stack: `lut`, `vignette`, `film_grain`, `chromatic_aberration`
  - `[G]` `matcap_from_pbr`: render/bake a sphere from `texture.procedural_v1` outputs into a matcap
  - `[G]` “Studio rig” matcaps: 2–3 key lights + rim + ambient (bigger look range than single-light)

## Spritesheets (`sprite.sheet_v1`)

- Keywords: `palette`, `dither`, `outline`, `shading_ramp`, `hueshift`, `subpixel`, `silhouette`, `attachments`
- Algorithms: SDF-based shape synthesis → raster, palette quantization (median cut / k-means), ordered/Bayer dithering, edge-aware outlining
- Ideas / Enhancements:
  - `[Q]` Deterministic packing with padding/mip gutters + atlas metadata (`*.json`: frames, pivots, rects)
  - `[Q]` Auto-trim + consistent pivot modes (`center`, `feet`, `custom`) + optional pixel snapping
  - `[Q]` Palette constraints: `max_colors`, `locked_colors[]`, “preserve skin tones” heuristics
  - `[Q]` Dither modes: Bayer (ordered), blue-noise, none; plus per-channel dithering
  - `[I]` Optional SDF/MSDF export alongside raster for crisp scaling + outline/glow in-engine
  - `[I]` Multi-resolution export (`1x/2x/4x`) with shared palette + consistent snapping rules
  - `[G]` First-class attachments: named anchors, hitboxes/hurtboxes, emitter points
  - `[G]` Optional extra per-frame maps: `emissive_mask`, `normal`, `height` for 2D lighting pipelines

## Sprite Animation (`sprite.animation_v1`)

- Keywords: `fps`, `events`, `loops`, `directions`, `blendspace`, `timing_curves`, `root_motion`, `constraints`
- Algorithms: parametric motion curves, pose-to-pose interpolation, smear frames, secondary motion (spring), procedural walk cycles
- Ideas / Enhancements:
  - `[Q]` `timing_curves` for ease/hold + deterministic retiming for punchier motion
  - `[Q]` Event tracks (`events[]`) for SFX/FX/attack windows + gameplay sync
  - `[I]` Loop sealing: auto-adjust last N frames to match first (or generate crossfade frames)
  - `[I]` Smear/trails: procedural smear frames from motion vectors + tunable intensity
  - `[G]` `blendspace`: generate directional/speed variants + export blend tables
  - `[G]` Secondary motion: spring constraints on named bones/parts (hair, cloth, antennae)
  - `[G]` Root motion authoring/extraction (especially important for top-down movement feel)
  - `[G]` Export to common runtimes (JSON schemas), preview renders, automatic packing + trimming

## Texture/Material Expansion

- `[I]` Unified procedural texture graph (`texture.procedural_v1`) for named-map workflows (implemented)
- `[Q]` More procedural ops for `texture.procedural_v1`:
  - Cracks, pitting, pores, rust, moss, stains, water streaks, fingerprints
  - Blend modes, warp/transform, blur, morphology, edge detection
  - Smart masks (curvature, cavity, slope, edge-distance) derived from height/normal
- `[Q]` Stochastic tiling (Wang tiles / texture bombing) to reduce visible repetition
- `[Q]` Additional noise/pattern primitives: blue-noise masks, fabric/weave, stone stratification, raindrop streaking
- `[I]` Trimsheets + atlases: packing, padding, mip-safe gutters, labeled strips + metadata
  - Candidate recipe: `texture.trimsheet_v1`
- `[I]` Decals: RGBA decal + optional normal/roughness + placement metadata
  - Candidate recipe: `texture.decal_v1`
- `[G]` Terrain splat sets: albedo/normal/roughness + splat masks + macro variation
  - Candidate recipe: `texture.splat_set_v1`
- `[G]` Stylized hand-painted helpers: edge highlights + ambient shading ramp + hue drift (big throughput win)
- `[G]` Material preset system: “preset + parameterization” for stable art direction + cheap variety
- `[G]` Preview/thumbnail outputs once `preview` output kind is supported by schema/validation

## 2D VFX + UI Expansion

- VFX sprites (flipbooks + masks):
  - `[Q]` Deterministic flipbook generators for common effects: smoke puff, spark burst, shockwave, splash, magic impact
    - Candidate recipes: `vfx.flipbook_v1`, `vfx.smoke_puff_v1`, `vfx.spark_burst_v1`, `vfx.shockwave_v1`
  - `[G]` Shared particle “material” presets: additive/soft/distort/etc.
    - Candidate recipe: `vfx.particle_profile_v1`
- UI/icon generators:
  - `[Q]` Nine-slice panel generator with corner rules + palette variants
    - Candidate recipe: `ui.nine_slice_v1`
  - `[Q]` Icon set generator with consistent stroke/shape language + theme palettes
    - Candidate recipe: `ui.icon_set_v1`
  - `[I]` UI kit presets (buttons, panels, badges, progress bars) to standardize look and speed up production
  - `[I]` Item card templates with slots (icon, rarity border, background pattern)
    - Candidate recipe: `ui.item_card_v1`
  - `[G]` Deterministic “damage numbers” sprites (font + outline + crit styles) for game juice
- Font generation:
  - `[I]` Bitmap pixel fonts + outline/shadow variants + kerning tables
    - Candidate recipe: `font.bitmap_v1`
  - `[I]` MSDF fonts for crisp scaling + JSON metrics
    - Candidate recipe: `font.msdf_v1`

## Mesh/Animation Expansion

- `[Q]` Expose a curated `modifier_stack[]` for `static_mesh.blender_primitives_v1`:
  - `mirror`, `solidify`, `bevel`, `subdivide`, `array`, `triangulate`
- `[Q]` UV automation: unwrap + pack with texel-density targets (plus optional lightmap UVs)
- `[Q]` Normals automation: `auto_smooth`, weighted normals, hard-edge-by-angle presets
- `[I]` Parametric LODs: deterministic decimate to target triangle counts + validate bounds/tri metrics
- `[I]` Collision mesh generation: convex hull / simplified mesh outputs (as extra output or separate asset type)
- `[I]` Navmesh hints/metadata: walkable surfaces, slope/stair tagging, “no-walk” volumes (engine integration glue)
- `[I]` Modular kit generators (walls/doors/pipes) built from primitives + modifiers (content explosion)
- `[G]` Organic modeling gap-fill: metaballs → remesh → smooth → displacement noise (rocks/creatures/organic props)
- `[G]` Shrinkwrap workflows: wrap armor/clothes onto body parts (strict limits + validation for stability)
- `[G]` Boolean kitbashing: union/difference + cleanup for “greeble” style modeling
- `[G]` Baking suite: high→low normal/AO/curvature, vertex color baking, texture-space dilation
- `[G]` Render-to-sprite bridge: render a `static_mesh` with lighting preset → `sprite.sheet_v1`
- `[G]` Animation helpers: IK targets + constraint presets for procedural walk/run cycles

## Audio/Music Expansion

- Audio (`audio_v1`):
  - `[Q]` Wavetable oscillator (tables + interpolation + detune/unison)
  - `[Q]` Granular synth (grain size/rate/jitter/window) using generated buffers (no external samples required)
  - `[Q]` Modulators: LFOs for pitch/amp/filter + envelope curve types (linear/exp/log)
  - `[I]` Effect chain (deterministic DSP): delay, reverb (algo), chorus, phaser, bitcrush, waveshaper, compressor
  - `[I]` Loudness targets (LUFS) + true-peak limiter (production-ready loudness control)
  - `[I]` Better loop point generation: zero-crossing search + crossfade loops for sustained instruments
  - `[I]` One-shot + loop pairing (transient + loopable sustain from the same recipe)
  - `[G]` Foley layering helpers: impact builder (transient/body/tail) + whoosh builder (noise + sweep)
  - `[G]` Convolution reverb IR generation + apply as effect (big realism jump)
  - `[G]` Impulse/decay modeling (room/plate/spring style) for more realistic tails without hand-tuning
  - `[G]` Batch SFX variation sets: seed sweeps + constraints + sample set export
- Music (tracker + compose IR):
  - `[Q]` Expand `effect_name` support + validation (arp, porta, vibrato, retrig, vol slide, etc.)
  - `[Q]` Deterministic swing/humanize macros in Pattern IR (timing + velocity ranges)
  - `[I]` Motif transforms: transpose/rotate/invert/stretch with constrained randomness
  - `[I]` Cue templates: `loop_low/loop_main/loop_hi` + stingers + transitions (compile-time helpers)
  - `[G]` Genre-kit integration: choose kit + tempo + intensity → generate arrangement skeleton
  - `[G]` Style presets: curated defaults for kit + arrangement templates + mix-ish constraints
  - `[G]` Harmony helpers: scale/chord constraints + voice-leading guardrails to reduce dissonance failures
  - `[G]` Fill generator: bar-end drum fills + risers driven by section boundaries
  - `[G]` Enforce sample/preset role aliasing per kit (integration + reuse)

## Pipeline “Powerhouse” Features

- `[I]` Incremental builds: content-addressed cache keyed by canonical JSON + toolchain/backend versions
- `[I]` CI gates: `fmt`, `clippy`, determinism checks, golden hashes, performance budgets
- `[I]` Quality controls: perceptual diffing (image SSIM/ΔE; audio loudness/spectral), duplicate detection
- `[I]` Preset registry: stable preset IDs + “preset + overrides” across audio/texture/modifier stacks
- `[I]` Profiling + observability: per-stage timings, memory stats, reproducible performance runs
- `[G]` Plugin API: external generators as WASM or subprocess backends with strict I/O contracts + determinism reporting
- `[G]` “Bridge specs”: mesh → sprites, materials → packed textures, etc., without bespoke glue code
- `[G]` Constraint solvers + property-based fuzzing for spec validation and generator robustness
