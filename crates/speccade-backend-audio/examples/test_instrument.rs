//! Simple test to verify instrument generation works.

use speccade_backend_audio::generate;
use speccade_spec::recipe::audio::{AudioV1Params, Envelope, NoteSpec, Synthesis, Waveform};
use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};

fn main() {
    println!("Testing instrument generation...");

    // Create a simple sine wave instrument at A4 (440 Hz)
    let params = AudioV1Params {
        duration_seconds: 1.0,
        sample_rate: 44100,
        layers: vec![speccade_spec::recipe::audio::AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0, // Will be overridden by note
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
            },
            volume: 1.0,
            pan: 0.0,
            delay: None,
            filter: None,
            lfo: None,
        }],
        master_filter: None,
        effects: vec![],
        pitch_envelope: None,
        base_note: Some(NoteSpec::MidiNote(69)), // A4
        loop_config: None,
        generate_loop_points: true,
        post_fx_lfos: vec![],
    };

    let spec = Spec::builder("test-instrument", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(
            OutputFormat::Wav,
            "instruments/test_instrument.wav",
        ))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::to_value(&params).unwrap(),
        ))
        .build();

    match generate(&spec) {
        Ok(result) => {
            println!("Success!");
            println!("  Base note: {:?}", result.base_note);
            println!("  Sample rate: {} Hz", result.wav.sample_rate);
            println!("  Number of samples: {}", result.wav.num_samples);
            println!("  Duration: {:.2} seconds", result.wav.duration_seconds());
            println!("  Loop point: {:?}", result.loop_point);
            println!("  PCM hash: {}", result.wav.pcm_hash);
            println!("  WAV size: {} bytes", result.wav.wav_data.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
