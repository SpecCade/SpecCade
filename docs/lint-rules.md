# Lint Rules Reference

SpecCade includes 44 semantic quality rules that detect perceptual problems in generated assets. Rules are organized into four domains: audio, texture, mesh, and music.

## Severity Levels

| Level | Meaning | Exit code |
|-------|---------|-----------|
| **error** | Definitely broken, fails by default | 1 |
| **warning** | Likely problem, fails with `--strict` | 0 (1 with `--strict`) |
| **info** | Suggestion or stylistic preference | 0 |

## Usage

```bash
# Lint a single asset
speccade lint --input sounds/laser.wav

# Lint with original spec (enables spec_path in output)
speccade lint --input audio.wav --spec audio.star

# Strict mode (warnings also fail)
speccade lint --input mesh.glb --strict

# Disable specific rules
speccade lint --input texture.png --disable-rule texture/power-of-two

# Only run specific rules
speccade lint --input audio.wav --only-rules audio/clipping,audio/too-quiet

# JSON output
speccade lint --input-dir ./out --format json
```

Lint also runs automatically after `speccade generate` and results appear in the `lint` section of `report.json`.

---

## Audio Rules (10)

### audio/clipping (error)

**Detects:** Sample values exceeding +/-1.0, causing digital distortion.
**Threshold:** Any sample with absolute value > 1.0.
**Fix:** Reduce amplitude. `fix_delta` provides the multiplier (1.0 / peak). `fix_param`: `amplitude`.
**Spec path:** Amplitude or gain parameter in synthesis layer.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/clipping",
  "severity": "error",
  "message": "Audio contains 142 samples exceeding +/-1.0 (peak: 1.350)",
  "actual_value": "1.350",
  "expected_range": "[-1.0, 1.0]",
  "suggestion": "Reduce amplitude to prevent clipping",
  "fix_delta": 0.741,
  "fix_param": "amplitude"
}
```

</details>

### audio/dc-offset (error)

**Detects:** Non-zero average sample value, which wastes headroom and stresses speakers.
**Threshold:** Mean sample value > +/-0.01.
**Fix:** Add a highpass filter at 20 Hz to remove DC offset.
**Spec path:** Synthesis filter chain.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/dc-offset",
  "severity": "error",
  "message": "Audio has DC offset of 0.3000 (threshold: +/-0.01)",
  "actual_value": "0.3000",
  "expected_range": "[-0.01, 0.01]",
  "suggestion": "Add a highpass filter to remove DC offset",
  "fix_template": "add highpass(cutoff=20)"
}
```

</details>

### audio/silence (error)

**Detects:** Entirely silent audio (peak amplitude < 0.001).
**Threshold:** Peak < 0.001 (~-60 dBFS).
**Fix:** Check envelope attack or oscillator amplitude settings.
**Spec path:** Oscillator or envelope parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/silence",
  "severity": "error",
  "message": "Audio is silent (peak: 0.000000, RMS: -inf dB)",
  "actual_value": "0.000000",
  "expected_range": ">= 0.001",
  "suggestion": "Check envelope attack or oscillator amplitude"
}
```

</details>

### audio/too-quiet (warning)

**Detects:** Very low loudness level.
**Threshold:** RMS < -30 dBFS (target: -18 dBFS).
**Fix:** Increase amplitude by the suggested multiplier. `fix_delta` provides the gain factor. `fix_param`: `amplitude`.
**Spec path:** Amplitude or gain parameter.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/too-quiet",
  "severity": "warning",
  "message": "Audio is too quiet (RMS: -42.0dB, threshold: -30dB)",
  "actual_value": "-42.0dB",
  "expected_range": ">= -30dB",
  "suggestion": "Increase amplitude by ~12.6x to reach -18dB",
  "fix_delta": 12.6,
  "fix_param": "amplitude"
}
```

</details>

### audio/too-loud (warning)

