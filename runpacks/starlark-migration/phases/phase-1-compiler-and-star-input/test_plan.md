# Phase 1 Test Plan

This document specifies the test coverage strategy for the Starlark input layer.

---

## 1. Unit Tests

Location: `crates/speccade-cli/src/compiler/tests.rs`

### 1.1 Starlark Parsing

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `parse_minimal_spec` | Minimal valid Starlark spec | Parses successfully |
| `parse_empty_file` | Empty Starlark file | Error: spec must return a dict |
| `parse_syntax_error` | Invalid Starlark syntax | Error with line number |
| `parse_unicode_strings` | Spec with Unicode in strings | Parses correctly |
| `parse_multiline_strings` | Triple-quoted strings | Parses correctly |

### 1.2 Value Conversion

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `convert_dict_to_object` | Starlark dict -> JSON object | Correct JSON object |
| `convert_list_to_array` | Starlark list -> JSON array | Correct JSON array |
| `convert_primitives` | string, int, float, bool, None | Correct JSON primitives |
| `convert_nested_structures` | Nested dicts and lists | Correct nested JSON |
| `convert_int_overflow` | Very large integers | Handles gracefully |
| `reject_function_value` | Dict containing a function | Error: cannot convert |
| `reject_struct_value` | Dict containing a struct | Error: cannot convert |

### 1.3 Spec Validation

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `validate_complete_spec` | Full valid spec from Starlark | Validation passes |
| `validate_missing_required` | Missing `asset_id` | Validation error E001 |
| `validate_invalid_type` | Wrong type for field | Validation error |
| `validate_unknown_field` | Extra field in spec | Validation error (deny_unknown_fields) |

### 1.4 Safety Limits

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `timeout_infinite_loop` | `while True: pass` | Error: timeout after 30s |
| `timeout_long_computation` | Very slow computation | Error: timeout |
| `no_recursion` | Recursive function call | Error: recursion not allowed |
| `memory_large_list` | `list(range(10000000))` | Error or completes (depends on limits) |

### 1.5 Error Messages

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `error_includes_line` | Syntax error | Error message includes line number |
| `error_includes_column` | Syntax error | Error message includes column |
| `error_includes_filename` | Any error | Error message includes filename |
| `error_runtime_context` | Runtime error | Error includes call stack |

---

## 2. Input Layer Unit Tests

Location: `crates/speccade-cli/src/input/tests.rs`

### 2.1 Extension Dispatch

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `dispatch_json_extension` | File with `.json` | Uses JSON parser |
| `dispatch_star_extension` | File with `.star` | Uses Starlark compiler |
| `dispatch_bzl_extension` | File with `.bzl` | Uses Starlark compiler |
| `dispatch_unknown_extension` | File with `.txt` | Error: unknown extension |
| `dispatch_no_extension` | File without extension | Error: unknown extension |
| `dispatch_case_insensitive` | File with `.JSON` | Uses JSON parser |

### 2.2 Source Provenance

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `provenance_json_source` | Load JSON file | `source_kind = "json"` |
| `provenance_star_source` | Load Starlark file | `source_kind = "starlark"` |
| `provenance_source_hash` | Load any file | `source_hash` is BLAKE3 hex |
| `provenance_json_hash_equals_spec` | Load JSON file | `source_hash == spec_hash` |
| `provenance_star_hash_differs` | Load Starlark file | `source_hash != spec_hash` |

### 2.3 Error Handling

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `error_file_not_found` | Non-existent path | Error: FileRead |
| `error_not_a_file` | Directory path | Error: FileRead |
| `error_permission_denied` | Unreadable file | Error: FileRead |

---

## 3. Integration Tests

Location: `crates/speccade-tests/tests/starlark_input.rs`

### 3.1 Eval Command

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `eval_json_outputs_same` | `eval` on JSON file | Output equals input (canonicalized) |
| `eval_star_outputs_json` | `eval` on Starlark file | Valid JSON output |
| `eval_star_pretty` | `eval --pretty` | Pretty-printed JSON |
| `eval_invalid_star` | `eval` on invalid Starlark | Exit code 1, error message |
| `eval_missing_file` | `eval` on non-existent file | Exit code 1, error message |

### 3.2 Validate Command

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `validate_star_valid` | Valid Starlark spec | Exit code 0, report.ok=true |
| `validate_star_invalid` | Invalid Starlark spec | Exit code 1, report.ok=false |
| `validate_star_syntax_error` | Starlark syntax error | Exit code 1, error in stderr |
| `validate_json_unchanged` | Existing JSON spec | Behavior unchanged |
| `validate_star_report_provenance` | Valid Starlark | Report has source_kind, source_hash |

