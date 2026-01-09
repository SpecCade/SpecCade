//! Simple test to verify instrument generation works.

use speccade_backend_audio::generate_instrument;
use speccade_spec::recipe::audio_instrument::{AudioInstrumentSynthPatchV1Params, NoteSpec};
use speccade_spec::recipe::audio_sfx::{Envelope, Synthesis, Waveform};

fn main() {
    println!("Testing instrument generation...");

    // Create a simple sine wave instrument at A4 (440 Hz)
    let params = AudioInstrumentSynthPatchV1Params {
        note_duration_seconds: 1.0,
        sample_rate: 44100,
        synthesis: Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0, // Will be overridden by note
            freq_sweep: None,
        },
        envelope: Envelope {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        },
        notes: Some(vec![NoteSpec::MidiNote(69)]), // A4
        generate_loop_points: true,
    };

    match generate_instrument(&params, 42) {
        Ok(result) => {
            println!("Success!");
            println!("  Notes generated: {:?}", result.notes);
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
