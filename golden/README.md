# Golden Test Fixtures

This directory contains golden test fixtures for SpecCade determinism verification.

## Directory Structure

```
golden/
  legacy/           # Legacy .spec.py files (for migration testing)
  speccade/
    specs/          # Canonical JSON spec files organized by asset type
      audio/
      music/
      texture/
      static_mesh/
      skeletal_mesh/
      skeletal_animation/
    expected/
      hashes/       # Expected BLAKE3 hashes for Tier-1 determinism verification
        audio/      # *.hash files for audio specs
        music/      # *.hash files for music specs
        texture/    # *.hash files for texture specs
      metrics/      # Expected metrics for Tier-2 backends (Blender)
        static_mesh/
        skeletal_mesh/
        skeletal_animation/
```

## Hash Verification (Tier-1)

Tier-1 backends (audio, music, texture) must produce byte-identical output for the same spec and seed. This is verified by comparing BLAKE3 hashes of generated output against expected hash files.

### Hash File Format

Each `.hash` file contains a single 64-character lowercase hexadecimal BLAKE3 hash:

```
a0869f7e31d1c12debb3741ff7058e1e9c35fd09dafc0396709dc6352e1c06f0
```

The filename matches the spec name (without extension): `simple_beep.hash` for `simple_beep.json`.

### Running Hash Verification

```bash
# Run hash verification tests
cargo test -p speccade-tests --test golden_hash_verification

# Run with verbose output
cargo test -p speccade-tests --test golden_hash_verification -- --nocapture
```

### Updating Expected Hashes

When intentionally changing backend output (e.g., fixing a synthesis algorithm), update the expected hashes:

```bash
SPECCADE_UPDATE_GOLDEN_HASHES=1 cargo test -p speccade-tests --test golden_hash_verification
```

This regenerates hash files for all specs that can be generated. Review changes carefully before committing.

### Adding New Golden Specs

1. Create the spec JSON file in `golden/speccade/specs/<type>/`
2. Run with `SPECCADE_UPDATE_GOLDEN_HASHES=1` to generate the hash file
3. Commit both the spec and the hash file

### Skipped Specs

Some specs reference external files via relative paths (e.g., music specs with external instrument references). These are skipped during hash verification if no hash file exists, as they cannot be generated in the test context.

## CI Integration

The golden hash verification tests run as part of the `golden-gates` CI job. Any hash mismatch or missing expected hash file will fail the CI build.

## Troubleshooting

### Hash mismatch

A hash mismatch indicates non-deterministic output or an intentional change:

1. If intentional: Update hashes with `SPECCADE_UPDATE_GOLDEN_HASHES=1`
2. If unintentional: Investigate the determinism regression

### Missing hash file

A spec generates successfully but has no expected hash file:

1. New spec: Run with `SPECCADE_UPDATE_GOLDEN_HASHES=1` to generate
2. External references: Spec may need external files that aren't available in test context

### Generation error

A spec with an expected hash file fails to generate:

1. Check if backend code changed
2. Check if spec format changed
3. Check dependencies (external references)
