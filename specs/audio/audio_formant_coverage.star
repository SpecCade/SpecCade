# Golden test: Formant/vocal synthesis enum coverage
# Covers uncovered enum values for formant/vocal synthesis:
#   - vowel::a, vowel::i, vowel::u, vowel::e, vowel::o (for formant_filter and formant_synth)
#   - vowel_morph::a, vowel_morph::i, vowel_morph::u, vowel_morph::e, vowel_morph::o (for formant_synth)
#   - source_type::noise, source_type::tone, source_type::formant (for granular_source)
#   - excitation::impulse, excitation::noise, excitation::pluck (for modal)
#   - band_spacing::linear, band_spacing::logarithmic (for vocoder)

# ============================================================================
# Section 1: formant_filter with all vowel values
# ============================================================================

formant_filter_a = audio_layer(
    synthesis = oscillator(150.0, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.2),
    volume = 0.5,
    filter = formant_filter("a", 0.8)
)

formant_filter_i = audio_layer(
    synthesis = oscillator(150.0, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.2),
    volume = 0.5,
    filter = formant_filter("i", 0.8)
)

formant_filter_u = audio_layer(
    synthesis = oscillator(150.0, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.2),
    volume = 0.5,
    filter = formant_filter("u", 0.8)
)

formant_filter_e = audio_layer(
    synthesis = oscillator(150.0, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.2),
    volume = 0.5,
    filter = formant_filter("e", 0.8)
)

formant_filter_o = audio_layer(
    synthesis = oscillator(150.0, "sawtooth"),
    envelope = envelope(0.01, 0.2, 0.6, 0.2),
    volume = 0.5,
    filter = formant_filter("o", 0.8)
)

# ============================================================================
# Section 2: formant_synth with all vowel values
# ============================================================================

formant_synth_a = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "a", breathiness = 0.1),
    envelope = envelope(0.02, 0.1, 0.7, 0.2),
    volume = 0.5
)

formant_synth_i = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "i", breathiness = 0.1),
    envelope = envelope(0.02, 0.1, 0.7, 0.2),
    volume = 0.5
)

formant_synth_u = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "u", breathiness = 0.1),
    envelope = envelope(0.02, 0.1, 0.7, 0.2),
    volume = 0.5
)

formant_synth_e = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "e", breathiness = 0.1),
    envelope = envelope(0.02, 0.1, 0.7, 0.2),
    volume = 0.5
)

formant_synth_o = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "o", breathiness = 0.1),
    envelope = envelope(0.02, 0.1, 0.7, 0.2),
    volume = 0.5
)

# ============================================================================
# Section 3: formant_synth with all vowel_morph target values
# ============================================================================

formant_morph_to_a = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "i", vowel_morph = "a", morph_amount = 0.5, breathiness = 0.1),
    envelope = envelope(0.02, 0.15, 0.7, 0.2),
    volume = 0.5
)

formant_morph_to_i = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "a", vowel_morph = "i", morph_amount = 0.5, breathiness = 0.1),
    envelope = envelope(0.02, 0.15, 0.7, 0.2),
    volume = 0.5
)

formant_morph_to_u = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "a", vowel_morph = "u", morph_amount = 0.5, breathiness = 0.1),
    envelope = envelope(0.02, 0.15, 0.7, 0.2),
    volume = 0.5
)

formant_morph_to_e = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "a", vowel_morph = "e", morph_amount = 0.5, breathiness = 0.1),
    envelope = envelope(0.02, 0.15, 0.7, 0.2),
    volume = 0.5
)

formant_morph_to_o = audio_layer(
    synthesis = formant_synth(frequency = 220.0, vowel = "a", vowel_morph = "o", morph_amount = 0.5, breathiness = 0.1),
    envelope = envelope(0.02, 0.15, 0.7, 0.2),
    volume = 0.5
)

# ============================================================================
# Section 4: granular_source with all source_type values
# ============================================================================