### 3.3 Generate Command

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `generate_star_audio` | Starlark audio spec | Generates .wav file |
| `generate_star_texture` | Starlark texture spec | Generates .png file |
| `generate_star_music` | Starlark music spec | Generates audio files |
| `generate_star_report` | Any Starlark spec | Report written with provenance |
| `generate_json_unchanged` | Existing JSON spec | Output unchanged |

### 3.4 End-to-End Workflow

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `e2e_star_to_generate` | Full workflow: eval -> validate -> generate | All steps succeed |
| `e2e_eval_then_validate` | `eval spec.star > ir.json && validate ir.json` | Both succeed, same IR |

---

## 4. Golden Tests

Location: `golden/starlark/`

### 4.1 Fixture Files

Create the following test fixtures:

```
golden/starlark/
  minimal.star            # Minimal valid spec
  minimal.expected.json   # Expected canonical IR
  audio_sfx.star          # Audio spec with layers
  audio_sfx.expected.json
  texture_proc.star       # Procedural texture spec
  texture_proc.expected.json
  music_tracker.star      # Music tracker spec
  music_tracker.expected.json
  with_variants.star      # Spec with variants array
  with_variants.expected.json
```

### 4.2 Minimal Spec Example

**`minimal.star`:**
```python
# Minimal valid SpecCade spec
{
    "spec_version": 1,
    "asset_id": "test-minimal-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "format": "wav",
            "path": "output.wav",
            "primary": True,
        },
    ],
}
```

**`minimal.expected.json`:**
```json
{
  "spec_version": 1,
  "asset_id": "test-minimal-01",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "format": "wav",
      "path": "output.wav",
      "primary": true
    }
  ]
}
```

### 4.3 Golden Test Runner

```rust
// In crates/speccade-tests/tests/golden_starlark.rs

use std::path::Path;

#[test]
fn golden_minimal() {
    run_golden_test("minimal");
}

#[test]
fn golden_audio_sfx() {
    run_golden_test("audio_sfx");
}

fn run_golden_test(name: &str) {
    let star_path = format!("golden/starlark/{}.star", name);
    let expected_path = format!("golden/starlark/{}.expected.json", name);

    // Load and compile Starlark
    let result = load_spec(Path::new(&star_path)).unwrap();

    // Load expected JSON
    let expected_content = std::fs::read_to_string(&expected_path).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&expected_content).unwrap();

    // Compare canonical JSON
    let actual = result.spec.to_value().unwrap();
    assert_eq!(actual, expected, "Golden test {} failed", name);
}
```

### 4.4 Byte-Identical IR Verification

```rust
#[test]
fn golden_ir_byte_identical() {
    // Run eval twice on same input
    let ir1 = run_eval("golden/starlark/minimal.star");
    let ir2 = run_eval("golden/starlark/minimal.star");

    // Must be byte-identical
    assert_eq!(ir1, ir2, "IR not deterministic");
}

#[test]
fn golden_ir_hash_stable() {
    // Compile Starlark
    let result = load_spec(Path::new("golden/starlark/minimal.star")).unwrap();
    let hash = canonical_spec_hash(&result.spec).unwrap();

    // Hash must match expected
    let expected_hash = "..."; // Precomputed
    assert_eq!(hash, expected_hash);
}
```

---

## 5. Regression Tests

Location: `crates/speccade-tests/tests/regression_json.rs`

### 5.1 JSON Spec Unchanged

| Test Name | Description | Expected Result |
|-----------|-------------|-----------------|
| `json_spec_hash_unchanged` | Load existing golden JSON | Hash matches precomputed |
| `json_validate_unchanged` | Validate existing JSON | Same errors/warnings |
| `json_generate_unchanged` | Generate from existing JSON | Output bytes identical |
| `json_report_unchanged` | Generate report from JSON | Report structure unchanged |

### 5.2 Golden JSON Stability

```rust
#[test]
fn json_spec_hashes_stable() {
    let known_hashes = [
        ("golden/speccade/specs/audio/laser-01.json", "abc123..."),
        ("golden/speccade/specs/texture/brick-wall.json", "def456..."),
        // ... more known hashes
    ];

    for (path, expected_hash) in known_hashes {
        let content = std::fs::read_to_string(path).unwrap();
        let spec = Spec::from_json(&content).unwrap();
        let hash = canonical_spec_hash(&spec).unwrap();
        assert_eq!(hash, expected_hash, "Hash changed for {}", path);
    }
}
```

