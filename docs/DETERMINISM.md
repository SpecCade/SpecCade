# SpecCade Determinism Policy

This document defines the complete determinism policy for SpecCade asset generation. Determinism ensures that identical specs with identical seeds produce identical (or measurably equivalent) outputs, enabling reproducible builds, caching, and verification.

---

## Table of Contents

1. [Overview](#overview)
2. [RNG Algorithm](#rng-algorithm)
3. [Seed Derivation](#seed-derivation)
4. [Spec Canonicalization](#spec-canonicalization)
5. [Spec Hashing](#spec-hashing)
6. [Artifact Comparison Rules](#artifact-comparison-rules)
7. [Tier 1 vs Tier 2 Guarantees](#tier-1-vs-tier-2-guarantees)
8. [Pseudocode and Rust Snippets](#pseudocode-and-rust-snippets)
9. [Cross-Platform Considerations](#cross-platform-considerations)
10. [References](#references)

---

## Overview

SpecCade guarantees deterministic output within documented tolerances. The level of determinism depends on the backend tier:

| Tier | Backends | Guarantee |
|------|----------|-----------|
| **Tier 1** | Rust (audio, music, texture) | Byte-identical output per `(target_triple, backend_version)` |
| **Tier 2** | Blender (mesh, animation) | Metric validation only (not byte-identical) |

**Core Principles:**

- Same spec + same seed = identical output (within tier constraints)
- RNG algorithm is standardized across all backends
- Spec hashing enables caching and verification
- Artifact comparison is format-aware

---

## RNG Algorithm

### PCG32 Specification

All SpecCade Rust backends **MUST** use **PCG32** (Permuted Congruential Generator, 32-bit output) for random number generation.

**Why PCG32?**

| Property | Benefit |
|----------|---------|
| Deterministic | Same seed produces identical sequence |
| Fast | Minimal performance overhead |
| High quality | Passes statistical randomness tests |
| Portable | Identical output across platforms |
| Small state | Only 128 bits of state (64-bit state + 64-bit increment) |

**Rust Implementation:**

Use the `rand_pcg` crate with `Pcg32` (also known as `Lcg64Xsh32`):

```toml
# Cargo.toml
[dependencies]
rand = "0.8"
rand_pcg = "0.3"
```

```rust
use rand::SeedableRng;
use rand_pcg::Pcg32;

// Initialize from a 64-bit seed (expanded from 32-bit spec seed)
fn create_rng(seed: u32) -> Pcg32 {
    // Expand 32-bit seed to 64-bit for PCG32 state
    let seed64 = (seed as u64) | ((seed as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}
```

**State Initialization:**

The PCG32 generator requires a 64-bit seed for its internal state. Since spec seeds are 32-bit (`0` to `2^32-1`), we expand by duplicating the bits:

```
seed64 = (seed32 as u64) | ((seed32 as u64) << 32)
```

This ensures the full 64-bit state space is utilized while maintaining backwards compatibility with 32-bit seeds.

### Algorithm Stability

The PCG32 algorithm and its output sequence **MUST NOT** change between SpecCade versions. Any change to the RNG algorithm constitutes a breaking change requiring a `spec_version` bump.

---

## Seed Derivation

SpecCade uses hierarchical seed derivation to ensure independent random streams for different components within a spec.

### Base Seed

The base seed comes directly from the spec:

```json
{
  "seed": 42
}
```

Constraints:
- Range: `0` to `2^32-1` (4294967295)
- Type: unsigned 32-bit integer

### Per-Layer Seed

When generating assets with multiple layers (audio layers, texture layers, etc.), each layer receives a derived seed:

```
layer_seed = truncate_u32(BLAKE3(base_seed || layer_index))
```

Where:
- `||` denotes byte concatenation
- `base_seed` is encoded as 4 little-endian bytes
- `layer_index` is encoded as 4 little-endian bytes (0-indexed)
- `truncate_u32` takes the first 4 bytes of the BLAKE3 hash and interprets them as a little-endian u32

**Example:**

```
base_seed = 42
layer_index = 0

input = [42, 0, 0, 0] || [0, 0, 0, 0]  # 8 bytes total
hash = BLAKE3(input)                    # 32 bytes
layer_seed = u32::from_le_bytes(hash[0..4])
```

### Per-Variant Seed

When generating variants from a single spec, each variant receives a derived seed:

```
variant_seed = truncate_u32(BLAKE3(base_seed || seed_offset || variant_id))
```

Where:
- `base_seed` is encoded as 4 little-endian bytes
- `seed_offset` is encoded as 4 little-endian bytes
- `variant_id` is the UTF-8 encoded variant identifier string

**Example:**

```
base_seed = 42
seed_offset = 0
variant_id = "soft"

input = [42, 0, 0, 0] || [0, 0, 0, 0] || b"soft"  # 12 bytes total
hash = BLAKE3(input)              # 32 bytes
variant_seed = u32::from_le_bytes(hash[0..4])
```

### Why Hash-Based Derivation?

The hash-based approach is preferred over simple offset addition (`seed + offset`) because:

1. **Collision avoidance**: Offset addition can cause collisions when seeds wrap around `2^32`
2. **Independence**: BLAKE3 ensures derived seeds are statistically independent
3. **Security**: No predictable relationship between base and derived seeds
4. **Extensibility**: Works for any string-based identifier (variant_id, layer names, etc.)

---

## Spec Canonicalization

### RFC 8785 (JCS) Summary

SpecCade uses **JSON Canonicalization Scheme (JCS)** as defined in [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) for spec canonicalization.

JCS defines a deterministic serialization of JSON that produces identical byte sequences regardless of the original formatting or key ordering.

**JCS Rules:**

1. **Object key ordering**: Keys are sorted by their UTF-16 code unit values (lexicographic)
2. **No whitespace**: No spaces or newlines between tokens
3. **Number formatting**: IEEE 754 double precision with specific formatting rules
4. **String escaping**: Minimal escaping (only required characters)
5. **No trailing data**: Single JSON value only

### Why Canonicalization Matters

Without canonicalization, the same logical spec can produce different byte sequences:

```json
// These are logically identical but have different bytes:
{"seed": 42, "asset_id": "test"}
{"asset_id": "test", "seed": 42}
{  "seed":42,  "asset_id":"test"  }
```

Canonicalization ensures:
- **Consistent hashing**: Same spec always produces same hash
- **Cache correctness**: Identical specs hit the same cache entry
- **Verification**: Specs can be compared by hash alone
- **Version control**: Meaningful diffs in spec files

### Implementation

**Rust (recommended):**

```toml
# Cargo.toml
[dependencies]
serde_json_canonicalizer = "0.2"
```

```rust
use serde_json_canonicalizer::to_string as canonicalize;

fn canonical_json(spec: &serde_json::Value) -> Result<String, Error> {
    canonicalize(spec)
}
```

**Manual implementation (if needed):**

```rust
use serde_json::Value;
use std::collections::BTreeMap;

fn canonicalize_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => format_jcs_number(n),
        Value::String(s) => format_jcs_string(s),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .map(canonicalize_value)
                .collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            // Sort keys lexicographically by UTF-16 code units
            let mut sorted: BTreeMap<&String, &Value> = BTreeMap::new();
            for (k, v) in obj {
                sorted.insert(k, v);
            }
            let pairs: Vec<String> = sorted.iter()
                .map(|(k, v)| format!("{}:{}", format_jcs_string(k), canonicalize_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}
```

---

## Spec Hashing

### BLAKE3 Rationale

SpecCade uses **BLAKE3** for all hashing operations.

**Why BLAKE3?**

| Property | Value |
|----------|-------|
| Speed | ~4x faster than SHA-256 on modern CPUs |
| Determinism | Identical output across all platforms |
| Security | 256-bit security level |
| Parallelism | Can utilize SIMD and multiple cores |
| Streaming | Supports incremental hashing |
| Simplicity | Single algorithm for all hash sizes |

### Spec Hash Computation

The spec hash uniquely identifies a canonical spec:

```
spec_hash = hex(BLAKE3(JCS(spec_json)))
```

**Steps:**

1. Parse the spec JSON
2. Canonicalize using JCS (RFC 8785)
3. Compute BLAKE3 hash of the canonical UTF-8 bytes
4. Encode as lowercase hexadecimal string

**Output format:**
- 64 hexadecimal characters (256 bits / 4 bits per char)
- Lowercase letters (`a-f`, not `A-F`)

**Example:**

```json
{"asset_id":"laser-blast-01","asset_type":"audio","seed":42,"spec_version":1}
```

Produces hash: `a1b2c3d4...` (64 hex chars)

### Rust Implementation

```toml
# Cargo.toml
[dependencies]
blake3 = "1.5"
```

```rust
use blake3::Hasher;

fn spec_hash(canonical_json: &str) -> String {
    let hash = blake3::hash(canonical_json.as_bytes());
    hash.to_hex().to_string()
}
```

---

## Artifact Comparison Rules

Different output formats require different comparison strategies due to format-specific metadata and encoding variations.

### WAV Files

**Comparison method:** Hash PCM sample data only (ignore RIFF header)

**Rationale:** WAV headers contain metadata fields (timestamps, software version) that may vary between runs without affecting the actual audio content.

**Implementation:**

1. Parse the WAV file structure
2. Locate the `data` chunk
3. Extract raw PCM sample bytes
4. Compute BLAKE3 hash of sample bytes only

**Fields to ignore:**
- RIFF chunk size (varies with metadata)
- `INFO` chunk and subchunks
- `LIST` chunk and subchunks
- Any non-audio chunks

**Fields to include:**
- `fmt ` chunk (format specification)
- `data` chunk (audio samples)

### XM/IT Files

**Comparison method:** Hash full file bytes

**Rationale:** XM and IT tracker formats have deterministic structure with no variable metadata. The entire file content is significant.

**Implementation:**

```rust
fn hash_tracker_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    blake3::hash(&bytes).to_hex().to_string()
}
```

### PNG Files

**Comparison method:** Hash full file bytes

**Rationale:** With deterministic encoder settings, PNG files should be byte-identical.

**Required encoder settings for determinism:**

| Setting | Value | Notes |
|---------|-------|-------|
| Compression level | Fixed (e.g., 6) | Same level across runs |
| Filter strategy | Fixed (e.g., adaptive) | Same strategy across runs |
| Interlacing | Off | Interlacing affects byte order |
| Metadata chunks | None | No `tEXt`, `iTXt`, `zTXt` chunks |
| Timestamp | Omit | No `tIME` chunk |

**Rust implementation (using `image` crate):**

```rust
use image::codecs::png::{CompressionType, FilterType, PngEncoder};

fn create_deterministic_png_encoder<W: std::io::Write>(writer: W) -> PngEncoder<W> {
    PngEncoder::new_with_quality(writer, CompressionType::Default, FilterType::Adaptive)
}
```

### GLB Files

**Comparison method:** Metric validation only (not byte-identical)

**Rationale:** GLB files are produced by Blender, which may have non-deterministic internal operations (floating-point order, buffer packing, etc.).

**Validated metrics:**

| Metric | Tolerance | Notes |
|--------|-----------|-------|
| Triangle count | Exact (0%) | Must match exactly |
| Bounding box (min/max XYZ) | +/- 0.001 units | Per-axis comparison |
| UV island count | Exact | Must match exactly |
| Bone count | Exact | Must match exactly |
| Material slot count | Exact | Must match exactly |
| Animation frame count | Exact | Must match exactly |
| Animation duration | +/- 0.001 seconds | Floating-point tolerance |

---

## Tier 1 vs Tier 2 Guarantees

### Tier 1: Byte-Identical Output

**Applies to:** Rust backends (audio, music, texture)

**Guarantee:** The same spec produces byte-identical output when:

1. **Target triple matches** (e.g., `x86_64-pc-windows-msvc`)
2. **Backend version matches** (e.g., `speccade-backend-audio v0.1.0`)
3. **Spec is identical** (same `spec_hash`)

**Verification:**

```rust
fn verify_tier1_output(expected_hash: &str, actual_path: &Path) -> bool {
    let actual_hash = compute_artifact_hash(actual_path);
    expected_hash == actual_hash
}
```

**Caching:**

Tier 1 outputs can be cached by `(spec_hash, target_triple, backend_version)` and reused without regeneration.

### Tier 2: Metric Validation

**Applies to:** Blender backends (mesh, animation)

**Guarantee:** The same spec produces output that passes metric validation within defined tolerances.

**Verification:**

```rust
fn verify_tier2_output(expected_metrics: &Metrics, actual_path: &Path) -> ValidationResult {
    let actual_metrics = extract_glb_metrics(actual_path);
    compare_metrics(expected_metrics, actual_metrics)
}
```

**Metric comparison rules:**

```rust
fn compare_metrics(expected: &Metrics, actual: &Metrics) -> ValidationResult {
    let mut errors = Vec::new();

    // Exact matches
    if expected.triangle_count != actual.triangle_count {
        errors.push(format!(
            "Triangle count mismatch: expected {}, got {}",
            expected.triangle_count, actual.triangle_count
        ));
    }

    // Tolerance matches
    let bbox_tolerance = 0.001;
    if !within_tolerance(expected.bounding_box, actual.bounding_box, bbox_tolerance) {
        errors.push("Bounding box outside tolerance".to_string());
    }

    ValidationResult { ok: errors.is_empty(), errors }
}
```

### Cross-Platform Caveats

**Cross-platform determinism is NOT guaranteed** for Tier 1 outputs unless explicitly documented.

**Reasons:**

1. **Floating-point differences**: IEEE 754 operations may produce slightly different results across CPU architectures (x86 vs ARM, different FPU implementations)
2. **SIMD variations**: Vectorized operations may have architecture-specific behavior
3. **Library differences**: System libraries (libc, etc.) may vary

**Recommendations:**

- Cache artifacts per `(spec_hash, target_triple, backend_version)`
- Document known cross-platform differences
- Use fixed-point arithmetic where exact reproducibility is critical

---

## Pseudocode and Rust Snippets

### canonical_spec_hash

Computes the canonical hash of a spec JSON document.

```rust
use blake3::Hasher;
use serde_json::Value;
use serde_json_canonicalizer::to_string as canonicalize;

/// Computes the canonical BLAKE3 hash of a spec.
///
/// # Arguments
/// * `spec` - The spec as a serde_json::Value
///
/// # Returns
/// * A 64-character lowercase hexadecimal string
///
/// # Example
/// ```
/// let spec: Value = serde_json::from_str(r#"{"seed": 42, "asset_id": "test"}"#)?;
/// let hash = canonical_spec_hash(&spec)?;
/// assert_eq!(hash.len(), 64);
/// ```
pub fn canonical_spec_hash(spec: &Value) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Canonicalize the JSON using JCS (RFC 8785)
    let canonical = canonicalize(spec)?;

    // Step 2: Compute BLAKE3 hash
    let hash = blake3::hash(canonical.as_bytes());

    // Step 3: Return as lowercase hex string
    Ok(hash.to_hex().to_string())
}
```

### derive_layer_seed

Derives a deterministic seed for a specific layer.

```rust
/// Derives a seed for a specific layer from the base seed.
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `layer_index` - The 0-indexed layer number
///
/// # Returns
/// * A derived u32 seed for the layer
///
/// # Example
/// ```
/// let base = 42u32;
/// let layer0_seed = derive_layer_seed(base, 0);
/// let layer1_seed = derive_layer_seed(base, 1);
/// assert_ne!(layer0_seed, layer1_seed);
/// ```
pub fn derive_layer_seed(base_seed: u32, layer_index: u32) -> u32 {
    // Concatenate base_seed and layer_index as little-endian bytes
    let mut input = Vec::with_capacity(8);
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&layer_index.to_le_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}
```

### derive_variant_seed

Derives a deterministic seed for an arbitrary string identifier (used for named sub-seeds like map keys or logical variant labels).

```rust
/// Derives a seed for a specific variant from the base seed.
///
/// # Arguments
/// * `base_seed` - The spec's base seed (u32)
/// * `variant_id` - The variant identifier string
///
/// # Returns
/// * A derived u32 seed for the variant
///
/// # Example
/// ```
/// let base = 42u32;
/// let soft_seed = derive_variant_seed(base, "soft");
/// let hard_seed = derive_variant_seed(base, "hard");
/// assert_ne!(soft_seed, hard_seed);
/// ```
pub fn derive_variant_seed(base_seed: u32, variant_id: &str) -> u32 {
    // Concatenate base_seed (as little-endian bytes) and variant_id (as UTF-8)
    let mut input = Vec::with_capacity(4 + variant_id.len());
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(variant_id.as_bytes());

    // Hash with BLAKE3
    let hash = blake3::hash(&input);

    // Truncate to u32 (first 4 bytes, little-endian)
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}
```

### derive_variant_spec_seed

Derives a deterministic seed for expanding `Spec.variants[]` using both `seed_offset` and `variant_id`.

```rust
/// Derives a seed for a specific *spec variant* from the base seed, variant id, and seed offset.
///
/// Intended for expanding `Spec.variants[]` in the CLI and tooling.
pub fn derive_variant_spec_seed(base_seed: u32, seed_offset: u32, variant_id: &str) -> u32 {
    let mut input = Vec::with_capacity(8 + variant_id.len());
    input.extend_from_slice(&base_seed.to_le_bytes());
    input.extend_from_slice(&seed_offset.to_le_bytes());
    input.extend_from_slice(variant_id.as_bytes());

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    u32::from_le_bytes(bytes)
}
```

### compare_wav_pcm

Compares two WAV files by their PCM sample data.

```rust
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

/// Compares two WAV files by their PCM sample data only.
/// Ignores header metadata that may vary between runs.
///
/// # Arguments
/// * `expected` - Path to the expected WAV file
/// * `actual` - Path to the actual WAV file
///
/// # Returns
/// * `true` if PCM data matches, `false` otherwise
///
/// # Example
/// ```
/// let matches = compare_wav_pcm(
///     Path::new("expected/laser.wav"),
///     Path::new("output/laser.wav")
/// )?;
/// assert!(matches, "WAV PCM data mismatch");
/// ```
pub fn compare_wav_pcm(expected: &Path, actual: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let expected_pcm = extract_wav_pcm_data(expected)?;
    let actual_pcm = extract_wav_pcm_data(actual)?;

    // Compare lengths first
    if expected_pcm.len() != actual_pcm.len() {
        return Ok(false);
    }

    // Compare bytes
    Ok(expected_pcm == actual_pcm)
}

/// Extracts raw PCM data from a WAV file.
fn extract_wav_pcm_data(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Find the "data" chunk
    // WAV structure: RIFF header (12 bytes) + chunks
    // Each chunk: 4-byte ID + 4-byte size + data

    let mut pos = 12; // Skip RIFF header
    while pos + 8 <= buffer.len() {
        let chunk_id = &buffer[pos..pos+4];
        let chunk_size = u32::from_le_bytes(buffer[pos+4..pos+8].try_into()?) as usize;

        if chunk_id == b"data" {
            // Found the data chunk
            let data_start = pos + 8;
            let data_end = data_start + chunk_size;
            if data_end <= buffer.len() {
                return Ok(buffer[data_start..data_end].to_vec());
            }
        }

        pos += 8 + chunk_size;
        // Align to even boundary (WAV chunks are word-aligned)
        if chunk_size % 2 != 0 {
            pos += 1;
        }
    }

    Err("No data chunk found in WAV file".into())
}

/// Alternative: Compare by hash for efficiency
pub fn compare_wav_pcm_hash(expected: &Path, actual: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let expected_hash = hash_wav_pcm(expected)?;
    let actual_hash = hash_wav_pcm(actual)?;
    Ok(expected_hash == actual_hash)
}

fn hash_wav_pcm(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pcm_data = extract_wav_pcm_data(path)?;
    Ok(blake3::hash(&pcm_data).to_hex().to_string())
}
```

### validate_glb_metrics

Validates a GLB file against expected metrics.

```rust
use std::path::Path;

/// Metrics extracted from a GLB file for Tier 2 validation.
#[derive(Debug, Clone)]
pub struct GlbMetrics {
    pub triangle_count: u32,
    pub bounding_box: BoundingBox,
    pub uv_island_count: u32,
    pub bone_count: u32,
    pub material_slot_count: u32,
    pub animation_frame_count: Option<u32>,
    pub animation_duration: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// Tolerances for metric validation.
#[derive(Debug, Clone)]
pub struct MetricTolerances {
    pub bounding_box: f64,      // Default: 0.001
    pub animation_duration: f64, // Default: 0.001
}

impl Default for MetricTolerances {
    fn default() -> Self {
        Self {
            bounding_box: 0.001,
            animation_duration: 0.001,
        }
    }
}

/// Result of GLB metric validation.
#[derive(Debug)]
pub struct ValidationResult {
    pub ok: bool,
    pub errors: Vec<String>,
}

/// Validates a GLB file against expected metrics.
///
/// # Arguments
/// * `expected` - Expected metrics from spec or baseline
/// * `actual` - Actual metrics extracted from the GLB file
/// * `tolerances` - Tolerance values for floating-point comparisons
///
/// # Returns
/// * `ValidationResult` with ok=true if all metrics pass
///
/// # Example
/// ```
/// let expected = GlbMetrics { triangle_count: 500, ... };
/// let actual = extract_glb_metrics(Path::new("output/crate.glb"))?;
/// let result = validate_glb_metrics(&expected, &actual, &MetricTolerances::default());
/// assert!(result.ok, "Validation failed: {:?}", result.errors);
/// ```
pub fn validate_glb_metrics(
    expected: &GlbMetrics,
    actual: &GlbMetrics,
    tolerances: &MetricTolerances,
) -> ValidationResult {
    let mut errors = Vec::new();

    // Exact match: triangle count
    if expected.triangle_count != actual.triangle_count {
        errors.push(format!(
            "Triangle count mismatch: expected {}, got {}",
            expected.triangle_count, actual.triangle_count
        ));
    }

    // Exact match: UV island count
    if expected.uv_island_count != actual.uv_island_count {
        errors.push(format!(
            "UV island count mismatch: expected {}, got {}",
            expected.uv_island_count, actual.uv_island_count
        ));
    }

    // Exact match: bone count
    if expected.bone_count != actual.bone_count {
        errors.push(format!(
            "Bone count mismatch: expected {}, got {}",
            expected.bone_count, actual.bone_count
        ));
    }

    // Exact match: material slot count
    if expected.material_slot_count != actual.material_slot_count {
        errors.push(format!(
            "Material slot count mismatch: expected {}, got {}",
            expected.material_slot_count, actual.material_slot_count
        ));
    }

    // Tolerance match: bounding box
    for i in 0..3 {
        let axis = ['X', 'Y', 'Z'][i];

        if (expected.bounding_box.min[i] - actual.bounding_box.min[i]).abs() > tolerances.bounding_box {
            errors.push(format!(
                "Bounding box min {} outside tolerance: expected {:.6}, got {:.6}",
                axis, expected.bounding_box.min[i], actual.bounding_box.min[i]
            ));
        }

        if (expected.bounding_box.max[i] - actual.bounding_box.max[i]).abs() > tolerances.bounding_box {
            errors.push(format!(
                "Bounding box max {} outside tolerance: expected {:.6}, got {:.6}",
                axis, expected.bounding_box.max[i], actual.bounding_box.max[i]
            ));
        }
    }

    // Animation metrics (if present)
    if let (Some(expected_frames), Some(actual_frames)) =
        (expected.animation_frame_count, actual.animation_frame_count)
    {
        if expected_frames != actual_frames {
            errors.push(format!(
                "Animation frame count mismatch: expected {}, got {}",
                expected_frames, actual_frames
            ));
        }
    }

    if let (Some(expected_dur), Some(actual_dur)) =
        (expected.animation_duration, actual.animation_duration)
    {
        if (expected_dur - actual_dur).abs() > tolerances.animation_duration {
            errors.push(format!(
                "Animation duration outside tolerance: expected {:.6}s, got {:.6}s",
                expected_dur, actual_dur
            ));
        }
    }

    ValidationResult {
        ok: errors.is_empty(),
        errors,
    }
}

/// Extracts metrics from a GLB file.
///
/// Note: This is a placeholder signature. Actual implementation requires
/// a glTF parsing library like `gltf` crate.
pub fn extract_glb_metrics(path: &Path) -> Result<GlbMetrics, Box<dyn std::error::Error>> {
    // Implementation would use the `gltf` crate to parse the GLB
    // and extract the relevant metrics.
    todo!("Implement GLB metric extraction")
}
```

### Complete Example: RNG Initialization

```rust
use rand::SeedableRng;
use rand::Rng;
use rand_pcg::Pcg32;

/// Creates a seeded RNG for a specific layer.
pub fn create_layer_rng(base_seed: u32, layer_index: u32) -> Pcg32 {
    let layer_seed = derive_layer_seed(base_seed, layer_index);

    // Expand 32-bit seed to 64-bit for PCG32
    let seed64 = (layer_seed as u64) | ((layer_seed as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Example usage in audio generation
pub fn generate_noise_layer(base_seed: u32, layer_index: u32, num_samples: usize) -> Vec<f32> {
    let mut rng = create_layer_rng(base_seed, layer_index);

    (0..num_samples)
        .map(|_| rng.gen::<f32>() * 2.0 - 1.0) // Range [-1, 1]
        .collect()
}
```

---

## Cross-Platform Considerations

### Known Differences

| Component | Issue | Mitigation |
|-----------|-------|------------|
| Floating-point | x87 FPU uses 80-bit intermediate values | Use SSE2 (64-bit) or explicit rounding |
| Trigonometry | `sin`, `cos` may vary across libm versions | Use deterministic implementations (e.g., `libm` crate) |
| SIMD | AVX vs SSE vs NEON produce different rounding | Document SIMD requirements per target |
| Thread ordering | Parallel operations may reorder | Use deterministic reduction patterns |

### Recommendations

1. **Pin dependencies**: Use exact versions for math libraries
2. **Test across platforms**: CI should verify determinism on all supported targets
3. **Document exceptions**: If a backend cannot guarantee cross-platform determinism, document it clearly
4. **Provide reference hashes**: Include expected hashes per `(spec, target_triple)` in test fixtures

---

## References

- [RFC 8785 - JSON Canonicalization Scheme (JCS)](https://www.rfc-editor.org/rfc/rfc8785)
- [BLAKE3 Hash Function](https://github.com/BLAKE3-team/BLAKE3)
- [PCG Random Number Generator](https://www.pcg-random.org/)
- [rand_pcg Rust crate](https://docs.rs/rand_pcg/)
- [serde_json_canonicalizer Rust crate](https://docs.rs/serde_json_canonicalizer/)
- [RFC-0001: Canonical Spec Architecture](./rfcs/RFC-0001-canonical-spec.md)
- [PARITY_MATRIX.md](../PARITY_MATRIX.md) - Generator tiers (Tier 1 vs Tier 2)

---

*Document version: 1.0*
*Last updated: 2026-01-10*
*SpecCade Phase 1 Task 1.3*
