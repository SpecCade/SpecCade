# XM/IT Format Differences

This document describes known differences between XM (FastTracker II Extended Module) and IT (Impulse Tracker) formats as implemented in SpecCade. These differences may affect playback behavior when the same music spec is rendered to both formats.

## Overview

SpecCade supports generating tracker music in both XM and IT formats from the same spec. While we aim for structural parity (matching instrument counts, pattern counts, rows per pattern, note placements), the two formats have inherent differences that can affect playback.

## Structural Parity Guarantees

When generating from the same spec, SpecCade guarantees:

- **Instrument count**: Same number of instruments
- **Sample count**: Same number of samples (in XM, samples are embedded in instruments)
- **Pattern count**: Same number of patterns
- **Rows per pattern**: Same row count for each corresponding pattern
- **Tempo/Speed**: Same initial tempo and speed values
- **BPM**: Same initial BPM

Use `speccade_backend_music::parity::check_parity()` to verify structural equivalence programmatically.

## Known Playback Differences

### 1. Volume Handling

| Aspect | XM | IT |
|--------|----|----|
| Volume range | 0-64 | 0-64 (sample), 0-128 (instrument global) |
| Default volume | Per-sample | Per-sample + instrument global volume |
| Volume column | 0x10-0x50 for volume, special commands above | Different encoding scheme |

**Impact**: Volume levels may sound slightly different between players due to mixing differences.

### 2. Effect Commands

| Effect | XM | IT | Notes |
|--------|----|----|-------|
| Arpeggio | 0xx | Jxx | Same behavior |
| Porta Up | 1xx | Fxx | Different command letters |
| Porta Down | 2xx | Exx | Different command letters |
| Tone Porta | 3xx | Gxx | Different command letters |
| Vibrato | 4xx | Hxx | Different command letters |
| Volume Slide | Axx | Dxx | IT has more granular control |
| Note Cut | ECx | SCx | XM cuts on tick x, IT similar |
| Note Delay | EDx | SDx | Similar behavior |
| Pattern Loop | E6x | SBx | Similar behavior |
| Set Tempo | Fxx | Txx | XM uses F for both speed/tempo |
| Set Speed | Fxx (1-1F) | Axx | Overlapping in XM |

**Impact**: Complex effect patterns may need format-specific adjustments. SpecCade's spec-to-format conversion handles common mappings, but edge cases may differ.

### 3. Envelope Behavior

| Aspect | XM | IT |
|--------|----|----|
| Envelope points | 12 max | 25 max |
| Envelope types | Volume, Panning | Volume, Panning, Pitch |
| Sustain loop | Sustain point | Sustain loop (begin/end) |
| Carry | Not supported | Supported |

**Impact**: IT allows more complex envelopes. When converting to XM, envelopes may be simplified.

### 4. New Note Actions (NNA)

| Feature | XM | IT |
|---------|----|----|
| NNA modes | Not supported | Cut, Continue, Note Off, Fade |
| Duplicate Check | Not supported | Supported (DCT/DCA) |

**Impact**: IT files can achieve more natural-sounding polyphony and legato. XM uses simpler note replacement behavior.

### 5. Sample Precision

| Aspect | XM | IT |
|--------|----|----|
| Bit depth | 8-bit or 16-bit | 8-bit or 16-bit |
| Sample storage | Delta-encoded (16-bit) | Delta or direct (configurable) |
| Compression | None | IT2.15 compression (optional) |

**Impact**: Identical at the waveform level when using 16-bit samples without compression.

### 6. Channel Count

| Aspect | XM | IT |
|--------|----|----|
| Max channels | 32 | 64 |
| Channel header | Explicit count | 64 slots (with enable/disable) |

**Impact**: IT files always reserve 64 channel slots. XM specifies only used channels. This is a structural difference that does not affect playback.

### 7. Frequency Tables

| Aspect | XM | IT |
|--------|----|----|
| Default | Linear | Linear or Amiga |
| Amiga mode | Supported | Supported |

**Impact**: Both default to linear frequency mode in SpecCade, ensuring consistent pitch behavior.

## QA Listening Checklist

When comparing XM and IT renders of the same spec, check:

1. **Volume balance**: Do instruments have similar relative volumes?
2. **Envelope shape**: Do notes have the correct attack/decay/sustain/release feel?
3. **Effect timing**: Do portamentos and vibratos sound correct?
4. **Loop points**: Do sustained instruments loop cleanly?
5. **Tempo consistency**: Does the BPM feel the same throughout?
6. **Note transitions**: Do slides and arpeggios sound musical?

## Recommended Players for Testing

For consistent playback during QA:

| Player | Platform | Notes |
|--------|----------|-------|
| OpenMPT | Windows/Linux | Reference implementation, supports both formats |
| libxmp | Cross-platform | Library-based, good for batch testing |
| XMPlay | Windows | Fast, accurate XM playback |
| Schism Tracker | Cross-platform | IT-native, good reference for IT behavior |

## Programmatic Parity Checking

Use the parity module to verify structural equivalence:

```rust
use speccade_backend_music::parity::{check_parity, ParityReport};

let xm_data: Vec<u8> = /* generated XM bytes */;
let it_data: Vec<u8> = /* generated IT bytes */;

let report: ParityReport = check_parity(&xm_data, &it_data)?;

if report.is_parity {
    println!("Structural parity confirmed");
} else {
    for mismatch in &report.mismatches {
        eprintln!("Mismatch: {}", mismatch);
    }
}
```

The parity check verifies:
- Instrument count
- Sample count
- Pattern count
- Rows per pattern
- Tempo and BPM

Note: Parity checking is structural only. It does not compare audio content or guarantee identical playback.

## References

- [XM Format Specification](https://www.celersms.com/doc/XM_file_format.pdf)
- [ITTECH.TXT (IT Format Specification)](https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT)
- [OpenMPT Documentation](https://wiki.openmpt.org/)
