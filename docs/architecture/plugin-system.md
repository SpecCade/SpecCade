# SpecCade Plugin/Extension System Architecture

This document describes how external backends integrate with SpecCade via subprocess or WASM interfaces.

## Overview

SpecCade supports external backends through a plugin system that enables third-party generators to integrate with the pipeline. These extensions communicate through a well-defined I/O contract and must declare their determinism guarantees.

### Design Goals

1. **Isolation**: Extensions run in separate processes/sandboxes for security and stability
2. **Determinism**: Clear contracts for reproducibility at each tier
3. **Discoverability**: Manifest-based registration for tooling support
4. **Simplicity**: Minimal protocol complexity for easy implementation

## Extension Types

### Subprocess Extensions

Subprocess extensions are external executables invoked by SpecCade. They receive input via command-line arguments and files, and produce output to a specified directory.

**Advantages:**
- Language-agnostic (Rust, Python, Node.js, Go, etc.)
- Full access to system resources
- Easy debugging and development
- Mature tooling support

**Disadvantages:**
- Higher overhead (process spawn)
- Harder to sandbox securely
- Platform-specific binaries

### WASM Extensions (Future)

WASM extensions are WebAssembly modules loaded and executed in-process. They provide better sandboxing and portability.

**Advantages:**
- Strong sandboxing
- Cross-platform by default
- Lower overhead
- Memory limits enforced

**Disadvantages:**
- Limited system access
- WASM toolchain required
- Larger module sizes

## Determinism Tiers

Extensions declare their determinism level in the manifest:

| Tier | Level | Guarantee | Verification |
|------|-------|-----------|--------------|
| 1 | `byte_identical` | Same input + seed = same bytes | Hash comparison |
| 2 | `semantic_equivalent` | Same input + seed = equivalent result | Metric validation |
| 3 | `non_deterministic` | No guarantee | None (accept as-is) |

### Tier 1: Byte-Identical

- Output must be byte-for-byte identical for the same input spec and seed
- Required for caching and incremental builds
- Extension must provide `output_hash` in determinism report
- SpecCade verifies hashes match on repeated runs

### Tier 2: Semantic Equivalent

- Output may vary in binary representation but must be semantically equivalent
- Used when external tools (Blender, ImageMagick) produce different bytes
- Extension provides metrics for validation
- No hash verification, but metrics must be consistent

### Tier 3: Non-Deterministic

- No reproducibility guarantee
- Useful for AI/ML-based generators or time-dependent tools
- Outputs accepted but not cached
- Extension must explain non-determinism reason

## Subprocess Protocol

### Invocation

SpecCade invokes subprocess extensions with:

```
<executable> --spec <spec_path> --out <out_dir> --seed <u64> [custom_args...]
```

Where:
- `<spec_path>`: Path to JSON spec file (temporary file in out_dir)
- `<out_dir>`: Directory where extension writes outputs
- `<u64>`: The seed from the spec (convenience argument)
- `[custom_args]`: Additional arguments from manifest (with placeholders substituted)

### Placeholder Substitution

Custom arguments support these placeholders:
- `{spec_path}` - Path to the input spec file
- `{out_dir}` - Output directory path
- `{seed}` - Seed value as string

### Input

The spec is written to `{out_dir}/input.spec.json` before invocation. Extensions read this file to get generation parameters.

### Output

Extensions must:
1. Write generated files to `{out_dir}/`
2. Write `{out_dir}/manifest.json` describing the results

### Exit Codes

- `0`: Success (manifest.json must have `success: true`)
- Non-zero: Failure (stderr should contain error details)

### Timeout Handling

Extensions have a configurable timeout (default: 300 seconds). If exceeded:
1. SIGKILL (Unix) or TerminateProcess (Windows) is sent
2. Generation is reported as failed with `Timeout` error

## Output Manifest

Extensions must write `manifest.json` to the output directory:

```json
{
  "manifest_version": 1,
  "success": true,
  "output_files": [
    {
      "path": "output.png",
      "hash": "<64-char-blake3-hex>",
      "size": 1024,
      "kind": "primary",
      "format": "png"
    }
  ],
  "determinism_report": {
    "input_hash": "<64-char-blake3-hex>",
    "output_hash": "<64-char-blake3-hex>",
    "tier": 1,
    "determinism": "byte_identical",
    "seed": 42,
    "deterministic": true
  },
  "errors": [],
  "warnings": [],
  "duration_ms": 150,
  "extension_version": "1.0.0"
}
```

### Output File Entry

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | Yes | Relative path within output directory |
| `hash` | string | Yes | BLAKE3 hash (64 hex characters) |
| `size` | number | Yes | File size in bytes |
| `kind` | string | No | "primary", "metadata", or "preview" (default: "primary") |
| `format` | string | No | File format ("png", "wav", "json", etc.) |

