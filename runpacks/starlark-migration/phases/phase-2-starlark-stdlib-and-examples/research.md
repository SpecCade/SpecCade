# Phase 2 Research: Starlark Stdlib and Examples

## 1. Phase 1 Compiler Entry Points for Stdlib

### Current Architecture

The Phase 1 compiler is structured as follows:

```
crates/speccade-cli/src/compiler/
  mod.rs    - Public API: compile(), CompilerConfig, STDLIB_VERSION
  eval.rs   - Starlark evaluation with timeout
  convert.rs - Starlark Value -> JSON conversion
  error.rs  - CompileError enum
```

### Key Entry Point for Stdlib

The stdlib should be registered in `eval.rs` at the `Globals::standard()` call:

```rust
// Current code (eval.rs:51):
let globals = Globals::standard();

// To add stdlib builtins:
let globals = GlobalsBuilder::standard()
    .with(speccade_stdlib)
    .build();
```

The `starlark` crate provides `GlobalsBuilder` for extending the global environment with custom functions. Functions are registered using the `#[starlark_module]` attribute macro.

### How to Add Stdlib Builtins

1. **Create stdlib module**: `crates/speccade-cli/src/compiler/stdlib.rs` (or separate crate `crates/speccade-starlark-stdlib/`)

2. **Register functions using starlark macros**:
   ```rust
   use starlark::starlark_module;
   use starlark::environment::GlobalsBuilder;
   use starlark::values::Value;

   #[starlark_module]
   fn speccade_stdlib(builder: &mut GlobalsBuilder) {
       fn audio_layer(
           synthesis: Value,
           envelope: Value,
           volume: f64,
           pan: f64,
       ) -> anyhow::Result<Dict> {
           // Build and return a dict
       }
   }
   ```

3. **Wire into evaluation**: Modify `eval.rs` to include stdlib in globals.

### Safety Limits (Phase 1)

- **Timeout**: 30 seconds default (`DEFAULT_TIMEOUT_SECONDS`)
- **No external loads**: `enable_load: false` in dialect config
- **No recursion**: Standard Starlark dialect disables recursion by default
- **No memory limit**: Documented as Phase 3 work item in `followups.md`

---

## 2. Complete List of IR Types by Asset Category

### Audio (`audio_v1`)

**Top-level params** (`AudioV1Params`):
- `base_note`: Optional note spec (MIDI number or note name like "C4")
- `duration_seconds`: f64
- `sample_rate`: u32 (22050, 44100, 48000)
- `layers`: Vec<AudioLayer>
- `pitch_envelope`: Optional PitchEnvelope
- `generate_loop_points`: bool
- `master_filter`: Optional Filter
- `effects`: Vec<Effect>
- `post_fx_lfos`: Vec<LfoModulation>

**AudioLayer fields**:
- `synthesis`: Synthesis enum (see below)
- `envelope`: Envelope (attack, decay, sustain, release)
- `volume`: f64 (0.0-1.0)
- `pan`: f64 (-1.0 to 1.0)
- `delay`: Optional f64
- `filter`: Optional Filter
- `lfo`: Optional LfoModulation

**Synthesis variants** (28 types):
- Basic: `Oscillator`, `NoiseBurst`, `Additive`, `MultiOscillator`
- FM/AM: `FmSynth`, `AmSynth`, `RingModSynth`, `FeedbackFm`
- Physical: `KarplusStrong`, `BowedString`, `Waveguide`, `MembraneDrum`
- Modal/Metallic: `Modal`, `Metallic`, `CombFilterSynth`
- Granular/Wavetable: `Granular`, `Wavetable`, `Pulsar`, `SpectralFreeze`
- Advanced: `PdSynth`, `Vocoder`, `Formant`, `Vector`, `SupersawUnison`, `Vosim`
- Other: `PitchedBody`

**Waveform types**: `Sine`, `Sawtooth`, `Square`, `Triangle`, `Pulse`, `Noise`

**Filter types**: `Lowpass`, `Highpass`, `Bandpass`, `Notch`

**Effect types**: Compressor, Delay, Reverb, Chorus, Flanger, Phaser, Eq, Waveshaper, Bitcrusher, Cabinet, StereoWidener

