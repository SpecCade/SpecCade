# Lint Rules Reference

SpecCade's semantic quality lint system detects perceptual problems in generated assets. This document describes all 44 lint rules organized by asset type and severity level.

## Severity Levels

| Level | Description | Exit Code Impact |
|-------|-------------|------------------|
| Error | Definitely broken, should fail CI | Exit 1 |
| Warning | Likely problems, may indicate quality issues | Exit 1 with `--strict` |
| Info | Suggestions for improvement | No impact |

---

## Audio Rules (10)

### Error-Level

#### audio/clipping
**Severity:** Error
**Detection:** Sample values exceed +/-1.0
**Threshold:** Any sample with absolute value > 1.0
**Fix Guidance:** Reduce amplitude to prevent digital distortion
**Fix Metadata:**
- `fix_delta`: Multiplier to bring peak to 1.0 (calculated as `1.0 / peak`)
- `fix_param`: `amplitude`
- `expected_range`: `[-1.0, 1.0]`

#### audio/dc-offset
**Severity:** Error
**Detection:** Non-zero average sample value causes headroom loss and speaker stress
**Threshold:** Mean absolute value > 0.01
**Fix Guidance:** Add a highpass filter to remove DC offset
**Fix Metadata:**
- `fix_template`: `add highpass(cutoff=20)`
- `expected_range`: `[-0.01, 0.01]`

#### audio/silence
**Severity:** Error
**Detection:** Asset is entirely silent (peak amplitude below threshold)
**Threshold:** Peak amplitude < 0.001 (approximately -60dB)
**Fix Guidance:** Check envelope attack or oscillator amplitude
**Fix Metadata:**
- `expected_range`: `>= 0.001`

### Warning-Level

#### audio/too-quiet
**Severity:** Warning
**Detection:** Audio loudness is very low
**Threshold:** RMS level < -30 dBFS (target reference: -18 dBFS)
**Fix Guidance:** Increase amplitude to reach target level
**Fix Metadata:**
- `fix_delta`: Multiplier to reach -18 dBFS target
- `fix_param`: `amplitude`
- `expected_range`: `>= -30dB`

**Note:** Skipped if audio is completely silent (handled by `audio/silence` rule).

#### audio/too-loud
**Severity:** Warning
**Detection:** Audio is near clipping levels
**Threshold:** RMS level >= -6 dBFS
**Fix Guidance:** Add a limiter to prevent potential clipping
**Fix Metadata:**
- `fix_template`: `add limiter()`
- `expected_range`: `< -6dB`

**Note:** Skipped if audio is already clipping (handled by `audio/clipping` rule).

#### audio/harsh-highs
**Severity:** Warning
**Detection:** Excessive high-frequency energy above 8kHz
**Threshold:** High-frequency energy ratio >= 50%
**Fix Guidance:** Apply a lowpass filter to reduce harshness
**Fix Metadata:**
- `fix_template`: `lowpass(cutoff=6000)`
- `expected_range`: `< 50%`

**Note:** Only applies to sample rates >= 16kHz.

#### audio/muddy-lows
**Severity:** Warning
**Detection:** Excessive low-mid frequency energy (200-500Hz range)
**Threshold:** Low-mid energy ratio >= 60%
**Fix Guidance:** Apply a highpass filter to reduce muddiness
**Fix Metadata:**
- `fix_template`: `highpass(cutoff=80)`
- `expected_range`: `< 60%`

**Note:** Only applies to sample rates >= 1kHz.

#### audio/abrupt-end
**Severity:** Warning
**Detection:** Audio ends abruptly without fade-out
**Threshold:** Final 10ms amplitude >= 0.1
**Fix Guidance:** Increase envelope release for a smoother ending
**Fix Metadata:**
- `expected_range`: `< 0.1`

### Info-Level

#### audio/no-effects
**Severity:** Info
**Detection:** Audio has no spatial effects (reverb, delay)
**Threshold:** Spec recipe has no effects array or empty effects
**Fix Guidance:** Consider adding reverb for more natural sound
**Fix Metadata:**
- `fix_template`: `reverb()`

**Note:** Requires spec context to check effects chain. Does not fire without spec.