**Detects:** Near-clipping loudness levels (but not yet clipping).
**Threshold:** RMS >= -6 dBFS and peak <= 1.0.
**Fix:** Add a limiter to prevent potential clipping.
**Spec path:** Effects chain.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/too-loud",
  "severity": "warning",
  "message": "Audio is very loud (RMS: -3.2dB, peak: 0.950)",
  "actual_value": "-3.2dB",
  "expected_range": "< -6dB",
  "suggestion": "Add a limiter to prevent potential clipping",
  "fix_template": "add limiter()"
}
```

</details>

### audio/harsh-highs (warning)

**Detects:** Excessive high-frequency energy above 8 kHz.
**Threshold:** High-frequency energy ratio > 50%. Only checked when sample rate >= 16 kHz.
**Fix:** Apply a lowpass filter to reduce harshness.
**Spec path:** `recipe.params.layers[*].synthesis`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/harsh-highs",
  "severity": "warning",
  "message": "Audio has excessive high-frequency energy (62% above 8kHz)",
  "actual_value": "62%",
  "expected_range": "< 50%",
  "suggestion": "Apply a lowpass filter to reduce harshness",
  "fix_template": "lowpass(cutoff=6000)"
}
```

</details>

### audio/muddy-lows (warning)

**Detects:** Excessive low-mid frequency energy in the 200-500 Hz range.
**Threshold:** Low-mid energy ratio > 60%.
**Fix:** Apply a highpass filter to reduce muddiness.
**Spec path:** Synthesis filter chain.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/muddy-lows",
  "severity": "warning",
  "message": "Audio has excessive low-mid energy (72% in 200-500Hz)",
  "actual_value": "72%",
  "expected_range": "< 60%",
  "suggestion": "Apply a highpass filter to reduce muddiness",
  "fix_template": "highpass(cutoff=80)"
}
```

</details>

### audio/abrupt-end (warning)

**Detects:** Audio ending abruptly without fade-out.
**Threshold:** Final 10 ms peak amplitude > 0.1.
**Fix:** Increase envelope release for a smoother ending.
**Spec path:** Envelope release parameter.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/abrupt-end",
  "severity": "warning",
  "message": "Audio ends abruptly (final 10ms amplitude: 0.450)",
  "actual_value": "0.450",
  "expected_range": "< 0.1",
  "suggestion": "Increase envelope release for a smoother ending"
}
```

</details>

### audio/no-effects (info)

**Detects:** Dry signal with no spatial processing (reverb, delay). Requires spec context to inspect the effects chain.
**Threshold:** Empty effects array in recipe params.
**Fix:** Add reverb or other spatial effects.
**Spec path:** `recipe.params.effects`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/no-effects",
  "severity": "info",
  "message": "Audio has no spatial effects applied",
  "suggestion": "Consider adding reverb for more natural sound",
  "fix_template": "reverb()"
}
```

</details>

### audio/mono-recommended (info)

**Detects:** Stereo files for short sound effects where mono would reduce file size.
**Threshold:** Stereo (2+ channels) and duration < 2 seconds.
**Fix:** Use mono output format for short sound effects.
**Spec path:** Output format settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "audio/mono-recommended",
  "severity": "info",
  "message": "Short audio (0.50s) uses stereo - mono would reduce file size",
  "actual_value": "2 channels, 0.50s",
  "expected_range": "mono for duration < 2s",
  "suggestion": "Use mono output format for short sound effects"
}
```

</details>

---

## Texture Rules (10)

### texture/all-black (error)