### Texture (`texture.procedural_v1`)

**Top-level params** (`TextureProceduralV1Params`):
- `resolution`: [u32; 2]
- `tileable`: bool
- `nodes`: Vec<TextureProceduralNode>

**Node operations** (`TextureProceduralOp`):
- Primitives: `Constant`, `Noise`, `Gradient`, `Stripes`, `Checkerboard`
- Grayscale ops: `Invert`, `Clamp`, `Add`, `Multiply`, `Lerp`, `Threshold`
- Color ops: `ToGrayscale`, `ColorRamp`, `Palette`, `ComposeRgba`, `NormalFromHeight`

**Noise configuration**:
- `algorithm`: Perlin, Simplex, Worley, Value, Fbm
- `scale`, `octaves`, `persistence`, `lacunarity`

### Static Mesh (`static_mesh.blender_primitives_v1`)

**Top-level params** (`StaticMeshBlenderPrimitivesV1Params`):
- `base_primitive`: MeshPrimitive enum
- `dimensions`: [f64; 3]
- `modifiers`: Vec<MeshModifier>
- `uv_projection`: Optional UvProjection
- `material_slots`: Vec<MaterialSlot>
- `export`: Optional MeshExportSettings
- `constraints`: Optional MeshConstraints

**MeshPrimitive types**: Cube, Sphere, Cylinder, Cone, Torus, Plane, IcoSphere

**MeshModifier types**: Bevel, Subdivision, Decimate, Mirror, Array, Solidify

### Music (`music.tracker_song_v1`, `music.tracker_song_compose_v1`)

**Top-level params** (`MusicTrackerSongV1Params`):
- `name`, `title`: Optional strings
- `format`: TrackerFormat (xm, it)
- `bpm`: u16 (30-300)
- `speed`: u8 (1-31)
- `channels`: u8 (XM: 1-32, IT: 1-64)
- `loop`: bool
- `restart_position`: Optional u16
- `instruments`: Vec<TrackerInstrument>
- `patterns`: HashMap<String, TrackerPattern>
- `arrangement`: Vec<ArrangementEntry>
- `automation`: Vec<AutomationEntry>
- `it_options`: Optional ItOptions

---

## 3. Existing Example Patterns

### Golden Starlark Examples (Phase 1)

**`golden/starlark/minimal.star`** - Bare-minimum spec:
```python
{
    "spec_version": 1,
    "asset_id": "starlark-minimal-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [...]
}
```

**`golden/starlark/with_functions.star`** - Helper functions:
```python
def make_output(name, format = "wav"):
    return {...}

BASE_ID = "starlark-functions"
COMMON_TAGS = ["retro", "scifi", "effects"]

{
    "spec_version": 1,
    "asset_id": BASE_ID + "-01",
    "style_tags": COMMON_TAGS + ["laser"],
    "outputs": [make_output("laser_blast")]
}
```

**`golden/starlark/with_comprehensions.star`** - List comprehensions:
```python
file_names = ["kick", "snare", "hihat", "tom"]
outputs = [{"kind": "primary", "format": "wav", "path": "drums/" + name + ".wav"} for name in file_names]
style_tags = [tag.upper() for tag in raw_tags]
```

### JSON Pack Examples

**`packs/preset_library_v1/audio/bass/bass_808.json`**:
- Multi-layer audio with oscillators, FM, PD synthesis
- Effect chain (waveshaper, compressor)
- Full recipe structure

**`packs/preset_library_v1/texture/preset_texture_noise_height_basic.json`**:
- Procedural texture graph with single noise node
- Output references node by id ("height")

### Pattern Observations

1. **Layered construction**: Audio specs commonly use 2-5 layers for complex sounds
2. **Naming conventions**: `asset_id` uses kebab-case with numeric suffix
3. **Output paths**: Follow `{category}/{name}.{ext}` pattern
4. **Recipes are verbose**: Current JSON recipes are 50-150 lines for typical sounds

---

## 4. Recommended Stdlib Function Signatures

### Design Principles

1. **Flat, explicit parameters** - LLMs understand positional/keyword args better than nested dicts
2. **Sensible defaults** - Most params should be optional with good defaults
3. **Composable** - Functions return dicts that can be modified or combined
4. **Deterministic** - No random, no time, no network access

