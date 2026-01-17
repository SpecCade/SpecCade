# Phase 2 Interfaces: Stdlib API and Error Codes

## Stdlib Function Signatures

### Core Functions

```python
def spec(
    asset_id: str,           # Required: kebab-case identifier
    asset_type: str,         # Required: "audio", "texture", "static_mesh", etc.
    seed: int,               # Required: 0 to 2^32-1
    outputs: list[dict],     # Required: at least one output with kind="primary"
    recipe: dict = None,     # Optional: recipe specification
    description: str = None, # Optional: asset description
    tags: list[str] = None,  # Optional: style tags
    license: str = "CC0-1.0" # Optional: SPDX license identifier
) -> dict:
    """
    Create a complete spec dictionary.

    Returns a dict matching the Spec IR structure with spec_version: 1.
    """
    pass

def output(
    path: str,               # Required: output file path (e.g., "sounds/laser.wav")
    format: str,             # Required: "wav", "png", "glb", etc.
    kind: str = "primary"    # Optional: "primary" or "secondary"
) -> dict:
    """
    Create an output specification.

    Returns:
        {"kind": kind, "format": format, "path": path}
    """
    pass
```

### Audio Functions

```python
def envelope(
    attack: float = 0.01,    # Attack time in seconds
    decay: float = 0.1,      # Decay time in seconds
    sustain: float = 0.5,    # Sustain level (0.0-1.0)
    release: float = 0.2     # Release time in seconds
) -> dict:
    """
    Create an ADSR envelope.

    Returns:
        {"attack": attack, "decay": decay, "sustain": sustain, "release": release}
    """
    pass

def oscillator(
    frequency: float,             # Required: frequency in Hz (must be positive)
    waveform: str = "sine",       # "sine", "sawtooth", "square", "triangle", "pulse", "noise"
    sweep_to: float = None,       # Optional: target frequency for sweep
    curve: str = "linear"         # "linear" or "exponential"
) -> dict:
    """
    Create an oscillator synthesis block.

    Returns:
        {
            "type": "oscillator",
            "waveform": waveform,
            "frequency": frequency,
            "freq_sweep": {"end_freq": sweep_to, "curve": curve} if sweep_to else None
        }
    """
    pass

def fm_synth(
    carrier: float,               # Required: carrier frequency in Hz
    modulator: float,             # Required: modulator frequency in Hz
    index: float,                 # Required: modulation index
    sweep_to: float = None        # Optional: target carrier frequency
) -> dict:
    """
    Create an FM synthesis block.

    Returns:
        {
            "type": "fm_synth",
            "carrier_freq": carrier,
            "modulator_freq": modulator,
            "modulation_index": index,
            "freq_sweep": {...} if sweep_to else None
        }
    """
    pass

def noise_burst(
    noise_type: str = "white",    # "white", "pink", "brown"
    filter: dict = None           # Optional: filter from lowpass()/highpass()
) -> dict:
    """
    Create a noise burst synthesis block.

    Returns:
        {"type": "noise_burst", "noise_type": noise_type, "filter": filter}
    """
    pass

def karplus_strong(
    frequency: float,             # Required: base frequency in Hz
    decay: float = 0.99,          # Decay factor (0.0-1.0)
    blend: float = 0.5            # Lowpass blend factor
) -> dict:
    """
    Create Karplus-Strong plucked string synthesis.

    Returns:
        {"type": "karplus_strong", "frequency": frequency, "decay": decay, "blend": blend}
    """
    pass

def lowpass(
    cutoff: float,                # Required: cutoff frequency in Hz
    resonance: float = 0.707,     # Q factor / resonance
    sweep_to: float = None        # Optional: target cutoff for sweep
) -> dict:
    """
    Create a lowpass filter.

    Returns:
        {
            "type": "lowpass",
            "cutoff": cutoff,
            "resonance": resonance,
            "cutoff_end": sweep_to
        }
    """
    pass

def highpass(
    cutoff: float,                # Required: cutoff frequency in Hz
    resonance: float = 0.707,     # Q factor / resonance
    sweep_to: float = None        # Optional: target cutoff for sweep
) -> dict:
    """
    Create a highpass filter.

    Returns:
        {
            "type": "highpass",
            "cutoff": cutoff,
            "resonance": resonance,
            "cutoff_end": sweep_to
        }
    """
    pass

def audio_layer(
    synthesis: dict,              # Required: from oscillator(), fm_synth(), etc.
    envelope: dict = None,        # Optional: from envelope(), uses default if None
    volume: float = 0.8,          # Layer volume (0.0-1.0)
    pan: float = 0.0,             # Stereo pan (-1.0 to 1.0)
    filter: dict = None,          # Optional: from lowpass(), highpass()
    lfo: dict = None,             # Optional: LFO modulation
    delay: float = None           # Optional: layer start delay in seconds
) -> dict:
    """
    Create a complete audio synthesis layer.

    Returns:
        {
            "synthesis": synthesis,
            "envelope": envelope or default_envelope(),
            "volume": volume,
            "pan": pan,
            "filter": filter,
            "lfo": lfo,
            "delay": delay
        }
    """
    pass

def reverb(
    decay: float = 0.5,           # Reverb decay time
    wet: float = 0.3,             # Wet/dry mix (0.0-1.0)
    room_size: float = 0.8        # Room size factor
) -> dict:
    """
    Create a reverb effect.

    Returns:
        {"type": "reverb", "decay": decay, "wet": wet, "room_size": room_size}
    """
    pass

def delay(
    time_ms: float = 250,         # Delay time in milliseconds
    feedback: float = 0.4,        # Feedback amount (0.0-1.0)
    wet: float = 0.3              # Wet/dry mix (0.0-1.0)
) -> dict:
    """
    Create a delay effect.

    Returns:
        {"type": "delay", "time_ms": time_ms, "feedback": feedback, "wet": wet}
    """
    pass

def compressor(
    threshold_db: float = -12,    # Threshold in dB
    ratio: float = 4,             # Compression ratio
    attack_ms: float = 10,        # Attack time in milliseconds
    release_ms: float = 100       # Release time in milliseconds
) -> dict:
    """
    Create a compressor effect.

    Returns:
        {
            "type": "compressor",
            "threshold_db": threshold_db,
            "ratio": ratio,
            "attack_ms": attack_ms,
            "release_ms": release_ms
        }
    """
    pass
```

