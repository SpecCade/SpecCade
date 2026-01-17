# LLM Prompt-to-Asset Roadmap (SpecCade)

This document captures actionable, forward-looking work to make SpecCade easier to drive from an LLM/tooling loop: **prompt → spec → validate → generate → analyze → revise**.

This is **not** a spec contract. The canonical contract remains `docs/spec-reference/` + `crates/speccade-spec/`.

## Current State (What Already Helps)

- Starlark authoring (`.star`) compiles to canonical JSON IR, then goes through the same validation + hashing + backends as JSON specs.
- A Starlark stdlib exists for common structures (core scaffolding plus audio/texture/mesh/music helpers).
- Budget validation exists to prevent runaway specs (including `starlark_timeout_seconds`).
- Compiler/stdlib errors use stable S-series codes (S001–S006 compiler, S101–S104 stdlib argument validation).

These are necessary foundations: they reduce boilerplate and make failures more repairable than hand-editing raw JSON.

## Goal: A Reliable LLM Loop

The target experience:

1) Provide a short text prompt plus required constraints (target budget profile, loop length, output formats).
2) Tool generates a first-draft `.star` using real stdlib functions and/or curated templates.
3) Tool runs `eval`, `validate`, and `generate`, then runs `analyze` to score outputs.
4) Tool iterates with structured, minimal diffs until scores meet thresholds.

## Missing Pieces (Actionable Deliverables)

### 1) Machine-Readable API Surface (stdlib/schema dump)

Problem: LLMs hallucinate function names, enums, and parameter shapes when the API surface is only described in prose.

Deliverable: a command that emits the actual callable surface in a stable format:

- `speccade stdlib dump --format json` (or similar)
- Includes: function names, parameters (required/default), enums, ranges, return shape tags, examples, and stdlib version

Example output shape (illustrative):
```json
{
  "stdlib_version": "0.1.0",
  "domains": ["core", "audio", "texture", "mesh", "music"],
  "functions": [
    {
      "name": "oscillator",
      "domain": "audio",
      "params": [
        { "name": "frequency", "type": "float", "min": 0.0, "required": true },
        {
          "name": "waveform",
          "type": "enum",
          "values": ["sine", "square", "sawtooth", "triangle", "pulse"],
          "default": "sine"
        }
      ],
      "returns": "dict(audio.synthesis)",
      "examples": ["oscillator(440, \"sine\")"]
    }
  ]
}
```

### 2) Structured Errors for Repair Loops (`--json`)

Problem: “human-friendly” error text is hard to reliably parse and fix in an automated loop.

Deliverable: `--json` output modes for `eval`, `validate`, and `generate` that include:

- `code`, `category`, `path`, `message`
- `location` (file/line/col) when available
- `suggestions` (optional), such as enum alternatives or a minimal patch hint

Example validation error (illustrative):
```json
{
  "ok": false,
  "errors": [
    {
      "code": "budget.audio.allowed_sample_rates",
      "category": "budget",
      "path": "recipe.params.sample_rate",
      "message": "sample_rate 44100 not allowed (expected one of: 22050)",
      "location": { "file": "laser.star", "line": 18, "col": 28 },
      "suggestions": [{ "op": "replace", "value": 22050 }]
    }
  ]
}
```

### 3) Templates / Presets Beyond Textures

Problem: Without curated starting points, “valid” specs often sound/look bad on the first try.

Deliverable:

- Extend `speccade template` to cover at least `audio` and `music` (and optionally `mesh`) templates.
- Encourage “preset + overrides” patterns where a stable base is tweaked by a small set of parameters.

### 4) Higher-Level Starlark Constructors

Problem: Raw recipe dict construction has a large surface area for mistakes.

Deliverable: add helpers similar in spirit to `music_spec()` for other domains:

- `audio_spec(...)` for common SFX/instrument cases
- `texture_spec(...)` for common map sets / graph scaffolding
- `mesh_spec(...)` for common primitive + modifier cases

### 5) Quality Metrics: `analyze`

Problem: Budget/validation correctness is not quality. You need feedback signals to converge.

Deliverable: `speccade analyze --spec ... --out-root ... --json` that emits metrics suitable for iteration.

Example metrics (illustrative):
```json
{
  "asset_id": "laser-01",
  "audio": {
    "sample_rate": 22050,
    "peak_dbfs": -0.3,
    "clipped_samples": 0,
    "dc_offset": 0.001,
    "integrated_lufs": -16.5
  }
}
```

Suggested minimum metrics:
- Audio: peak/clipping, DC offset, loudness proxy, duration, basic spectral centroid/rolloff.
- Textures: resolution, tileability checks, histogram/contrast stats, simple artifact detectors.
- Music: structural validity checks (loop flags, pattern size limits), and “sanity” checks (no empty patterns, instrument refs resolve).
- Mesh: bounds/scale, vertex/face counts, bone counts; for Blender tier, validated metrics rather than byte-identical hashes.

### 6) Preview Artifacts (Human + Tool Loop)

Deliverable: implement `speccade preview` or ensure generation can emit standardized preview artifacts:

- Audio: waveform PNG + short spectrogram PNG.
- Textures: thumbnail PNGs.
- Mesh: simple viewer hook or render thumbnails (tier-2 caveats).

Tooling note: preview artifacts become powerful when paired with `analyze` (so the tool can “see” results numerically).

### 7) Iteration Speed: Content-Addressed Cache

Deliverable: cache generation outputs by canonical spec/recipe hash + backend versions so repeated attempts are cheap.

This matters because LLM loops often do many small edits.

### 8) Budget Profiles That Match Nethercore (Without Misleading Labels)

Existing profiles include `default`, `strict`, and `zx-8bit`. The `zx-8bit` name is not about your console’s “bitness”; it’s just a tight profile.

Deliverable: add a Nethercore-oriented budget profile (e.g. `nethercore`) that enforces **22050 Hz audio** (and any other runtime constraints) without inheriting overly strict “8-bit” limits on durations/layers/texture sizes.

## Execution Path (Pragmatic Order)

1) Implement `stdlib dump --format json` and `--json` structured errors (unblocks robust tool use).
2) Add a minimal `analyze --json` for audio + textures first (fastest ROI for iterative quality).
3) Expand templates/presets across domains + add `audio_spec()`/`texture_spec()` constructors (reduces first-draft failure rate).
4) Add caching keyed by canonical hashes (makes loops cheap).
5) Build a small orchestrator (separate tool or CLI subcommand) that runs the loop and applies minimal patches.

## Are We “100% Ready” After This?

Not quite. These changes remove most of the *mechanical/tooling* friction, but some pain remains:

- Subjective quality and art direction: metrics help, but “on-style” needs curated presets/kits and/or human review.
- Multi-asset coherence: generating a cohesive set (music + instruments + SFX + materials + meshes) needs shared style constraints and reuse patterns (refs/kits), not just per-asset correctness.
- Tier-2 variability: Blender-backed assets are harder to make deterministic and to score automatically.
- Capability gaps: if a backend/recipe cannot express a desired look/sound, no amount of prompting fixes that; it becomes a generator/stdlib expansion task.

Treat “LLM readiness” as: (1) validity, (2) iteration speed, (3) measurable quality convergence, (4) curated art direction defaults.