### Determinism Report

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `input_hash` | string | Yes | BLAKE3 hash of input spec |
| `output_hash` | string | Tier 1 only | Combined hash of all outputs |
| `tier` | number | Yes | Determinism tier (1, 2, or 3) |
| `determinism` | string | Yes | "byte_identical", "semantic_equivalent", or "non_deterministic" |
| `seed` | number | Yes | Seed used for generation |
| `deterministic` | boolean | Yes | Whether this run was deterministic |
| `non_determinism_reason` | string | Tier 3 only | Explanation for non-determinism |

## Extension Manifest

Extensions are registered via manifest files:

```json
{
  "name": "custom-texture-gen",
  "version": "1.0.0",
  "tier": 1,
  "determinism": "byte_identical",
  "interface": {
    "type": "subprocess",
    "executable": "custom-texture-gen",
    "args": ["--verbose"],
    "env": {},
    "timeout_seconds": 300
  },
  "recipe_kinds": ["texture.custom_v1"],
  "description": "Custom procedural texture generator",
  "author": "Example Author",
  "license": "MIT",
  "input_schema": null,
  "output_schema": null
}
```

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique extension name (lowercase, hyphens allowed) |
| `version` | string | Yes | Semantic version (X.Y.Z) |
| `tier` | number | Yes | Determinism tier (1, 2, or 3) |
| `determinism` | string | Yes | Determinism level |
| `interface` | object | Yes | Interface configuration |
| `recipe_kinds` | array | Yes | Recipe kinds this extension handles |
| `description` | string | No | Human-readable description |
| `author` | string | No | Author name |
| `license` | string | No | License identifier |
| `input_schema` | object | No | JSON Schema for input validation |
| `output_schema` | object | No | JSON Schema for output validation |

## Extension Discovery

Extensions are discovered from these locations (in order):

1. `./extensions/` - Current working directory
2. `~/.config/speccade/extensions/` - User config directory
3. `/usr/share/speccade/extensions/` - System-wide (Unix)
4. `/usr/local/share/speccade/extensions/` - Local system-wide (Unix)

Each directory should contain subdirectories with `manifest.json` files:

```
extensions/
  custom-texture-gen/
    manifest.json
    custom-texture-gen.exe (or binary)
  another-extension/
    manifest.json
    ...
```

## Verification Flow

```
1. Load spec
2. Determine recipe kind
3. Check registry for matching extension
4. If extension found:
   a. Compute input hash
   b. Spawn subprocess with arguments
   c. Wait for completion (with timeout)
   d. Parse output manifest
   e. Verify manifest structure
   f. For Tier 1: verify output hashes
   g. For Tier 2: validate metrics
   h. Return results
5. If no extension: fall back to built-in backends
```

## Error Handling

### Extension Errors

| Error | Description | Recovery |
|-------|-------------|----------|
| `SpawnFailed` | Could not start subprocess | Check executable path |
| `Timeout` | Extension exceeded time limit | Increase timeout or optimize |
| `NonZeroExit` | Process exited with error | Check stderr for details |
| `ManifestMissing` | No manifest.json produced | Extension bug |
| `ManifestInvalid` | Manifest failed to parse | Extension bug |
| `OutputFileMissing` | Declared file not found | Extension bug |
| `OutputHashMismatch` | Hash verification failed | Determinism bug |
| `TierMismatch` | Declared tier doesn't match | Manifest misconfiguration |
| `InputHashMismatch` | Spec was modified during run | Spec file corruption |

### Error Recovery

Extensions should:
1. Write meaningful error messages to stderr
2. Include error details in manifest.json errors array
3. Exit with non-zero code on failure
4. Clean up partial outputs on failure

## Security Considerations

### Subprocess Extensions

- Extensions run with the same privileges as SpecCade
- No sandboxing by default
- Users should only install trusted extensions
- Consider running in containers for untrusted extensions

### WASM Extensions (Future)

- Memory limits enforced
- No direct file system access
- Capabilities granted via WASI
- Time limits enforced

## Implementation Checklist

For extension developers:

- [ ] Create manifest.json with required fields
- [ ] Implement argument parsing (--spec, --out, --seed)
- [ ] Read spec from provided path
- [ ] Generate outputs to output directory
- [ ] Compute BLAKE3 hashes for all outputs
- [ ] Write manifest.json with determinism report
- [ ] Handle errors gracefully
- [ ] Test determinism by running multiple times with same seed
- [ ] Document recipe kinds and parameters

## Examples

See `examples/extensions/simple-subprocess/` for a reference implementation.