### Texture Functions

```python
def noise_node(
    id: str,                      # Required: unique node identifier
    algorithm: str = "perlin",    # "perlin", "simplex", "worley", "value", "fbm"
    scale: float = 0.1,           # Noise scale factor
    octaves: int = 4,             # Number of octaves for fractal noise
    persistence: float = 0.5,     # Amplitude decay per octave
    lacunarity: float = 2.0       # Frequency multiplier per octave
) -> dict:
    """
    Create a noise texture node.

    Returns:
        {
            "id": id,
            "type": "noise",
            "noise": {
                "algorithm": algorithm,
                "scale": scale,
                "octaves": octaves,
                "persistence": persistence,
                "lacunarity": lacunarity
            }
        }
    """
    pass

def gradient_node(
    id: str,                      # Required: unique node identifier
    direction: str = "horizontal",# "horizontal", "vertical", "radial"
    start: float = 0.0,           # Start value
    end: float = 1.0              # End value
) -> dict:
    """
    Create a gradient texture node.

    Returns:
        {"id": id, "type": "gradient", "direction": direction, "start": start, "end": end}
    """
    pass

def constant_node(
    id: str,                      # Required: unique node identifier
    value: float                  # Required: constant value (0.0-1.0)
) -> dict:
    """
    Create a constant value node.

    Returns:
        {"id": id, "type": "constant", "value": value}
    """
    pass

def threshold_node(
    id: str,                      # Required: unique node identifier
    input: str,                   # Required: input node id
    threshold: float = 0.5        # Threshold value (0.0-1.0)
) -> dict:
    """
    Create a threshold operation node.

    Returns:
        {"id": id, "type": "threshold", "input": input, "threshold": threshold}
    """
    pass

def invert_node(
    id: str,                      # Required: unique node identifier
    input: str                    # Required: input node id
) -> dict:
    """
    Create an invert operation node (1 - x).

    Returns:
        {"id": id, "type": "invert", "input": input}
    """
    pass

def color_ramp_node(
    id: str,                      # Required: unique node identifier
    input: str,                   # Required: input node id
    ramp: list[str]               # Required: list of hex colors ["#000000", "#ffffff"]
) -> dict:
    """
    Create a color ramp mapping node.

    Returns:
        {"id": id, "type": "color_ramp", "input": input, "ramp": ramp}
    """
    pass

def texture_graph(
    resolution: list[int],        # Required: [width, height] in pixels
    nodes: list[dict],            # Required: list of texture nodes
    tileable: bool = True         # Whether texture should tile seamlessly
) -> dict:
    """
    Create a complete texture graph recipe params.

    Returns:
        {"resolution": resolution, "tileable": tileable, "nodes": nodes}
    """
    pass
```