#### audio/mono-recommended
**Severity:** Info
**Detection:** Short sound effect uses stereo when mono would suffice
**Threshold:** Stereo audio with duration < 2 seconds
**Fix Guidance:** Use mono output format for short sound effects
**Fix Metadata:**
- `expected_range`: `mono for duration < 2s`

---

## Texture Rules (10)

### Error-Level

#### texture/all-black
**Severity:** Error
**Detection:** Image is entirely black
**Threshold:** Maximum pixel luminance value < 5
**Fix Guidance:** Check noise scale or color ramp
**Fix Metadata:**
- `expected_range`: `>= 5`

#### texture/all-white
**Severity:** Error
**Detection:** Image is entirely white
**Threshold:** Minimum pixel luminance value > 250
**Fix Guidance:** Check threshold or invert node
**Fix Metadata:**
- `expected_range`: `<= 250`

#### texture/corrupt-alpha
**Severity:** Error
**Detection:** Alpha channel is uniform (all 0 or all 255)
**Threshold:** All alpha values identical and either 0 (fully transparent) or 255 (fully opaque)
**Fix Guidance:** Check alpha source node
**Fix Metadata:**
- `expected_range`: `variable alpha values`

**Note:** Only applies to images with alpha channel (RGBA, GrayscaleAlpha).

### Warning-Level

#### texture/low-contrast
**Severity:** Warning
**Detection:** Image has low contrast (narrow value range)
**Threshold:** Standard deviation of luminance < 20
**Fix Guidance:** Increase color_ramp spread
**Fix Metadata:**
- `expected_range`: `>= 20`

#### texture/banding
**Severity:** Warning
**Detection:** Image shows color banding (visible stepping in gradients)
**Threshold:** Fewer than 32 unique values in any RGB channel
**Fix Guidance:** Add dithering noise
**Fix Metadata:**
- `expected_range`: `>= 32 per channel`

#### texture/tile-seam
**Severity:** Warning
**Detection:** Image has visible seams at tile edges
**Threshold:** Maximum edge discontinuity (left-right or top-bottom) > 50 luminance units
**Fix Guidance:** Enable seamless mode
**Fix Metadata:**
- `expected_range`: `<= 50`

#### texture/noisy
**Severity:** Warning
**Detection:** Image has excessive high-frequency noise
**Threshold:** Average local variance (4x4 windows) > 2500
**Fix Guidance:** Reduce octaves or add blur
**Fix Metadata:**
- `expected_range`: `<= 2500`

#### texture/color-cast
**Severity:** Warning
**Detection:** Image has a strong color cast (one channel dominates)
**Threshold:** Any RGB channel average > 1.5x the minimum channel average
**Fix Guidance:** Balance color ramp
**Fix Metadata:**
- `expected_range`: `channels within 1.5x of each other`

**Note:** Skipped for grayscale images or very dark images (min average < 10).

### Info-Level

#### texture/power-of-two
**Severity:** Info
**Detection:** Image dimensions are not power-of-two
**Threshold:** Width or height is not a power of 2 (1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, etc.)
**Fix Guidance:** Use 256/512/1024
**Fix Metadata:**
- `expected_range`: `power-of-two (e.g., 256, 512, 1024)`

#### texture/large-solid-regions
**Severity:** Info
**Detection:** Image has large solid regions
**Threshold:** More than 25% of pixels have identical luminance
**Fix Guidance:** Add subtle variation
**Fix Metadata:**
- `expected_range`: `<= 25%`

---

## Mesh Rules (12)

### Error-Level

#### mesh/non-manifold
**Severity:** Error
**Detection:** Non-manifold edges (edges shared by more than 2 faces)
**Threshold:** Any edge referenced by > 2 triangles
**Fix Guidance:** Remove duplicate faces or fix mesh topology
**Fix Metadata:**
- `expected_range`: `0 non-manifold edges`

#### mesh/degenerate-faces
**Severity:** Error
**Detection:** Zero-area triangles (degenerate faces)
**Threshold:** Triangle area < 1e-8 or duplicate vertex indices
**Fix Guidance:** Remove or merge vertices to eliminate zero-area faces
**Fix Metadata:**
- `expected_range`: `0 degenerate faces`

