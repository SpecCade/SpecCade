# Sample Library for SpecCade Golden Instruments

This directory contains shared .wav sample files for sample-based instruments.

## Directory Structure

```
samples/
  piano/          # Acoustic piano samples
    c4.wav        # Middle C (MIDI 60)
    c5.wav        # C5 (MIDI 72)
    c3.wav        # C3 (MIDI 48)
    ...
  bass/           # Electric bass samples
    e1.wav        # Low E (MIDI 28)
    a1.wav        # A string (MIDI 33)
    d2.wav        # D string (MIDI 38)
    g2.wav        # G string (MIDI 43)
  drums/          # Drum kit samples
    kick.wav      # Bass drum
    snare.wav     # Snare drum
    hihat.wav     # Closed hi-hat
    crash.wav     # Crash cymbal
    tom_hi.wav    # High tom
    tom_mid.wav   # Mid tom
    tom_low.wav   # Low tom
  strings/        # String ensemble samples
    violin_c4.wav # Violin at C4
    cello_c3.wav  # Cello at C3
    viola_c4.wav  # Viola at C4
  brass/          # Brass section samples
    trumpet_c4.wav  # Trumpet at C4
    trombone_c3.wav # Trombone at C3
    french_horn_f3.wav # French horn at F3
```

## Sample Requirements

All samples should meet these specifications for compatibility:

| Property | Value |
|----------|-------|
| Format | WAV (PCM) |
| Bit depth | 16-bit or 24-bit |
| Sample rate | 44100 Hz (preferred) or 22050 Hz |
| Channels | Mono (stereo will be converted to mono) |

The SpecCade audio backend will:
- Convert stereo to mono by averaging channels
- Resample to 22050 Hz for tracker modules
- Normalize audio levels as needed

## Obtaining Samples

For a complete sample library, you can:

1. **Use free sample packs** (CC0 or compatible licenses):
   - [Freesound.org](https://freesound.org) - CC0 samples
   - [Philharmonia Orchestra](https://philharmonia.co.uk/resources/sound-samples/) - Free orchestral samples
   - [University of Iowa EMS](http://theremin.music.uiowa.edu/MIS.html) - Musical instrument samples

2. **Generate with synthesis tools**:
   - Use SpecCade's synthesis-based instruments for placeholder samples
   - Export from a DAW or synthesizer

3. **Record your own**:
   - Ensure consistent recording levels
   - Trim to include just the note attack, sustain, and natural decay

## License

All samples in this directory should be CC0-1.0 (public domain) or have compatible licensing
for inclusion in the SpecCade golden corpus.

## Note Naming Convention

Samples use scientific pitch notation:
- `c4.wav` = Middle C (MIDI note 60, 261.63 Hz)
- `a4.wav` = Concert A (MIDI note 69, 440 Hz)
- `e1.wav` = Low E (MIDI note 28, 41.20 Hz)

For multi-sample instruments, provide samples at key intervals (typically every octave or
every major third) for best pitch-shifting quality.