**Detects:** Image is entirely black.
**Threshold:** Maximum pixel luminance < 5.
**Fix:** Check noise scale or color ramp parameters.
**Spec path:** Noise or color ramp node.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/all-black",
  "severity": "error",
  "message": "Image is entirely black",
  "actual_value": "max_value=0",
  "expected_range": ">= 5",
  "suggestion": "Check noise scale or color ramp"
}
```

</details>

### texture/all-white (error)

**Detects:** Image is entirely white.
**Threshold:** Minimum pixel luminance > 250.
**Fix:** Check threshold or invert node.
**Spec path:** Threshold or invert node.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/all-white",
  "severity": "error",
  "message": "Image is entirely white",
  "actual_value": "min_value=255",
  "expected_range": "<= 250",
  "suggestion": "Check threshold or invert node"
}
```

</details>

### texture/corrupt-alpha (error)

**Detects:** RGBA image with uniform alpha channel (all 0 or all 255). Only checks images that have an alpha channel.
**Threshold:** All alpha values identical and equal to 0 or 255.
**Fix:** Check alpha source node.
**Spec path:** Alpha source node configuration.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/corrupt-alpha",
  "severity": "error",
  "message": "Alpha channel is uniform (all transparent)",
  "actual_value": "alpha=0",
  "expected_range": "variable alpha values",
  "suggestion": "Check alpha source node"
}
```

</details>

### texture/low-contrast (warning)

**Detects:** Narrow value range across the image.
**Threshold:** Luminance standard deviation < 20.
**Fix:** Increase color_ramp spread.
**Spec path:** Color ramp parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/low-contrast",
  "severity": "warning",
  "message": "Image has low contrast",
  "actual_value": "std_dev=4.50",
  "expected_range": ">= 20",
  "suggestion": "Increase color_ramp spread"
}
```

</details>

### texture/banding (warning)

**Detects:** Visible color stepping from too few unique values.
**Threshold:** Any channel has < 32 unique values.
**Fix:** Add dithering noise.
**Spec path:** Output bit depth or dithering settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/banding",
  "severity": "warning",
  "message": "Image shows color banding (R:4, G:4, B:4 unique values)",
  "actual_value": "unique_values: R=4, G=4, B=4",
  "expected_range": ">= 32 per channel",
  "suggestion": "Add dithering noise"
}
```

</details>

### texture/tile-seam (warning)

**Detects:** Edge discontinuity that would cause visible seams when tiled.
**Threshold:** Maximum edge luminance difference > 50 (comparing left/right and top/bottom edges).
**Fix:** Enable seamless mode.
**Spec path:** Tiling or seamless mode setting.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/tile-seam",
  "severity": "warning",
  "message": "Image has visible tile seams (max edge diff: 128.0)",
  "actual_value": "edge_diff=128.0",
  "expected_range": "<= 50",
  "suggestion": "Enable seamless mode"
}
```

</details>

### texture/noisy (warning)

**Detects:** Excessive high-frequency noise measured by local variance in 4x4 windows.
**Threshold:** Average local variance > 2500.
**Fix:** Reduce noise octaves or add blur.
**Spec path:** Noise octaves or blur settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/noisy",
  "severity": "warning",
  "message": "Image has excessive noise (local variance: 3200.5)",
  "actual_value": "local_variance=3200.5",
  "expected_range": "<= 2500",
  "suggestion": "Reduce octaves or add blur"
}
```

</details>

### texture/color-cast (warning)

**Detects:** Strong single-channel dominance in RGB images. Skips grayscale images and very dark images (min channel avg < 10).
**Threshold:** Any channel average > 1.5x the lowest channel average.
**Fix:** Balance color ramp.
**Spec path:** Color ramp or color balance settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/color-cast",
  "severity": "warning",
  "message": "Image has red color cast (4.00x other channels)",
  "actual_value": "R=200.0, G=50.0, B=50.0",
  "expected_range": "channels within 1.5x of each other",
  "suggestion": "Balance color ramp"
}
```

</details>

### texture/power-of-two (info)