#### mesh/unweighted-verts
**Severity:** Error
**Detection:** Vertices with no bone weights in skinned meshes
**Threshold:** Total weight sum < 1e-6 for any vertex (only for meshes with skeleton)
**Fix Guidance:** Apply auto-weights or paint weights for unweighted vertices
**Fix Metadata:**
- `expected_range`: `0 unweighted vertices`

**Note:** Only applies to skinned meshes (meshes with skeleton/armature).

#### mesh/inverted-normals
**Severity:** Error
**Detection:** Normals pointing inward (inverted face orientation)
**Threshold:** More than 50% of faces have normals pointing toward mesh centroid
**Fix Guidance:** Recalculate normals with consistent outward orientation
**Fix Metadata:**
- `expected_range`: `< 50% inverted`

### Warning-Level

#### mesh/humanoid-proportions
**Severity:** Warning
**Detection:** Checks humanoid armature limb proportions against anatomical ranges
**Threshold:** Missing lower segment bones when upper segment bones exist (e.g., has "upper_arm" but no "forearm")
**Fix Guidance:** Add missing lower segment bones to complete limb hierarchy
**Fix Metadata:**
- `expected_range`: `ratio 0.7-1.3`

**Bone Patterns Checked:**
- `upper_arm` / `forearm`
- `upperarm` / `lowerarm`
- `arm_upper` / `arm_lower`
- `thigh` / `shin`
- `upper_leg` / `lower_leg`
- `upperleg` / `lowerleg`
- `leg_upper` / `leg_lower`

**Note:** Only applies to skinned meshes with skeleton bones.

#### mesh/uv-overlap
**Severity:** Warning
**Detection:** Overlapping UV islands
**Threshold:** Total UV area > 105% of UV space (1.0)
**Fix Guidance:** Repack UVs to eliminate overlapping islands
**Fix Metadata:**
- `expected_range`: `< 5% overlap`

#### mesh/uv-stretch
**Severity:** Warning
**Detection:** High UV distortion (stretch)
**Threshold:** More than 10% of faces have UV-to-3D area ratio > 2.0 or < 0.5
**Fix Guidance:** Adjust UV seams and unwrap settings to reduce distortion
**Fix Metadata:**
- `expected_range`: `< 10% faces stretched`

#### mesh/missing-material
**Severity:** Warning
**Detection:** Faces with no material assigned
**Threshold:** Any face without material index
**Fix Guidance:** Assign a material to all faces
**Fix Metadata:**
- `expected_range`: `0 faces without material`

#### mesh/excessive-ngons
**Severity:** Warning
**Detection:** Excessive use of n-gons (faces with more than 4 vertices)
**Threshold:** More than 20% faces with > 4 vertices
**Fix Guidance:** Triangulate n-gons for better compatibility

**Note:** glTF/GLB files are always triangulated by format specification, so this rule only applies to other mesh formats.

#### mesh/isolated-verts
**Severity:** Warning
**Detection:** Vertices not used by any face
**Threshold:** Any vertex not referenced by indices
**Fix Guidance:** Remove isolated vertices to reduce file size
**Fix Metadata:**
- `expected_range`: `0 isolated vertices`

### Info-Level

#### mesh/high-poly
**Severity:** Info
**Detection:** High polygon count meshes
**Threshold:** More than 50,000 triangles
**Fix Guidance:** Consider adding LOD levels or decimating for performance
**Fix Metadata:**
- `fix_param`: `triangle_count`
- `expected_range`: `<= 50000 triangles`

#### mesh/no-uvs
**Severity:** Info
**Detection:** Meshes without UV coordinates
**Threshold:** No UV data present when mesh has geometry
**Fix Guidance:** Add UV projection (box, cylindrical, or smart UV project)
**Fix Metadata:**
- `fix_template`: `mesh.uv_project(method="smart")`

---

## Music Rules (12)

### Error-Level

