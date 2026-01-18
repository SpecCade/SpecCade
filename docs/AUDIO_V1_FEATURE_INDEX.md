# FG Audio V1 Library Expansion — Feature Index

Source: `docs/ROADMAP.md` (Audio).

Guidance: implement in this order unless dependencies force a reorder.

## Filter types (missing)

- [x] Notch — `features/FILTER_01_NOTCH.md` (added Filter::Notch, reused BiquadCoeffs::notch)
- [x] Allpass — `features/FILTER_02_ALLPASS.md` (added Filter::Allpass, reused BiquadCoeffs::allpass)
- [x] Comb — `features/FILTER_03_COMB.md` (added Filter::Comb, refactored filter.rs to module)
- [x] Formant — `features/FILTER_04_FORMANT.md` (added Filter::Formant with vowel bank)
- [x] Ladder — `features/FILTER_05_LADDER.md` (added Filter::Ladder, 4-pole cascade + tanh saturation)
- [x] Shelf Low — `features/FILTER_06_SHELF_LOW.md` (added Filter::ShelfLow, reused BiquadCoeffs::low_shelf)
- [x] Shelf High — `features/FILTER_07_SHELF_HIGH.md` (added Filter::ShelfHigh, reused BiquadCoeffs::high_shelf)

## LFO targets (missing)

- [x] Pulse width — `features/LFO_01_PULSE_WIDTH.md` (added ModulationTarget::PulseWidth with synthesis validation)
- [x] FM index — `features/LFO_02_FM_INDEX.md` (added ModulationTarget::FmIndex with synthesis validation)
- [x] Grain size — `features/LFO_03_GRAIN_SIZE.md` (added ModulationTarget::GrainSize, per-grain modulation)
- [x] Grain density — `features/LFO_04_GRAIN_DENSITY.md` (added ModulationTarget::GrainDensity, per-grain modulation)
- [x] Delay time — `features/LFO_05_DELAY_TIME.md` (added post_fx_lfos field + ModulationTarget::DelayTime)
- [x] Reverb size — `features/LFO_06_REVERB_SIZE.md` (added ModulationTarget::ReverbSize, matches reverb)
- [x] Distortion drive — `features/LFO_07_DISTORTION_DRIVE.md` (added ModulationTarget::DistortionDrive, matches waveshaper)

## Effects (missing, Priority 1–3)

### Priority 1

- [x] Flanger — `features/EFFECT_P1_01_FLANGER.md` (added Effect::Flanger with delay_time LFO support)
- [x] Parametric EQ — `features/EFFECT_P1_02_PARAMETRIC_EQ.md` (added Effect::ParametricEq with EqBand/EqBandType)
- [x] Limiter — `features/EFFECT_P1_03_LIMITER.md` (added Effect::Limiter with lookahead)
- [x] Gate/Expander — `features/EFFECT_P1_04_GATE_EXPANDER.md` (added Effect::GateExpander with hold timer)
- [x] Stereo widener — `features/EFFECT_P1_05_STEREO_WIDENER.md` (added Effect::StereoWidener with 3 modes)

### Priority 2

- [x] Multi-tap delay — `features/EFFECT_P2_01_MULTI_TAP_DELAY.md` (added Effect::MultiTapDelay with DelayTap, shared delay_line.rs)
- [x] Tape saturation — `features/EFFECT_P2_02_TAPE_SATURATION.md` (added Effect::TapeSaturation with seeded hiss, distortion_drive LFO)
- [x] Transient shaper — `features/EFFECT_P2_03_TRANSIENT_SHAPER.md` (added Effect::TransientShaper with dual envelope)
- [x] Auto-filter / envelope follower — `features/EFFECT_P2_04_AUTO_FILTER.md` (added Effect::AutoFilter with envelope follower)
- [x] Cabinet simulation — `features/EFFECT_P2_05_CABINET_SIM.md` (added Effect::CabinetSim with 5 cabinet types)

### Priority 3

- [x] Rotary speaker — `features/EFFECT_P3_01_ROTARY_SPEAKER.md` (added Effect::RotarySpeaker with Doppler)
- [x] Ring modulator (effect) — `features/EFFECT_P3_02_RING_MODULATOR.md` (added Effect::RingModulator)
- [x] Granular delay — `features/EFFECT_P3_03_GRANULAR_DELAY.md` (added Effect::GranularDelay with seeded RNG, delay_time LFO)

## Synthesis types (missing, Priority 1–3)

### Priority 1

- [x] Supersaw/Unison engine — `features/SYNTH_P1_01_SUPERSAW_UNISON.md` (added Synthesis::SupersawUnison with DetuneCurve)
- [x] Waveguide synthesis — `features/SYNTH_P1_02_WAVEGUIDE.md` (added Synthesis::Waveguide with delay-line + filtered noise)
- [x] Bowed string synthesis — `features/SYNTH_P1_03_BOWED_STRING.md` (added Synthesis::BowedString with bidirectional delay + stick-slip friction)
- [x] Membrane/drum synthesis — `features/SYNTH_P1_04_MEMBRANE_DRUM.md` (added Synthesis::MembraneDrum with modal synthesis + Bessel ratios)

### Priority 2

- [x] Feedback FM — `features/SYNTH_P2_01_FEEDBACK_FM.md` (added Synthesis::FeedbackFm with self-modulating operator)
- [x] Comb filter synthesis — `features/SYNTH_P2_02_COMB_FILTER_SYNTH.md` (added Synthesis::CombFilterSynth with CombExcitation enum)

### Priority 3

- [x] Pulsar synthesis — `features/SYNTH_P3_01_PULSAR.md` (added Synthesis::Pulsar with grain trains + Hann window)
- [x] VOSIM — `features/SYNTH_P3_02_VOSIM.md` (added Synthesis::Vosim with squared-sine pulse trains)
- [x] Spectral synthesis — `features/SYNTH_P3_03_SPECTRAL.md` (added Synthesis::SpectralFreeze with FFT freeze + overlap-add)
