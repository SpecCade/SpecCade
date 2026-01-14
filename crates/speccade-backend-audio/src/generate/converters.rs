//! Type conversion utilities between spec types and internal representations.

use speccade_spec::recipe::audio::{
    FormantVowel, ModalExcitation, NoiseType, PdWaveform, SweepCurve, VectorSourceType,
    VocoderBandSpacing, VocoderCarrierType,
};

use crate::synthesis::formant::VowelPreset as FormantVowelPresetImpl;
use crate::synthesis::modal::Excitation as ModalExcitationImpl;
use crate::synthesis::noise::NoiseColor;
use crate::synthesis::phase_distortion::PdWaveform as PdWaveformImpl;
use crate::synthesis::vector::{
    VectorSource as VectorSourceImpl, VectorSourceType as VectorSourceTypeImpl,
};
use crate::synthesis::vocoder::{
    BandSpacing as VocoderBandSpacingImpl, CarrierType as VocoderCarrierTypeImpl,
};

/// Converts spec sweep curve to internal representation.
pub fn convert_sweep_curve(curve: &SweepCurve) -> crate::synthesis::SweepCurve {
    match curve {
        SweepCurve::Linear => crate::synthesis::SweepCurve::Linear,
        SweepCurve::Exponential => crate::synthesis::SweepCurve::Exponential,
        SweepCurve::Logarithmic => crate::synthesis::SweepCurve::Logarithmic,
    }
}

/// Converts spec noise type to internal representation.
pub fn convert_noise_type(noise_type: &NoiseType) -> NoiseColor {
    match noise_type {
        NoiseType::White => NoiseColor::White,
        NoiseType::Pink => NoiseColor::Pink,
        NoiseType::Brown => NoiseColor::Brown,
    }
}

/// Converts spec PD waveform to internal representation.
pub fn convert_pd_waveform(waveform: &PdWaveform) -> PdWaveformImpl {
    match waveform {
        PdWaveform::Resonant => PdWaveformImpl::Resonant,
        PdWaveform::Sawtooth => PdWaveformImpl::Sawtooth,
        PdWaveform::Pulse => PdWaveformImpl::Pulse,
    }
}

/// Converts spec modal excitation to internal representation.
pub fn convert_modal_excitation(excitation: &ModalExcitation) -> ModalExcitationImpl {
    match excitation {
        ModalExcitation::Impulse => ModalExcitationImpl::Impulse,
        ModalExcitation::Noise => ModalExcitationImpl::Noise,
        ModalExcitation::Pluck => ModalExcitationImpl::Pluck,
    }
}

/// Converts spec vocoder carrier type to internal representation.
pub fn convert_vocoder_carrier_type(carrier_type: &VocoderCarrierType) -> VocoderCarrierTypeImpl {
    match carrier_type {
        VocoderCarrierType::Sawtooth => VocoderCarrierTypeImpl::Sawtooth,
        VocoderCarrierType::Pulse => VocoderCarrierTypeImpl::Pulse,
        VocoderCarrierType::Noise => VocoderCarrierTypeImpl::Noise,
    }
}

/// Converts spec vocoder band spacing to internal representation.
pub fn convert_vocoder_band_spacing(band_spacing: &VocoderBandSpacing) -> VocoderBandSpacingImpl {
    match band_spacing {
        VocoderBandSpacing::Linear => VocoderBandSpacingImpl::Linear,
        VocoderBandSpacing::Logarithmic => VocoderBandSpacingImpl::Logarithmic,
    }
}

/// Converts spec formant vowel to internal representation.
pub fn convert_formant_vowel(vowel: &FormantVowel) -> FormantVowelPresetImpl {
    match vowel {
        FormantVowel::A => FormantVowelPresetImpl::A,
        FormantVowel::I => FormantVowelPresetImpl::I,
        FormantVowel::U => FormantVowelPresetImpl::U,
        FormantVowel::E => FormantVowelPresetImpl::E,
        FormantVowel::O => FormantVowelPresetImpl::O,
    }
}

/// Converts spec vector source type to internal representation.
pub fn convert_vector_source_type(source_type: &VectorSourceType) -> VectorSourceTypeImpl {
    match source_type {
        VectorSourceType::Sine => VectorSourceTypeImpl::Sine,
        VectorSourceType::Saw => VectorSourceTypeImpl::Saw,
        VectorSourceType::Square => VectorSourceTypeImpl::Square,
        VectorSourceType::Triangle => VectorSourceTypeImpl::Triangle,
        VectorSourceType::Noise => VectorSourceTypeImpl::Noise,
        VectorSourceType::Wavetable => VectorSourceTypeImpl::Wavetable,
    }
}

/// Converts spec vector source to internal representation.
pub fn convert_vector_source(source: &speccade_spec::recipe::audio::VectorSource) -> VectorSourceImpl {
    VectorSourceImpl::new(
        convert_vector_source_type(&source.source_type),
        source.frequency_ratio,
    )
}