### 5.3 Behavior Parity

```rust
#[test]
fn json_validate_no_new_warnings() {
    // Validate all existing JSON specs
    for path in glob("golden/speccade/specs/**/*.json") {
        let content = std::fs::read_to_string(&path).unwrap();
        let spec = Spec::from_json(&content).unwrap();
        let result = validate_spec(&spec);

        // No new warnings (capture baseline first)
        let baseline = load_baseline(&path);
        assert_eq!(
            result.warnings.len(),
            baseline.warning_count,
            "New warnings for {}",
            path
        );
    }
}
```

---

## 6. Safety Tests

Location: `crates/speccade-cli/src/compiler/safety_tests.rs`

### 6.1 Timeout Tests

```rust
#[test]
fn timeout_enforced() {
    let source = r#"
while True:
    x = 1
{"spec_version": 1}
"#;

    let config = CompilerConfig {
        timeout_seconds: 1, // Short timeout for testing
        ..Default::default()
    };

    let result = compile("test.star", source, &config);
    assert!(matches!(result, Err(CompileError::Timeout { .. })));
}
```

### 6.2 Recursion Disabled

```rust
#[test]
fn recursion_disabled() {
    let source = r#"
def infinite():
    return infinite()

infinite()
"#;

    let result = compile("test.star", source, &CompilerConfig::default());
    assert!(result.is_err());
    // Error should mention recursion or call stack
}
```

### 6.3 No Dangerous Builtins

```rust
#[test]
fn no_file_access() {
    let source = r#"
open("/etc/passwd")  # Should not exist
"#;

    let result = compile("test.star", source, &CompilerConfig::default());
    assert!(result.is_err());
}

#[test]
fn no_random() {
    let source = r#"
random.randint(1, 10)  # Should not exist
"#;

    let result = compile("test.star", source, &CompilerConfig::default());
    assert!(result.is_err());
}
```

---

## 7. CLI Tests

Location: `crates/speccade-cli/src/main.rs` (test module)

### 7.1 Command Parsing

```rust
#[test]
fn test_cli_parses_eval() {
    let cli = Cli::try_parse_from(["speccade", "eval", "--spec", "spec.star"]).unwrap();
    match cli.command {
        Commands::Eval { spec, pretty } => {
            assert_eq!(spec, "spec.star");
            assert!(!pretty);
        }
        _ => panic!("expected eval command"),
    }
}

#[test]
fn test_cli_parses_eval_pretty() {
    let cli = Cli::try_parse_from([
        "speccade", "eval", "--spec", "spec.star", "--pretty"
    ]).unwrap();
    match cli.command {
        Commands::Eval { spec, pretty } => {
            assert_eq!(spec, "spec.star");
            assert!(pretty);
        }
        _ => panic!("expected eval command"),
    }
}
```

### 7.2 Help Text

```rust
#[test]
fn test_eval_help_mentions_starlark() {
    let output = Command::new("speccade")
        .arg("eval")
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starlark") || stdout.contains(".star"));
}
```

---

## 8. Test Coverage Summary

| Category | Tests | Coverage Target |
|----------|-------|-----------------|
| Compiler Unit | ~20 | Parsing, conversion, safety |
| Input Layer Unit | ~15 | Dispatch, provenance |
| Integration | ~20 | Commands, workflows |
| Golden | ~10 | IR stability |
| Regression | ~10 | JSON backward compat |
| Safety | ~5 | Timeout, recursion |
| CLI | ~5 | Parsing, help |
| **Total** | **~85** | |

---

## 9. CI Configuration

### Test Jobs

```yaml
# .github/workflows/test.yml

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test --all-features

  test-no-starlark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests without Starlark
        run: cargo test --no-default-features

  golden:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run golden tests
        run: cargo test -p speccade-tests golden

  regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run regression tests
        run: cargo test -p speccade-tests regression
```

### Safety Test Timeout

```yaml
  safety:
    runs-on: ubuntu-latest
    timeout-minutes: 5  # Safety tests should complete quickly
    steps:
      - uses: actions/checkout@v4
      - name: Run safety tests
        run: cargo test -p speccade-cli safety
```

---

## 10. Acceptance Criteria Mapping

| Criterion | Test Categories |
|-----------|-----------------|
| CLI accepts .json AND .star | Input dispatch, Integration (validate, generate) |
| New `eval` command | CLI parsing, Integration (eval) |
| Backends consume canonical Spec only | Integration (generate), Golden (IR stability) |
| Hashes computed on canonical IR | Golden (hash stability), Regression |
| No breaking changes for JSON | Regression (all) |