### Core Helpers

```python
# Spec scaffolding
def spec(asset_id, asset_type, seed, outputs, recipe=None, description=None, tags=None):
    """Create a complete spec dict."""

def output(path, format, kind="primary"):
    """Create an output specification."""
```

### Audio Helpers

```python
# Layer builders
def audio_layer(synthesis, envelope=None, volume=0.8, pan=0.0, filter=None, lfo=None):
    """Create an audio synthesis layer."""

def envelope(attack=0.01, decay=0.1, sustain=0.5, release=0.2):
    """Create an ADSR envelope."""

# Synthesis shortcuts
def oscillator(freq, waveform="sine", sweep_to=None, curve="linear"):
    """Create an oscillator synthesis block."""

def fm_synth(carrier, modulator, index, sweep_to=None):
    """Create FM synthesis block."""

def noise_burst(noise_type="white", filter=None):
    """Create noise burst synthesis."""

def karplus_strong(freq, decay=0.99, blend=0.5):
    """Create Karplus-Strong plucked string synthesis."""

# Filter helpers
def lowpass(cutoff, resonance=0.707, sweep_to=None):
    """Create lowpass filter."""

def highpass(cutoff, resonance=0.707, sweep_to=None):
    """Create highpass filter."""

# Effect helpers
def reverb(decay=0.5, wet=0.3, room_size=0.8):
    """Create reverb effect."""

def delay(time_ms=250, feedback=0.4, wet=0.3):
    """Create delay effect."""

def compressor(threshold_db=-12, ratio=4, attack_ms=10, release_ms=100):
    """Create compressor effect."""
```

### Texture Helpers

```python
def texture_node(id, op_type, **params):
    """Create a texture graph node."""

def noise_node(id, algorithm="perlin", scale=0.1, octaves=4):
    """Create a noise texture node."""

def gradient_node(id, direction="horizontal", start=0.0, end=1.0):
    """Create a gradient texture node."""

def color_ramp_node(id, input_node, ramp):
    """Create a color ramp node."""

def threshold_node(id, input_node, threshold=0.5):
    """Create a threshold node."""
```

### Mesh Helpers (Tier 2 - Lower Priority)

```python
def mesh_primitive(primitive, dimensions, modifiers=None):
    """Create a static mesh recipe."""

def bevel_modifier(width=0.02, segments=2):
    """Create a bevel modifier."""

def subdivision_modifier(levels=2):
    """Create a subdivision modifier."""
```

---

## 5. Error Code Format Recommendation

### Current System

SpecCade already has a well-designed error code system in `speccade-spec/src/error.rs`:

- **Validation errors**: E001-E023 (spec contract, recipe, packed output errors)
- **Warnings**: W001-W004 (missing license, description, etc.)
- **Backend errors**: Category-prefixed (AUDIO_001, TEXTURE_002, etc.)

### Recommendation for Starlark Compiler Errors

Extend the existing pattern with a new range for compiler/stdlib errors:

| Code | Category | Description |
|------|----------|-------------|
| S001 | Syntax | Starlark syntax error |
| S002 | Runtime | Starlark runtime error (undefined variable, etc.) |
| S003 | Timeout | Evaluation timed out |
| S004 | Type | Result is not a dict |
| S005 | Convert | Cannot convert Starlark value to JSON |
| S006 | Schema | Resulting JSON does not match Spec schema |
| S101 | Stdlib | Invalid stdlib function argument |
| S102 | Stdlib | Type mismatch in stdlib function |
| S103 | Stdlib | Value out of range in stdlib function |

### Error Message Format

```
S001: syntax error at spec.star:15:3: expected '}', got ':'
S101: audio_layer(): 'volume' must be between 0.0 and 1.0, got 1.5
S006: invalid spec: missing required field 'asset_id' (at root)
```

### Machine-Readable Format (--json)

```json
{
  "error": {
    "code": "S001",
    "message": "expected '}', got ':'",
    "location": {
      "file": "spec.star",
      "line": 15,
      "column": 3
    }
  }
}
```

---

## 6. Golden Test Implementation Strategy

