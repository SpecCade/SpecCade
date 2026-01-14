//! IT instrument and sample generation.

use std::path::Path;

use speccade_spec::recipe::music::{TrackerFormat, TrackerInstrument};

use crate::envelope::convert_envelope_to_it;
use crate::generate::{bake_instrument_sample, GenerateError, MusicInstrumentLoopReport};
use crate::it::{ItInstrument, ItSample};
use crate::note::calculate_c5_speed_for_base_note;

/// Generate an IT instrument and sample from spec.
///
/// IT format separates instruments from samples, so this returns both.
/// The instrument maps all notes to the generated sample.
pub fn generate_it_instrument(
    instr: &TrackerInstrument,
    base_seed: u32,
    index: u32,
    spec_dir: &Path,
) -> Result<(ItInstrument, ItSample, MusicInstrumentLoopReport), GenerateError> {
    let (baked, loop_report) =
        bake_instrument_sample(instr, base_seed, index, spec_dir, TrackerFormat::It)?;

    // IT samples store "C-5 speed" (playback rate for note C-5), not the sample's native rate.
    let c5_speed = calculate_c5_speed_for_base_note(baked.sample_rate, baked.base_midi);

    let mut sample = ItSample::new(&instr.name, baked.pcm16_mono, c5_speed);

    if let Some(loop_region) = baked.loop_region {
        let pingpong = loop_region.mode == crate::generate::LoopMode::PingPong;
        sample = sample.with_loop(loop_region.start, loop_region.end, pingpong);
    }

    // Set default volume
    sample.default_volume = instr.default_volume.unwrap_or(64).min(64);

    // Create instrument
    let mut it_instr = ItInstrument::new(&instr.name);

    // Map all notes to this sample (index + 1 since IT is 1-indexed)
    it_instr.map_all_to_sample((index + 1) as u8);

    // Convert envelope
    it_instr.volume_envelope = convert_envelope_to_it(&instr.envelope);

    Ok((it_instr, sample, loop_report))
}