**Detects:** Non-power-of-two image dimensions.
**Threshold:** Width or height is not 2^n.
**Fix:** Use standard power-of-two sizes (256, 512, 1024).
**Spec path:** Output dimensions.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/power-of-two",
  "severity": "info",
  "message": "Image has non-power-of-two dimensions (width=300, height=300)",
  "actual_value": "300x300",
  "expected_range": "power-of-two (e.g., 256, 512, 1024)",
  "suggestion": "Use 256/512/1024"
}
```

</details>

### texture/large-solid-regions (info)

**Detects:** Large areas of identical pixel values in the luminance histogram.
**Threshold:** Most common luminance value covers > 25% of pixels.
**Fix:** Add subtle variation to break up solid regions.
**Spec path:** Noise or variation parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "texture/large-solid-regions",
  "severity": "info",
  "message": "Image has large solid regions (100.0% identical pixels)",
  "actual_value": "100.0%",
  "expected_range": "<= 25%",
  "suggestion": "Add subtle variation"
}
```

</details>

---

## Mesh Rules (12)

### mesh/non-manifold (error)

**Detects:** Non-manifold edges (edges shared by more than 2 faces).
**Threshold:** Any edge with face count > 2.
**Fix:** Remove duplicate faces or fix mesh topology.
**Spec path:** Mesh generation parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/non-manifold",
  "severity": "error",
  "message": "Found 3 non-manifold edge(s) shared by more than 2 faces",
  "actual_value": "3 edges",
  "expected_range": "0 non-manifold edges",
  "suggestion": "Remove duplicate faces or fix mesh topology"
}
```

</details>

### mesh/degenerate-faces (error)

**Detects:** Zero-area triangles (degenerate faces), including triangles with duplicate vertex indices.
**Threshold:** Face area < 1e-8.
**Fix:** Remove degenerate faces or merge coincident vertices.
**Spec path:** Mesh generation parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/degenerate-faces",
  "severity": "error",
  "message": "Found 5 degenerate triangle(s) with zero or near-zero area",
  "actual_value": "5 faces",
  "expected_range": "0 degenerate faces",
  "suggestion": "Remove or merge vertices to eliminate zero-area faces"
}
```

</details>

### mesh/unweighted-verts (error)

**Detects:** Vertices with no bone weights in skinned meshes. Only applies when the mesh has a skeleton.
**Threshold:** Weight sum < 1e-6 for any vertex, or no weight data at all.
**Fix:** Apply auto-weights or manual weight painting.
**Spec path:** Skinning or weight parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/unweighted-verts",
  "severity": "error",
  "message": "Found 12 vertex/vertices with no bone weights",
  "actual_value": "12 unweighted",
  "expected_range": "0 unweighted vertices",
  "suggestion": "Apply auto-weights or paint weights for unweighted vertices"
}
```

</details>

### mesh/inverted-normals (error)

**Detects:** Normals pointing inward (away from mesh centroid). Uses the dot product between face normal and centroid-to-face vector.
**Threshold:** > 50% of faces have inward-pointing normals.
**Fix:** Recalculate normals with consistent outward orientation.
**Spec path:** Normal generation settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/inverted-normals",
  "severity": "error",
  "message": "Found 180 of 200 faces with normals pointing inward",
  "actual_value": "90.0% inverted",
  "expected_range": "< 50% inverted",
  "suggestion": "Recalculate normals with consistent outward orientation"
}
```

</details>

### mesh/humanoid-proportions (warning)

**Detects:** Incorrect limb proportions in humanoid armatures by checking standard bone name patterns (upper_arm/forearm, thigh/shin, etc.).
**Threshold:** Missing lower segment bones when upper segments exist. Ratio range: 0.7-1.3.
**Fix:** Add missing lower segment bones or adjust bone lengths.
**Spec path:** Armature or skeleton parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/humanoid-proportions",
  "severity": "warning",
  "message": "Found upper segment bones (upper_arm) but missing lower segment (forearm)",
  "asset_location": "bone:upper_arm_l",
  "expected_range": "ratio 0.7-1.3",
  "suggestion": "Add missing lower segment bones to complete limb hierarchy"
}
```

</details>

### mesh/uv-overlap (warning)

**Detects:** Overlapping UV islands by measuring total UV area.
**Threshold:** Total UV area > 1.05 (>5% overlap in UV space).
**Fix:** Repack UVs to eliminate overlapping islands.
**Spec path:** UV unwrap settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/uv-overlap",
  "severity": "warning",
  "message": "UV islands overlap by approximately 15.0%",
  "actual_value": "15.0% overlap",
  "expected_range": "< 5% overlap",
  "suggestion": "Repack UVs to eliminate overlapping islands"
}
```