#### music/empty-pattern
**Severity:** Error
**Detection:** Pattern has no notes
**Threshold:** All notes are empty (`---`, `...`, `===`) or pattern has no note entries
**Fix Guidance:** Add notes or remove pattern
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns["<pattern_name>"]`

#### music/invalid-note
**Severity:** Error
**Detection:** Note outside MIDI range
**Threshold:** MIDI value < 0 or > 127 (valid: C-1 to G9)
**Fix Guidance:** Transpose to valid range (0-127, C-1 to G9)
**Fix Metadata:**
- `expected_range`: `0-127`
- `spec_path`: `recipe.params.patterns["<pattern>"].notes["<channel>"]`

#### music/empty-arrangement
**Severity:** Error
**Detection:** No patterns in arrangement
**Threshold:** Empty arrangement array
**Fix Guidance:** Add patterns to arrangement
**Fix Metadata:**
- `spec_path`: `recipe.params.arrangement`

### Warning-Level

#### music/parallel-octaves
**Severity:** Warning
**Detection:** Consecutive parallel octaves between voices
**Threshold:** Two voices move in parallel motion while maintaining an octave interval (12 semitones)
**Fix Guidance:** Use contrary motion or different intervals
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns["<pattern>"]`

**Music Theory Note:** Parallel octaves reduce voice independence, a common counterpoint violation.

#### music/parallel-fifths
**Severity:** Warning
**Detection:** Consecutive parallel fifths between voices
**Threshold:** Two voices move in parallel motion while maintaining a perfect fifth interval (7 semitones)
**Fix Guidance:** Use different intervals or contrary motion
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns["<pattern>"]`

**Music Theory Note:** Parallel fifths reduce harmonic interest, traditionally avoided in classical voice leading.

#### music/voice-crossing
**Severity:** Warning
**Detection:** Lower voice is above a higher voice
**Threshold:** Lower channel number (higher voice) has lower pitch than higher channel number
**Fix Guidance:** Adjust voicing to maintain proper voice leading
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns["<pattern>"]`

**Convention:** Channel 0 = soprano (highest), Channel 1 = alto, etc.

#### music/dense-pattern
**Severity:** Warning
**Detection:** Too many simultaneous notes in a row
**Threshold:** More than 8 simultaneous notes on any row
**Fix Guidance:** Reduce polyphony by removing or offsetting notes
**Fix Metadata:**
- `expected_range`: `1-8`

#### music/sparse-pattern
**Severity:** Warning
**Detection:** Pattern has very low cell occupancy
**Threshold:** Less than 5% of cells (rows x channels) contain notes
**Fix Guidance:** Add more notes or reduce pattern size
**Fix Metadata:**
- `expected_range`: `>= 5%`

**Note:** Only triggers if pattern has at least 1 note (empty patterns caught by `music/empty-pattern`).

#### music/extreme-tempo
**Severity:** Warning
**Detection:** BPM outside reasonable range
**Threshold:** BPM < 40 or BPM > 300
**Fix Guidance:** Adjust tempo to 40-300 BPM
**Fix Metadata:**
- `fix_param`: `bpm`
- `expected_range`: `40-300`

### Info-Level

