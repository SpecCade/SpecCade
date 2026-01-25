# Extension I/O Contract Specification

This document defines the I/O contract between SpecCade and external backend extensions.

## Overview

External extensions communicate with SpecCade through a file-based protocol. SpecCade writes input to files, spawns the extension process, and reads output from files produced by the extension.

## Input Contract

### Command Line Arguments

Extensions receive these standard arguments:

```
--spec <path>    Path to the input spec JSON file
--out <path>     Directory for output files
--seed <u64>     Seed value from the spec
```

Example invocation:
```bash
my-extension --spec /tmp/speccade-12345/input.spec.json --out /tmp/speccade-12345 --seed 42
```

### Input Spec File

The spec file (`input.spec.json`) contains a standard SpecCade spec:

```json
{
  "spec_version": 1,
  "asset_id": "custom-texture-01",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "textures/custom.png"
    }
  ],
  "recipe": {
    "kind": "texture.custom_v1",
    "params": {
      "width": 256,
      "height": 256,
      "style": "metal"
    }
  }
}
```

Extensions should:
1. Parse the JSON spec
2. Extract `recipe.params` for generation parameters
3. Use `seed` for deterministic random number generation
4. Generate files matching `outputs` specifications

## Output Contract

### Required Files

Extensions must produce:

1. **Generated assets** - Files matching the spec's `outputs` declarations
2. **`manifest.json`** - Generation result manifest