</details>

### mesh/uv-stretch (warning)

**Detects:** High UV distortion by comparing 3D triangle area to UV triangle area.
**Threshold:** Area ratio > 2.0 or < 0.5, and > 10% of faces affected.
**Fix:** Adjust UV seams and unwrap settings to reduce distortion.
**Spec path:** UV unwrap parameters.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/uv-stretch",
  "severity": "warning",
  "message": "25.0% of faces have high UV distortion (stretch ratio > 2)",
  "actual_value": "25.0% stretched",
  "expected_range": "< 10% faces stretched",
  "suggestion": "Adjust UV seams and unwrap settings to reduce distortion"
}
```

</details>

### mesh/missing-material (warning)

**Detects:** Faces with no material assigned (primitives without a material index in glTF).
**Threshold:** Any face with no material.
**Fix:** Assign a material to all faces.
**Spec path:** Material assignment settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/missing-material",
  "severity": "warning",
  "message": "8 face(s) have no material assigned",
  "actual_value": "8 faces",
  "expected_range": "0 faces without material",
  "suggestion": "Assign a material to all faces"
}
```

</details>

### mesh/excessive-ngons (warning)

**Detects:** Faces with more than 4 vertices. Note: glTF/GLB files are always triangulated so this rule only applies to other formats (OBJ, FBX).
**Threshold:** > 20% of faces have > 4 vertices.
**Fix:** Triangulate the mesh.
**Spec path:** Mesh export or triangulation settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/excessive-ngons",
  "severity": "warning",
  "message": "35% of faces are n-gons (more than 4 vertices)",
  "actual_value": "35%",
  "expected_range": "<= 20%",
  "suggestion": "Triangulate mesh"
}
```

</details>

### mesh/isolated-verts (warning)

**Detects:** Vertices not referenced by any face.
**Threshold:** Any vertex not used in the index buffer.
**Fix:** Remove isolated vertices to reduce file size.
**Spec path:** Mesh cleanup settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/isolated-verts",
  "severity": "warning",
  "message": "Found 20 isolated vertex/vertices not used by any face",
  "actual_value": "20 isolated",
  "expected_range": "0 isolated vertices",
  "suggestion": "Remove isolated vertices to reduce file size"
}
```

</details>

### mesh/high-poly (info)

**Detects:** Meshes exceeding 50,000 triangles.
**Threshold:** Triangle count > 50,000.
**Fix:** Add LOD levels or decimate for performance. `fix_param`: `triangle_count`.
**Spec path:** LOD or decimation settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/high-poly",
  "severity": "info",
  "message": "Mesh has 75000 triangles (threshold: 50000)",
  "actual_value": "75000 triangles",
  "expected_range": "<= 50000 triangles",
  "suggestion": "Consider adding LOD levels or decimating for performance",
  "fix_param": "triangle_count"
}
```

</details>

### mesh/no-uvs (info)

**Detects:** Meshes with geometry but no UV coordinates.
**Threshold:** Positions present but UV array is empty.
**Fix:** Add UV projection (box, cylindrical, or smart UV project).
**Spec path:** UV generation settings.

<details><summary>Example issue</summary>

```json
{
  "rule_id": "mesh/no-uvs",
  "severity": "info",
  "message": "Mesh has no UV coordinates",
  "suggestion": "Add UV projection (box, cylindrical, or smart UV project)",
  "fix_template": "mesh.uv_project(method=\"smart\")"
}
```

</details>

---

## Music Rules (12)

### music/empty-pattern (error)

**Detects:** Patterns containing no notes (all cells empty or only note-off/empty markers).
**Threshold:** Zero playable notes in a pattern.
**Fix:** Add notes or remove the pattern.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/empty-pattern",
  "severity": "error",
  "message": "Pattern 'intro' has no notes",
  "spec_path": "recipe.params.patterns[\"intro\"]",
  "asset_location": "pattern:intro",
  "suggestion": "Add notes or remove pattern"
}
```

