//! Preset factory functions for vector synthesis.

use super::synth::VectorSynth;
use super::types::{VectorPath, VectorPathPoint, VectorPosition, VectorSource};

/// Creates an evolving pad preset.
///
/// A lush, slowly evolving pad sound that morphs between different timbres.
/// Uses a path that slowly moves around the vector space.
pub fn evolving_pad(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),     // A: Pure sine (fundamental)
        VectorSource::saw(1.0),      // B: Saw wave (bright)
        VectorSource::triangle(2.0), // C: Triangle (soft, octave up)
        VectorSource::wavetable(1.0), // D: Wavetable (complex)
    ];

    // Create a path that moves in a circle around the center
    let segment_duration = duration / 4.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.3, 0.3), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.7, 0.3), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.7, 0.7), segment_duration),
        VectorPathPoint::new(VectorPosition::new(0.3, 0.7), segment_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::new(0.3, 0.3))
        .with_path(path, true)
}

/// Creates a morphing texture preset.
///
/// A textured sound that morphs between noise and tonal sources,
/// creating interesting evolving textures.
pub fn morph_texture(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),       // A: Pure tone
        VectorSource::noise(),         // B: Noise
        VectorSource::square(0.5),     // C: Sub-octave square
        VectorSource::wavetable(2.0),  // D: Wavetable at octave
    ];

    // Path that sweeps from tonal to noisy and back
    let half_duration = duration / 2.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::new(0.0, 0.5), half_duration),
        VectorPathPoint::new(VectorPosition::new(1.0, 0.5), half_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::new(0.0, 0.5))
        .with_path(path, true)
}

/// Creates a sweep corners preset.
///
/// Demonstrates vector synthesis by sweeping through all four corners
/// in sequence, showcasing each source distinctly.
pub fn sweep_corners(frequency: f64, duration: f64) -> VectorSynth {
    let sources = [
        VectorSource::sine(1.0),      // A: Sine
        VectorSource::saw(1.0),       // B: Saw
        VectorSource::triangle(1.0),  // C: Triangle
        VectorSource::square(1.0),    // D: Square
    ];

    // Path that visits each corner
    let segment_duration = duration / 4.0;
    let path = VectorPath::new(vec![
        VectorPathPoint::new(VectorPosition::corner_a(), 0.0), // Start at A
        VectorPathPoint::new(VectorPosition::corner_b(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_d(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_c(), segment_duration),
        VectorPathPoint::new(VectorPosition::corner_a(), segment_duration),
    ]);

    VectorSynth::new(frequency, sources)
        .with_position(VectorPosition::corner_a())
        .with_path(path, true)
}
