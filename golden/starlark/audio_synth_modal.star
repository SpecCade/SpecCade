# Modal synthesis example
#
# Modal synthesis simulates resonant bodies using multiple decaying modes.
# Great for bells, plates, and metallic sounds.
# Covers: modal(), modal_mode()

spec(
    asset_id = "stdlib-audio-modal-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/modal.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 3.0,
            "sample_rate": 44100,
            "layers": [
                # Bell-like sound with inharmonic modes
                audio_layer(
                    synthesis = modal(
                        440,
                        [
                            modal_mode(1.0, 1.0, 1.5),    # Fundamental
                            modal_mode(2.0, 0.6, 1.2),   # 2nd harmonic
                            modal_mode(2.76, 0.4, 1.0),  # Inharmonic partial
                            modal_mode(5.4, 0.25, 0.8),  # Higher inharmonic
                            modal_mode(8.93, 0.15, 0.5), # Bell-like shimmer
                        ],
                        "impulse"
                    ),
                    envelope = envelope(0.001, 0.1, 0.0, 2.0),
                    volume = 0.7
                )
            ]
        }
    }
)