### Current Testing Infrastructure

`speccade-tests` provides:
- `DeterminismFixture` / `DeterminismBuilder` for multi-run determinism checks
- Format validators for WAV, PNG, XM, IT, glTF
- `load_spec()` from `speccade-cli` for JSON/Starlark input

### Phase 2 Golden Test Strategy

#### 1. IR Equality Tests

For each `.star` example, create a `.expected.json` with the canonical IR:

```
golden/starlark/
  audio_synth_basic.star
  audio_synth_basic.expected.json
  texture_noise_perlin.star
  texture_noise_perlin.expected.json
```

Test structure:
```rust
#[test]
fn golden_audio_synth_basic() {
    let star_result = load_spec("golden/starlark/audio_synth_basic.star").unwrap();
    let expected = std::fs::read_to_string("golden/starlark/audio_synth_basic.expected.json").unwrap();
    let expected_spec: Spec = serde_json::from_str(&expected).unwrap();

    // Compare canonical JSON (byte-identical)
    assert_eq!(star_result.spec.to_json().unwrap(), expected_spec.to_json().unwrap());
}
```

#### 2. Stdlib Function Unit Tests

In `crates/speccade-cli/src/compiler/stdlib.rs` (or stdlib crate):

```rust
#[test]
fn test_envelope_defaults() {
    let result = eval_expr("envelope()");
    assert_eq!(result["attack"], 0.01);
    assert_eq!(result["decay"], 0.1);
    // ...
}

#[test]
fn test_oscillator_with_sweep() {
    let result = eval_expr("oscillator(440, waveform='saw', sweep_to=220)");
    assert_eq!(result["frequency"], 440.0);
    assert_eq!(result["freq_sweep"]["end_freq"], 220.0);
}
```

#### 3. Determinism Tests

Extend existing `e2e_determinism.rs` pattern:

```rust
#[test]
fn starlark_spec_generation_is_deterministic() {
    let fixture = DeterminismFixture::new()
        .add_spec("golden/starlark/audio_synth_basic.star")
        .add_spec("golden/starlark/texture_noise_perlin.star")
        .runs(3);
    let report = fixture.run();
    assert!(report.all_deterministic());
}
```

#### 4. Test Organization

```
crates/speccade-tests/
  tests/
    starlark_input.rs        # Phase 1 tests (existing)
    starlark_stdlib.rs       # Phase 2 stdlib unit tests
    starlark_golden.rs       # Phase 2 golden IR equality tests
    starlark_determinism.rs  # Phase 2 generation determinism tests
```

### Golden File Update Workflow

For updating golden files when stdlib changes:

```bash
# Regenerate expected JSON from Starlark
speccade eval golden/starlark/audio_synth_basic.star > golden/starlark/audio_synth_basic.expected.json
```

Consider adding `--update-golden` flag to test harness for batch updates.

---

## 7. LLM-Friendly Patterns (Bazel/Buck2 Comparison)

### Observations from Bazel/Buck2

1. **Flat function calls**: `cc_library(name = "foo", srcs = [...], deps = [...])`
2. **String labels for references**: `"//pkg:target"` instead of object references
3. **Explicit over implicit**: All inputs must be declared
4. **Keyword arguments**: Named parameters for clarity
5. **No complex control flow**: Simple variable binding and list comprehensions

### Application to SpecCade Stdlib

**Good (LLM-friendly)**:
```python
audio_layer(
    synthesis = oscillator(440, waveform = "saw"),
    envelope = envelope(attack = 0.01, decay = 0.2),
    volume = 0.8,
    pan = 0.0,
)
```

**Avoid (harder for LLMs)**:
```python
AudioLayer({
    "synthesis": {"type": "oscillator", "waveform": "saw", ...},
    "envelope": {"attack": 0.01, ...}
})
```

### Error Messages for LLM Correction

Errors should include:
1. **Error code** for categorization
2. **Field path** pointing to the problem
3. **Expected vs actual** for type/range errors
4. **Suggestion** when obvious fix exists

Example:
```
S102: oscillator(): 'waveform' must be one of: sine, sawtooth, square, triangle, pulse, noise
  Got: "sinwave"
  Did you mean: "sine"?
```