### Mesh Functions

```python
def mesh_primitive(
    primitive: str,               # Required: "cube", "sphere", "cylinder", etc.
    dimensions: list[float]       # Required: [x, y, z] dimensions
) -> dict:
    """
    Create a base mesh primitive specification.

    Returns:
        {"base_primitive": primitive, "dimensions": dimensions}
    """
    pass

def bevel_modifier(
    width: float = 0.02,          # Bevel width
    segments: int = 2             # Number of bevel segments
) -> dict:
    """
    Create a bevel modifier.

    Returns:
        {"type": "bevel", "width": width, "segments": segments}
    """
    pass

def subdivision_modifier(
    levels: int = 2,              # Subdivision levels
    render_levels: int = None     # Render levels (defaults to levels)
) -> dict:
    """
    Create a subdivision surface modifier.

    Returns:
        {"type": "subdivision", "levels": levels, "render_levels": render_levels or levels}
    """
    pass

def decimate_modifier(
    ratio: float = 0.5            # Decimation ratio (0.0-1.0)
) -> dict:
    """
    Create a decimate modifier.

    Returns:
        {"type": "decimate", "ratio": ratio}
    """
    pass

def mesh_recipe(
    primitive: str,               # Required: primitive type
    dimensions: list[float],      # Required: [x, y, z] dimensions
    modifiers: list[dict] = None  # Optional: list of modifiers
) -> dict:
    """
    Create a complete static mesh recipe params.

    Returns:
        {
            "base_primitive": primitive,
            "dimensions": dimensions,
            "modifiers": modifiers or []
        }
    """
    pass
```

---

## Error Code Table

### Compiler Errors (S001-S009)

| Code | Variant | Description | Example |
|------|---------|-------------|---------|
| S001 | `Syntax` | Starlark syntax error | `S001: expected '}', got ':'` |
| S002 | `Runtime` | Starlark runtime error | `S002: undefined variable 'foo'` |
| S003 | `Timeout` | Evaluation timed out | `S003: evaluation timed out after 30s` |
| S004 | `NotADict` | Result is not a dict | `S004: spec must return dict, got list` |
| S005 | `JsonConversion` | Cannot convert to JSON | `S005: cannot convert function to JSON` |
| S006 | `InvalidSpec` | JSON doesn't match Spec schema | `S006: missing required field 'asset_id'` |

### Stdlib Errors (S101-S199)

| Code | Variant | Description | Example |
|------|---------|-------------|---------|
| S101 | `StdlibArgument` | Invalid argument | `S101: oscillator(): missing required argument 'frequency'` |
| S102 | `StdlibType` | Type mismatch | `S102: oscillator(): 'frequency' expected float, got string` |
| S103 | `StdlibRange` | Value out of range | `S103: oscillator(): 'frequency' must be positive, got -440` |
| S104 | `StdlibEnum` | Invalid enum value | `S104: oscillator(): 'waveform' must be one of: sine, sawtooth, square, triangle` |