#### music/unused-channel
**Severity:** Info
**Detection:** Channel has no notes across all patterns
**Threshold:** Declared channel with no notes in any pattern
**Fix Guidance:** Remove channel or add content to it
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns[*].notes["<channel>"]`

#### music/no-variation
**Severity:** Info
**Detection:** Same pattern repeats excessively
**Threshold:** Same pattern plays more than 4 times consecutively (including repeat counts)
**Fix Guidance:** Add variation (B-section) or reduce repeats
**Fix Metadata:**
- `expected_range`: `<= 4`
- `spec_path`: `recipe.params.arrangement`

#### music/unresolved-tension
**Severity:** Info
**Detection:** Song ends on dissonant interval
**Threshold:** Final chord contains tritone (6 semitones), minor 2nd (1 semitone), or major 7th (11 semitones)
**Fix Guidance:** Resolve to consonant interval (unison, 3rd, 5th, octave)
**Fix Metadata:**
- `spec_path`: `recipe.params.patterns["<last_pattern>"]`

**Dissonant Intervals:**
- Tritone: 6 semitones (e.g., C-F#)
- Minor 2nd: 1 semitone (e.g., C-Db)
- Major 7th: 11 semitones (e.g., C-B)

---

## Rule Summary by Severity

### Error Rules (12 total)

| Asset Type | Rule ID | Brief Description |
|------------|---------|-------------------|
| Audio | `audio/clipping` | Samples exceed +/-1.0 |
| Audio | `audio/dc-offset` | Non-zero average sample value |
| Audio | `audio/silence` | Entirely silent audio |
| Texture | `texture/all-black` | Image entirely black |
| Texture | `texture/all-white` | Image entirely white |
| Texture | `texture/corrupt-alpha` | Uniform alpha channel |
| Mesh | `mesh/non-manifold` | Edges shared by >2 faces |
| Mesh | `mesh/degenerate-faces` | Zero-area triangles |
| Mesh | `mesh/unweighted-verts` | Vertices without bone weights |
| Mesh | `mesh/inverted-normals` | Normals pointing inward |
| Music | `music/empty-pattern` | Pattern has no notes |
| Music | `music/invalid-note` | Note outside MIDI range |
| Music | `music/empty-arrangement` | No patterns in arrangement |

### Warning Rules (22 total)

| Asset Type | Rule ID | Brief Description |
|------------|---------|-------------------|
| Audio | `audio/too-quiet` | RMS below -30dB |
| Audio | `audio/too-loud` | RMS above -6dB |
| Audio | `audio/harsh-highs` | Excessive high frequencies |
| Audio | `audio/muddy-lows` | Excessive low-mid frequencies |
| Audio | `audio/abrupt-end` | No fade-out at end |
| Texture | `texture/low-contrast` | Narrow value range |
| Texture | `texture/banding` | Too few unique color values |
| Texture | `texture/tile-seam` | Visible seams at edges |
| Texture | `texture/noisy` | Excessive high-frequency noise |
| Texture | `texture/color-cast` | Strong single-channel dominance |
| Mesh | `mesh/humanoid-proportions` | Missing limb segments |
| Mesh | `mesh/uv-overlap` | Overlapping UV islands |
| Mesh | `mesh/uv-stretch` | High UV distortion |
| Mesh | `mesh/missing-material` | Faces without material |
| Mesh | `mesh/excessive-ngons` | Too many n-gons |
| Mesh | `mesh/isolated-verts` | Unused vertices |
| Music | `music/parallel-octaves` | Parallel octave motion |
| Music | `music/parallel-fifths` | Parallel fifth motion |
| Music | `music/voice-crossing` | Voice register crossing |
| Music | `music/dense-pattern` | >8 simultaneous notes |
| Music | `music/sparse-pattern` | <5% cell occupancy |
| Music | `music/extreme-tempo` | BPM outside 40-300 |

### Info Rules (10 total)

| Asset Type | Rule ID | Brief Description |
|------------|---------|-------------------|
| Audio | `audio/no-effects` | No spatial effects applied |
| Audio | `audio/mono-recommended` | Short stereo that could be mono |
| Texture | `texture/power-of-two` | Non-power-of-two dimensions |
| Texture | `texture/large-solid-regions` | >25% identical pixels |
| Mesh | `mesh/high-poly` | >50k triangles |
| Mesh | `mesh/no-uvs` | Missing UV coordinates |
| Music | `music/unused-channel` | Channel with no notes |
| Music | `music/no-variation` | Pattern repeats >4 times |
| Music | `music/unresolved-tension` | Ends on dissonant interval |

---

## Fix Metadata Fields

Lint issues may include metadata to assist automated fixes:

| Field | Description | Example |
|-------|-------------|---------|
| `fix_delta` | Numeric multiplier/adjustment value | `0.8` (multiply amplitude by 0.8) |
| `fix_param` | Parameter name to adjust in spec | `amplitude`, `bpm` |
| `fix_template` | Full expression to add/replace | `add highpass(cutoff=20)` |
| `actual_value` | Current measured value | `1.23`, `-35dB` |
| `expected_range` | Valid value range | `[-1.0, 1.0]`, `>= 5%` |
| `spec_path` | JSONPath to problematic spec field | `recipe.params.bpm` |
| `asset_location` | Location within asset | `pattern:intro:row16` |
