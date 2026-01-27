# Lint Rules Reference

SpecCade includes 44 semantic quality rules across four domains. Lint runs automatically after `speccade generate`.

> **SSOT:** Rule implementations in `crates/speccade-lint/src/rules/`.

## Severity Levels

| Level | Meaning | Exit code |
|-------|---------|-----------|
| **error** | Definitely broken | 1 |
| **warning** | Likely problem | 0 (1 with `--strict`) |
| **info** | Suggestion | 0 |

## Usage

```bash
speccade lint --input sounds/laser.wav
speccade lint --input audio.wav --spec audio.star    # with spec context
speccade lint --input mesh.glb --strict              # warnings fail too
speccade lint --input texture.png --disable-rule texture/power-of-two
speccade lint --input audio.wav --only-rules audio/clipping,audio/too-quiet
speccade lint --input-dir ./out --format json
```

---

## Audio Rules (10)

| Rule | Severity | Detects |
|------|----------|---------|
| `audio/clipping` | error | Samples exceeding +/-1.0 |
| `audio/dc-offset` | error | Mean sample > +/-0.01 |
| `audio/silence` | error | Peak < 0.001 (~-60 dBFS) |
| `audio/too-quiet` | warning | RMS < -30 dBFS |
| `audio/too-loud` | warning | RMS >= -6 dBFS (near clipping) |
| `audio/harsh-highs` | warning | >50% energy above 8 kHz |
| `audio/muddy-lows` | warning | >60% energy in 200-500 Hz |
| `audio/abrupt-end` | warning | Final 10 ms peak > 0.1 |
| `audio/no-effects` | info | Empty effects chain (needs spec context) |
| `audio/mono-recommended` | info | Stereo file < 2 seconds |

## Texture Rules (10)

| Rule | Severity | Detects |
|------|----------|---------|
| `texture/all-black` | error | Max luminance < 5 |
| `texture/all-white` | error | Min luminance > 250 |
| `texture/corrupt-alpha` | error | Uniform alpha (all 0 or all 255) in RGBA |
| `texture/low-contrast` | warning | Luminance std dev < 20 |
| `texture/banding` | warning | Any channel < 32 unique values |
| `texture/tile-seam` | warning | Edge luminance diff > 50 |
| `texture/noisy` | warning | Local 4x4 variance > 2500 |
| `texture/color-cast` | warning | Channel avg > 1.5x lowest channel |
| `texture/power-of-two` | info | Non-power-of-two dimensions |
| `texture/large-solid-regions` | info | Most common value > 25% of pixels |

## Mesh Rules (12)

| Rule | Severity | Detects |
|------|----------|---------|
| `mesh/non-manifold` | error | Edges shared by > 2 faces |
| `mesh/degenerate-faces` | error | Zero-area triangles |
| `mesh/unweighted-verts` | error | Vertices with no bone weights (skinned meshes) |
| `mesh/inverted-normals` | error | >50% faces with inward normals |
| `mesh/humanoid-proportions` | warning | Missing/mismatched limb bones |
| `mesh/uv-overlap` | warning | UV area > 1.05 |
| `mesh/uv-stretch` | warning | >10% faces with area ratio > 2.0 |
| `mesh/missing-material` | warning | Faces without material |
| `mesh/excessive-ngons` | warning | >20% faces with > 4 vertices |
| `mesh/isolated-verts` | warning | Vertices not in any face |
| `mesh/high-poly` | info | > 50,000 triangles |
| `mesh/no-uvs` | info | No UV coordinates |

## Music Rules (12)

| Rule | Severity | Detects |
|------|----------|---------|
| `music/empty-pattern` | error | Pattern with zero playable notes |
| `music/invalid-note` | error | MIDI note outside 0-127 |
| `music/empty-arrangement` | error | Empty arrangement array |
| `music/parallel-octaves` | warning | Consecutive parallel octaves between voices |
| `music/parallel-fifths` | warning | Consecutive parallel fifths between voices |
| `music/voice-crossing` | warning | Lower channel below higher channel |
| `music/dense-pattern` | warning | > 8 simultaneous notes on a row |
| `music/sparse-pattern` | warning | < 5% cell occupancy |
| `music/extreme-tempo` | warning | BPM < 40 or > 300 |
| `music/unused-channel` | info | Channel with no notes in any pattern |
| `music/no-variation` | info | Same pattern > 4 consecutive repeats |
| `music/unresolved-tension` | info | Song ends on dissonant interval |
