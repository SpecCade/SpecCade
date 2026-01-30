# Golden test: Physical modeling synthesis
# Tests: feedback_fm, pd_synth, metallic, comb_filter_synth

feedback_fm_layer = audio_layer(
    synthesis = feedback_fm(frequency = 220.0, feedback = 0.3, modulation_index = 2.0),
    envelope = envelope(attack = 0.01, decay = 0.2, sustain = 0.6, release = 0.3),
    volume = 0.7
)

feedback_fm_sweep_layer = audio_layer(
    synthesis = feedback_fm(frequency = 440.0, feedback = 0.5, modulation_index = 1.5, sweep_to = 220.0),
    envelope = envelope(attack = 0.005, decay = 0.15, sustain = 0.5, release = 0.25),
    volume = 0.6
)

pd_layer = audio_layer(
    synthesis = pd_synth(frequency = 330.0, distortion = 0.5, distortion_decay = 0.3, waveform = "resonant"),
    envelope = envelope(attack = 0.01, decay = 0.3, sustain = 0.5, release = 0.2),
    volume = 0.65
)

pd_saw_layer = audio_layer(
    synthesis = pd_synth(frequency = 200.0, distortion = 0.8, distortion_decay = 0.5, waveform = "sawtooth"),
    envelope = envelope(attack = 0.02, decay = 0.25, sustain = 0.4, release = 0.3),
    volume = 0.7
)

pd_pulse_layer = audio_layer(
    synthesis = pd_synth(frequency = 150.0, distortion = 0.6, distortion_decay = 0.4, waveform = "pulse", sweep_to = 100.0),
    envelope = envelope(attack = 0.01, decay = 0.2, sustain = 0.3, release = 0.4),
    volume = 0.55
)

metallic_layer = audio_layer(
    synthesis = metallic(base_freq = 200.0, num_partials = 8, inharmonicity = 1.05),
    envelope = envelope(attack = 0.001, decay = 0.5, sustain = 0.0, release = 0.1),
    volume = 0.8
)

metallic_bell_layer = audio_layer(
    synthesis = metallic(base_freq = 440.0, num_partials = 12, inharmonicity = 2.0),
    envelope = envelope(attack = 0.002, decay = 0.8, sustain = 0.1, release = 0.3),
    volume = 0.75
)

comb_synth_layer = audio_layer(
    synthesis = comb_filter_synth(frequency = 110.0, decay = 0.95, excitation = "impulse"),
    envelope = envelope(attack = 0.001, decay = 0.3, sustain = 0.2, release = 0.5),
    volume = 0.7
)

comb_noise_layer = audio_layer(
    synthesis = comb_filter_synth(frequency = 165.0, decay = 0.85, excitation = "noise"),
    envelope = envelope(attack = 0.002, decay = 0.4, sustain = 0.3, release = 0.4),
    volume = 0.65
)

comb_saw_layer = audio_layer(
    synthesis = comb_filter_synth(frequency = 220.0, decay = 0.9, excitation = "saw"),
    envelope = envelope(attack = 0.001, decay = 0.35, sustain = 0.25, release = 0.45),
    volume = 0.6
)

spec(
    asset_id = "stdlib-audio-physical-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/physical.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                feedback_fm_layer,
                feedback_fm_sweep_layer,
                pd_layer,
                pd_saw_layer,
                pd_pulse_layer,
                metallic_layer,
                metallic_bell_layer,
                comb_synth_layer,
                comb_noise_layer,
                comb_saw_layer,
            ]
        }
    }
)