</details>

### music/invalid-note (error)

**Detects:** Notes outside the valid MIDI range (0-127, C-1 to G9).
**Threshold:** MIDI number < 0 or > 127.
**Fix:** Transpose to valid range.
**Spec path:** `recipe.params.patterns["<name>"].notes["<channel>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/invalid-note",
  "severity": "error",
  "message": "Note 'C12' at row 0 channel 0 is outside MIDI range",
  "spec_path": "recipe.params.patterns[\"test\"].notes[\"0\"]",
  "asset_location": "pattern:test:row0:ch0",
  "actual_value": "156",
  "expected_range": "0-127",
  "suggestion": "Transpose to valid range (0-127, C-1 to G9)"
}
```

</details>

### music/empty-arrangement (error)

**Detects:** Songs with no patterns in the arrangement.
**Threshold:** Arrangement array length = 0.
**Fix:** Add patterns to the arrangement.
**Spec path:** `recipe.params.arrangement`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/empty-arrangement",
  "severity": "error",
  "message": "Song arrangement is empty",
  "spec_path": "recipe.params.arrangement",
  "suggestion": "Add patterns to arrangement"
}
```

</details>

### music/parallel-octaves (warning)

**Detects:** Consecutive parallel octaves between two voices (both voices moving in the same direction by the same interval while an octave apart).
**Threshold:** Two consecutive rows where the same voice pair maintains an octave interval with parallel motion.
**Fix:** Use contrary motion or different intervals.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/parallel-octaves",
  "severity": "warning",
  "message": "Parallel octaves between channels 0 and 1 at rows 0-4",
  "spec_path": "recipe.params.patterns[\"verse\"]",
  "asset_location": "pattern:verse:rows0-4",
  "suggestion": "Use contrary motion or different intervals"
}
```

</details>

### music/parallel-fifths (warning)

**Detects:** Consecutive parallel fifths between two voices (both voices moving in parallel while a fifth apart).
**Threshold:** Two consecutive rows where the same voice pair maintains a perfect fifth (7 semitones mod 12) with parallel motion.
**Fix:** Use different intervals or contrary motion.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/parallel-fifths",
  "severity": "warning",
  "message": "Parallel fifths between channels 0 and 2 at rows 8-12",
  "spec_path": "recipe.params.patterns[\"chorus\"]",
  "asset_location": "pattern:chorus:rows8-12",
  "suggestion": "Use different intervals or contrary motion"
}
```

</details>

### music/voice-crossing (warning)

**Detects:** Lower-numbered channel (higher voice) sounding below a higher-numbered channel (lower voice).
**Threshold:** Any row where channel N has a lower pitch than channel N+1.
**Fix:** Adjust voicing to maintain proper voice leading.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/voice-crossing",
  "severity": "warning",
  "message": "Voice crossing at row 16: channel 0 (pitch 48) below channel 1 (pitch 72)",
  "spec_path": "recipe.params.patterns[\"verse\"]",
  "asset_location": "pattern:verse:row16",
  "suggestion": "Adjust voicing to maintain proper voice leading"
}
```

</details>

### music/dense-pattern (warning)