### Output Manifest Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ExtensionOutputManifest",
  "type": "object",
  "required": ["manifest_version", "success", "determinism_report"],
  "properties": {
    "manifest_version": {
      "type": "integer",
      "const": 1,
      "description": "Manifest schema version"
    },
    "success": {
      "type": "boolean",
      "description": "Whether generation succeeded"
    },
    "output_files": {
      "type": "array",
      "items": { "$ref": "#/$defs/OutputFile" },
      "description": "List of generated files"
    },
    "determinism_report": {
      "$ref": "#/$defs/DeterminismReport"
    },
    "errors": {
      "type": "array",
      "items": { "$ref": "#/$defs/ErrorEntry" },
      "description": "List of errors"
    },
    "warnings": {
      "type": "array",
      "items": { "type": "string" },
      "description": "List of warnings"
    },
    "duration_ms": {
      "type": "integer",
      "minimum": 0,
      "description": "Generation duration in milliseconds"
    },
    "extension_version": {
      "type": "string",
      "description": "Version of the extension"
    },
    "metadata": {
      "type": "object",
      "description": "Extension-specific metadata"
    }
  },
  "$defs": {
    "OutputFile": {
      "type": "object",
      "required": ["path", "hash", "size"],
      "properties": {
        "path": {
          "type": "string",
          "pattern": "^[^./][^.]*$|^[^/].*[^/]$",
          "description": "Relative path (no .. or leading /)"
        },
        "hash": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$",
          "description": "BLAKE3 hash (64 hex chars)"
        },
        "size": {
          "type": "integer",
          "minimum": 0,
          "description": "File size in bytes"
        },
        "kind": {
          "type": "string",
          "enum": ["primary", "metadata", "preview"],
          "default": "primary"
        },
        "format": {
          "type": "string",
          "description": "File format (png, wav, json, etc.)"
        }
      }
    },
    "DeterminismReport": {
      "type": "object",
      "required": ["input_hash", "tier", "determinism", "seed", "deterministic"],
      "properties": {
        "input_hash": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$",
          "description": "BLAKE3 hash of input spec"
        },
        "output_hash": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$",
          "description": "Combined hash of outputs (required for tier 1)"
        },
        "tier": {
          "type": "integer",
          "enum": [1, 2, 3],
          "description": "Determinism tier"
        },
        "determinism": {
          "type": "string",
          "enum": ["byte_identical", "semantic_equivalent", "non_deterministic"]
        },
        "seed": {
          "type": "integer",
          "minimum": 0,
          "description": "Seed used for generation"
        },
        "deterministic": {
          "type": "boolean",
          "description": "Whether this run was deterministic"
        },
        "non_determinism_reason": {
          "type": "string",
          "description": "Explanation for non-determinism (tier 3 only)"
        }
      }
    },
    "ErrorEntry": {
      "type": "object",
      "required": ["code", "message"],
      "properties": {
        "code": {
          "type": "string",
          "description": "Error code"
        },
        "message": {
          "type": "string",
          "description": "Human-readable error message"
        },
        "context": {
          "description": "Additional error context"
        }
      }
    }
  }
}
```

## Examples

### Successful Tier 1 Generation

```json
{
  "manifest_version": 1,
  "success": true,
  "output_files": [
    {
      "path": "textures/custom.png",
      "hash": "a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890",
      "size": 4096,
      "kind": "primary",
      "format": "png"
    }
  ],
  "determinism_report": {
    "input_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "output_hash": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
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

### Successful Tier 2 Generation

```json
{
  "manifest_version": 1,
  "success": true,
  "output_files": [
    {
      "path": "meshes/output.glb",
      "hash": "a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890",
      "size": 8192,
      "kind": "primary",
      "format": "glb"
    }
  ],
  "determinism_report": {
    "input_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "tier": 2,
    "determinism": "semantic_equivalent",
    "seed": 42,
    "deterministic": true
  },
  "errors": [],
  "warnings": ["Blender version 4.0 used"],
  "duration_ms": 2500,
  "extension_version": "1.0.0",
  "metadata": {
    "blender_version": "4.0.0",
    "vertex_count": 1024,
    "face_count": 512
  }
}
```

### Failed Generation

```json
{
  "manifest_version": 1,
  "success": false,
  "output_files": [],
  "determinism_report": {
    "input_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "tier": 1,
    "determinism": "byte_identical",
    "seed": 42,
    "deterministic": true
  },
  "errors": [
    {
      "code": "INVALID_PARAM",
      "message": "Parameter 'style' must be one of: metal, wood, stone",
      "context": {
        "parameter": "style",
        "value": "invalid",
        "allowed": ["metal", "wood", "stone"]
      }
    }
  ],
  "warnings": [],
  "duration_ms": 10
}
```

## Hash Computation

### BLAKE3 Hashes

All hashes use BLAKE3 and are represented as 64 lowercase hexadecimal characters.

```python
import hashlib
# Note: Use blake3 library, not hashlib
import blake3

def compute_hash(data: bytes) -> str:
    return blake3.blake3(data).hexdigest()
```

```rust
fn compute_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}
```

### Input Hash

Compute the input hash from the canonical JSON representation of the spec:

1. Parse the spec JSON
2. Sort all object keys lexicographically (recursively)
3. Remove all whitespace
4. Compute BLAKE3 hash of the result

SpecCade provides this hash via `speccade_spec::canonical_spec_hash()`.

### Output Hash

For Tier 1 extensions, compute the combined output hash:

1. Compute BLAKE3 hash of each output file's contents
2. Sort hashes lexicographically
3. Concatenate sorted hashes
4. Compute BLAKE3 hash of the concatenation

```rust
fn combine_output_hashes(hashes: &[String]) -> String {
    let mut sorted = hashes.to_vec();
    sorted.sort();
    let combined = sorted.join("");
    blake3::hash(combined.as_bytes()).to_hex().to_string()
}
```

## Path Security

### Allowed Paths

Output file paths must:
- Be relative (no leading `/`)
- Not contain `..` components
- Not be empty
- Not end with `/`

### Examples

Valid paths:
- `output.png`
- `textures/albedo.png`
- `data/metadata.json`

Invalid paths:
- `/absolute/path.png` (absolute)
- `../escape.png` (traversal)
- `textures/` (directory)
- `` (empty)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (manifest.success must be true) |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Spec parse error |
| 4 | Generation failed |
| Other | Extension-specific errors |

## Validation Requirements

### Tier 1 Validation

SpecCade verifies:
1. manifest.json exists and is valid
2. All declared output files exist
3. All output file hashes match declared hashes
4. output_hash is present in determinism_report
5. input_hash matches SpecCade's computed hash

### Tier 2 Validation

SpecCade verifies:
1. manifest.json exists and is valid
2. All declared output files exist
3. input_hash matches SpecCade's computed hash
4. Metrics (if provided) are reasonable

### Tier 3 Validation

SpecCade verifies:
1. manifest.json exists and is valid
2. All declared output files exist
3. non_determinism_reason is provided

## Error Codes

Extensions should use meaningful error codes:

| Code | Description |
|------|-------------|
| `INVALID_SPEC` | Spec is malformed |
| `INVALID_PARAM` | Recipe parameter is invalid |
| `MISSING_PARAM` | Required parameter missing |
| `UNSUPPORTED_FORMAT` | Output format not supported |
| `GENERATION_FAILED` | Internal generation error |
| `IO_ERROR` | File I/O error |
| `TIMEOUT` | Internal timeout |

## Versioning

The manifest version is currently `1`. Future versions will maintain backward compatibility or clearly document migration paths.

## See Also

- [Plugin System Architecture](../architecture/plugin-system.md)
- [Extension Manifest Reference](#extension-manifest-schema)
- [Example Extension](../../../examples/extensions/simple-subprocess/)