---

## Error Message Format

### Human-Readable Format

```
S103: oscillator(): 'frequency' must be positive, got -440
  at spec.star:15:23
```

Components:
1. Error code prefix
2. Function name
3. Parameter name (if applicable)
4. Descriptive message
5. Location (if available)

### Machine-Readable Format (--json)

```json
{
  "success": false,
  "error": {
    "code": "S103",
    "category": "stdlib_range",
    "function": "oscillator",
    "param": "frequency",
    "message": "must be positive",
    "got": "-440",
    "location": {
      "file": "spec.star",
      "line": 15,
      "column": 23
    }
  }
}
```

### Success Response (--json)

```json
{
  "success": true,
  "spec": {
    "spec_version": 1,
    "asset_id": "example-01",
    "..."
  },
  "warnings": []
}
```

---

## CLI Flag Additions

### Existing Flags

```
speccade eval <spec.star|ir.json>      # Prints canonical IR JSON
speccade validate <spec.star|ir.json>  # Validates without generating
speccade generate <spec> --out-root    # Generates assets
```

### New Flags

| Flag | Description | Output |
|------|-------------|--------|
| `--json` | Machine-readable output | JSON error/success objects |
| `--stdlib-version` | Print stdlib version | `0.1.0` |
| `--list-stdlib` | List stdlib functions | Function signatures |

### Usage Examples

```bash
# Compile with JSON output
speccade eval spec.star --json

# Validate with machine-readable errors
speccade validate spec.star --json

# Check stdlib version
speccade --stdlib-version
# Output: 0.1.0

# List available stdlib functions
speccade --list-stdlib
# Output:
# Core:
#   spec(asset_id, asset_type, seed, outputs, ...)
#   output(path, format, kind)
# Audio:
#   envelope(attack, decay, sustain, release)
#   oscillator(frequency, waveform, sweep_to, curve)
#   ...
```

---

## Validation Rules

### String Enums

| Parameter | Valid Values |
|-----------|--------------|
| `waveform` | `sine`, `sawtooth`, `square`, `triangle`, `pulse`, `noise` |
| `noise_type` | `white`, `pink`, `brown` |
| `curve` | `linear`, `exponential` |
| `algorithm` | `perlin`, `simplex`, `worley`, `value`, `fbm` |
| `direction` | `horizontal`, `vertical`, `radial` |
| `primitive` | `cube`, `sphere`, `cylinder`, `cone`, `torus`, `plane`, `ico_sphere` |
| `kind` | `primary`, `secondary` |

### Numeric Ranges

| Parameter | Range | Unit |
|-----------|-------|------|
| `frequency` | > 0 | Hz |
| `volume` | 0.0 - 1.0 | normalized |
| `pan` | -1.0 - 1.0 | L/R |
| `sustain` | 0.0 - 1.0 | normalized |
| `decay` (K-S) | 0.0 - 1.0 | factor |
| `resonance` | > 0 | Q |
| `cutoff` | > 0 | Hz |
| `ratio` | > 0 | compression |
| `seed` | 0 - 2^32-1 | integer |

### Required vs Optional

Functions validate required parameters:
- Error if required parameter missing
- Use documented default if optional parameter missing
- Error if wrong type provided

---

## Stdlib Version

The stdlib version is tracked in `crates/speccade-cli/src/compiler/mod.rs`:

```rust
pub const STDLIB_VERSION: &str = "0.1.0";
```

Version semantics:
- **Patch** (0.1.x): Bug fixes, no output change
- **Minor** (0.x.0): New functions, backward compatible
- **Major** (x.0.0): Breaking changes to existing functions

The version is included in compilation reports for provenance tracking.