granular_noise_source = audio_layer(
    synthesis = granular(
        source = granular_source("noise", "white"),
        grain_size_ms = 50.0,
        grain_density = 20.0
    ),
    envelope = envelope(0.05, 0.2, 0.5, 0.3),
    volume = 0.5
)

granular_tone_source = audio_layer(
    synthesis = granular(
        source = granular_source("tone", "sine", 440.0),
        grain_size_ms = 75.0,
        grain_density = 25.0,
        pitch_spread = 1.0
    ),
    envelope = envelope(0.02, 0.3, 0.6, 0.25),
    volume = 0.5
)

granular_formant_source = audio_layer(
    synthesis = granular(
        source = granular_source("formant", 220.0, 880.0),
        grain_size_ms = 60.0,
        grain_density = 22.0
    ),
    envelope = envelope(0.01, 0.25, 0.55, 0.3),
    volume = 0.5
)

# ============================================================================
# Section 5: modal with all excitation values
# ============================================================================

modal_impulse = audio_layer(
    synthesis = modal(
        440.0,
        [
            modal_mode(1.0, 1.0, 1.5),
            modal_mode(2.0, 0.6, 1.2),
            modal_mode(2.76, 0.4, 1.0),
        ],
        "impulse"
    ),
    envelope = envelope(0.001, 0.1, 0.0, 2.0),
    volume = 0.5
)

modal_noise = audio_layer(
    synthesis = modal(
        440.0,
        [
            modal_mode(1.0, 1.0, 1.5),
            modal_mode(2.0, 0.6, 1.2),
            modal_mode(2.76, 0.4, 1.0),
        ],
        "noise"
    ),
    envelope = envelope(0.001, 0.1, 0.0, 2.0),
    volume = 0.5
)

modal_pluck = audio_layer(
    synthesis = modal(
        440.0,
        [
            modal_mode(1.0, 1.0, 1.5),
            modal_mode(2.0, 0.6, 1.2),
            modal_mode(2.76, 0.4, 1.0),
        ],
        "pluck"
    ),
    envelope = envelope(0.001, 0.1, 0.0, 2.0),
    volume = 0.5
)

# ============================================================================
# Section 6: vocoder with all band_spacing values
# ============================================================================

vocoder_linear = audio_layer(
    synthesis = vocoder(
        carrier_freq = 110.0,
        carrier_type = "sawtooth",
        num_bands = 16,
        band_spacing = "linear",
        envelope_attack = 0.01,
        envelope_release = 0.05
    ),
    envelope = envelope(0.01, 0.1, 0.7, 0.2),
    volume = 0.5
)

vocoder_logarithmic = audio_layer(
    synthesis = vocoder(
        carrier_freq = 110.0,
        carrier_type = "sawtooth",
        num_bands = 16,
        band_spacing = "logarithmic",
        envelope_attack = 0.01,
        envelope_release = 0.05
    ),
    envelope = envelope(0.01, 0.1, 0.7, 0.2),
    volume = 0.5
)

# ============================================================================
# Spec combining all layers
# ============================================================================

spec(
    asset_id = "stdlib-audio-formant-coverage-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/formant_coverage.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                # formant_filter vowels
                formant_filter_a,
                formant_filter_i,
                formant_filter_u,
                formant_filter_e,
                formant_filter_o,
                # formant_synth vowels
                formant_synth_a,
                formant_synth_i,
                formant_synth_u,
                formant_synth_e,
                formant_synth_o,
                # formant_synth morph targets
                formant_morph_to_a,
                formant_morph_to_i,
                formant_morph_to_u,
                formant_morph_to_e,
                formant_morph_to_o,
                # granular sources
                granular_noise_source,
                granular_tone_source,
                granular_formant_source,
                # modal excitation types
                modal_impulse,
                modal_noise,
                modal_pluck,
                # vocoder band spacing
                vocoder_linear,
                vocoder_logarithmic,
            ]
        }
    }
)
