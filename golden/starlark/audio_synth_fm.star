# FM synthesis example - demonstrates fm_synth() stdlib function
#
# FM synthesis creates complex timbres by modulating the frequency of a carrier
# oscillator with a modulator oscillator. The modulation_index controls harmonic richness.

spec(
    asset_id = "stdlib-audio-fm-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/fm.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(440, 880, 5.0),
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.7
                )
            ]
        }
    }
)