**Detects:** Rows with more than 8 simultaneous notes.
**Threshold:** > 8 notes with valid MIDI values on the same row.
**Fix:** Reduce polyphony by removing or offsetting notes.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/dense-pattern",
  "severity": "warning",
  "message": "Row 0 has 10 simultaneous notes (max 8)",
  "spec_path": "recipe.params.patterns[\"dense\"]",
  "asset_location": "pattern:dense:row0",
  "actual_value": "10",
  "expected_range": "1-8",
  "suggestion": "Reduce polyphony by removing or offsetting notes"
}
```

</details>

### music/sparse-pattern (warning)

**Detects:** Patterns with very low note density.
**Threshold:** < 5% cell occupancy (cells with notes / total cells). Does not fire on completely empty patterns (handled by `music/empty-pattern`).
**Fix:** Add more notes or reduce pattern size.
**Spec path:** `recipe.params.patterns["<name>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/sparse-pattern",
  "severity": "warning",
  "message": "Pattern 'sparse' has only 0.4% cell occupancy (2/512 cells)",
  "spec_path": "recipe.params.patterns[\"sparse\"]",
  "asset_location": "pattern:sparse",
  "actual_value": "0.4%",
  "expected_range": ">=5%",
  "suggestion": "Add more notes or reduce pattern size"
}
```

</details>

### music/extreme-tempo (warning)

**Detects:** BPM values outside the reasonable range.
**Threshold:** BPM < 40 or BPM > 300.
**Fix:** Adjust tempo to 40-300 BPM range. `fix_param`: `bpm`.
**Spec path:** `recipe.params.bpm`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/extreme-tempo",
  "severity": "warning",
  "message": "Tempo 30 BPM is outside reasonable range",
  "spec_path": "recipe.params.bpm",
  "actual_value": "30",
  "expected_range": "40-300",
  "suggestion": "Adjust tempo to 40-300 BPM",
  "fix_param": "bpm"
}
```

</details>

### music/unused-channel (info)

**Detects:** Channels declared in the song that have no notes across any pattern.
**Threshold:** Zero notes in a channel across all patterns.
**Fix:** Remove the channel or add content to it.
**Spec path:** `recipe.params.patterns[*].notes["<channel>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/unused-channel",
  "severity": "info",
  "message": "Channel 3 has no notes in any pattern",
  "spec_path": "recipe.params.patterns[*].notes[\"3\"]",
  "asset_location": "channel:3",
  "suggestion": "Remove channel or add content to it"
}
```

</details>

### music/no-variation (info)

**Detects:** Same pattern repeating more than 4 times consecutively in the arrangement (after expanding repeats).
**Threshold:** > 4 consecutive occurrences of the same pattern.
**Fix:** Add a B-section or reduce repeats.
**Spec path:** `recipe.params.arrangement`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/no-variation",
  "severity": "info",
  "message": "Pattern 'A' repeats 8 times consecutively",
  "spec_path": "recipe.params.arrangement",
  "asset_location": "arrangement:index0",
  "actual_value": "8",
  "expected_range": "<=4",
  "suggestion": "Add variation (B-section) or reduce repeats"
}
```

</details>

### music/unresolved-tension (info)

**Detects:** Song ending on a dissonant interval (minor 2nd, tritone, or major 7th).
**Threshold:** Any dissonant interval (1, 6, or 11 semitones mod 12) in the final chord of the last pattern.
**Fix:** Resolve to a consonant interval (unison, 3rd, 5th, octave).
**Spec path:** `recipe.params.patterns["<last_pattern>"]`

<details><summary>Example issue</summary>

```json
{
  "rule_id": "music/unresolved-tension",
  "severity": "info",
  "message": "Song ends with tritone (6 semitones) between notes",
  "spec_path": "recipe.params.patterns[\"ending\"]",
  "asset_location": "pattern:ending:row63",
  "suggestion": "Resolve to consonant interval (unison, 3rd, 5th, octave)"
}
```

</details>
