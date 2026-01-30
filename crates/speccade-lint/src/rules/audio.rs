//! Audio quality lint rules.
//!
//! Rules for detecting perceptual problems in generated audio assets.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;
use std::io::Cursor;

/// Parsed audio data for analysis.
struct AudioAnalysis {
    /// All samples normalized to [-1.0, 1.0] range.
    samples: Vec<f32>,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    channels: u16,
    /// Duration in seconds.
    duration: f32,
}

impl AudioAnalysis {
    /// Parses WAV data and returns an AudioAnalysis.
    fn from_wav(bytes: &[u8]) -> Option<Self> {
        let cursor = Cursor::new(bytes);
        let reader = hound::WavReader::new(cursor).ok()?;
        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels;
        let bits_per_sample = spec.bits_per_sample;
        let sample_format = spec.sample_format;

        let samples: Vec<f32> = match sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1i32 << (bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => reader
                .into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect(),
        };

        let total_samples = samples.len();
        let samples_per_channel = total_samples / channels as usize;
        let duration = samples_per_channel as f32 / sample_rate as f32;

        Some(AudioAnalysis {
            samples,
            sample_rate,
            channels,
            duration,
        })
    }

    /// Returns the peak amplitude (maximum absolute value).
    fn peak_amplitude(&self) -> f32 {
        self.samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, |a, b| a.max(b))
    }

    /// Returns RMS (root mean square) level.
    fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = self.samples.iter().map(|s| (*s as f64).powi(2)).sum();
        (sum_sq / self.samples.len() as f64).sqrt() as f32
    }

    /// Returns RMS level in decibels (dBFS).
    fn rms_db(&self) -> f32 {
        let rms = self.rms();
        if rms <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * rms.log10()
        }
    }

    /// Returns the mean (DC offset).
    fn mean(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().map(|s| *s as f64).sum();
        (sum / self.samples.len() as f64) as f32
    }

    /// Returns the number of samples exceeding +-1.0.
    fn clipping_count(&self) -> usize {
        self.samples.iter().filter(|s| s.abs() > 1.0).count()
    }

    /// Returns the amplitude of the final N milliseconds.
    fn final_amplitude(&self, ms: f32) -> f32 {
        let samples_needed = ((ms / 1000.0) * self.sample_rate as f32) as usize;
        let start = self.samples.len().saturating_sub(samples_needed);
        self.samples[start..]
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, |a, b| a.max(b))
    }

    /// Estimates the high-frequency energy ratio (above cutoff_hz).
    /// Uses zero-crossing rate as a proxy for frequency content.
    fn high_frequency_energy_ratio(&self, cutoff_hz: f32) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        // Calculate zero-crossing rate
        let mut zero_crossings = 0;
        for i in 1..self.samples.len() {
            if (self.samples[i - 1] >= 0.0) != (self.samples[i] >= 0.0) {
                zero_crossings += 1;
            }
        }

        // Zero-crossing rate approximates 2 * fundamental frequency for simple waveforms
        let zcr = zero_crossings as f32 / (self.samples.len() as f32 / self.sample_rate as f32);
        let estimated_freq = zcr / 2.0;

        // High-frequency energy is approximated by how much the ZCR exceeds the cutoff
        // normalized to give a 0-1 ratio estimate
        let nyquist = self.sample_rate as f32 / 2.0;
        let high_freq_factor = (estimated_freq - cutoff_hz).max(0.0) / (nyquist - cutoff_hz);
        high_freq_factor.clamp(0.0, 1.0)
    }

    /// Estimates low-mid frequency energy ratio (between low_hz and high_hz).
    /// Uses a simple band-energy approximation.
    fn low_mid_energy_ratio(&self, low_hz: f32, high_hz: f32) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        // Use a simple approach: calculate energy in different frequency bands
        // by analyzing sample differences (approximating high-pass filtering)
        let total_energy: f64 = self.samples.iter().map(|s| (*s as f64).powi(2)).sum();
        if total_energy <= 0.0 {
            return 0.0;
        }

        // Low-pass approximation: running average
        let window_size = (self.sample_rate as f32 / high_hz).max(1.0) as usize;
        let mut low_passed: Vec<f32> = Vec::with_capacity(self.samples.len());
        let mut sum = 0.0;
        for (i, &sample) in self.samples.iter().enumerate() {
            sum += sample;
            if i >= window_size {
                sum -= self.samples[i - window_size];
            }
            let count = (i + 1).min(window_size) as f32;
            low_passed.push(sum / count);
        }

        // High-pass: original - lowpass gives us high frequencies
        // For low-mid, we want what's left after removing very low frequencies
        let hp_window = (self.sample_rate as f32 / low_hz).max(1.0) as usize;
        let mut very_low: Vec<f32> = Vec::with_capacity(self.samples.len());
        sum = 0.0;
        for (i, &sample) in self.samples.iter().enumerate() {
            sum += sample;
            if i >= hp_window {
                sum -= self.samples[i - hp_window];
            }
            let count = (i + 1).min(hp_window) as f32;
            very_low.push(sum / count);
        }

        // Low-mid energy: low_passed - very_low
        let low_mid_energy: f64 = low_passed
            .iter()
            .zip(very_low.iter())
            .map(|(lp, vl)| ((*lp - *vl) as f64).powi(2))
            .sum();

        (low_mid_energy / total_energy) as f32
    }
}

// =============================================================================
// Error-level rules
// =============================================================================

/// Detects sample values exceeding +/-1.0 (clipping).
pub struct ClippingRule;

impl LintRule for ClippingRule {
    fn id(&self) -> &'static str {
        "audio/clipping"
    }

    fn description(&self) -> &'static str {
        "Sample values exceed +/-1.0, causing digital distortion"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        let clipping_count = analysis.clipping_count();
        if clipping_count == 0 {
            return vec![];
        }

        let peak = analysis.peak_amplitude();
        let fix_delta = 1.0 / peak as f64;

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio contains {} samples exceeding +/-1.0 (peak: {:.3})",
                clipping_count, peak
            ),
            "Reduce amplitude to prevent clipping",
        )
        .with_actual_value(format!("{:.3}", peak))
        .with_expected_range("[-1.0, 1.0]")
        .with_fix_delta(fix_delta)
        .with_fix_param("amplitude")]
    }
}

/// Detects non-zero average sample value (DC offset).
pub struct DcOffsetRule;

impl LintRule for DcOffsetRule {
    fn id(&self) -> &'static str {
        "audio/dc-offset"
    }

    fn description(&self) -> &'static str {
        "Non-zero average sample value causes headroom loss and speaker stress"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        let mean = analysis.mean();
        const DC_THRESHOLD: f32 = 0.01;

        if mean.abs() <= DC_THRESHOLD {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio has DC offset of {:.4} (threshold: +/-{})",
                mean, DC_THRESHOLD
            ),
            "Add a highpass filter to remove DC offset",
        )
        .with_actual_value(format!("{:.4}", mean))
        .with_expected_range(format!("[{}, {}]", -DC_THRESHOLD, DC_THRESHOLD))
        .with_fix_template("add highpass(cutoff=20)")]
    }
}

/// Detects entirely silent audio.
pub struct SilenceRule;

impl LintRule for SilenceRule {
    fn id(&self) -> &'static str {
        "audio/silence"
    }

    fn description(&self) -> &'static str {
        "Asset is entirely silent (RMS < -60dB)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        let peak = analysis.peak_amplitude();
        const SILENCE_THRESHOLD: f32 = 0.001;

        if peak >= SILENCE_THRESHOLD {
            return vec![];
        }

        let rms_db = analysis.rms_db();

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!("Audio is silent (peak: {:.6}, RMS: {:.1}dB)", peak, rms_db),
            "Check envelope attack or oscillator amplitude",
        )
        .with_actual_value(format!("{:.6}", peak))
        .with_expected_range(format!(">= {}", SILENCE_THRESHOLD))]
    }
}

// =============================================================================
// Warning-level rules
// =============================================================================

/// Detects very low loudness.
pub struct TooQuietRule;

impl LintRule for TooQuietRule {
    fn id(&self) -> &'static str {
        "audio/too-quiet"
    }

    fn description(&self) -> &'static str {
        "Audio loudness is very low"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        let rms = analysis.rms();
        let rms_db = analysis.rms_db();

        // Skip if completely silent (handled by silence rule)
        if rms < 0.001 {
            return vec![];
        }

        // Target: -18 dBFS is a typical reference level
        // Warn if below -30 dBFS
        const QUIET_THRESHOLD_DB: f32 = -30.0;
        const TARGET_DB: f32 = -18.0;

        if rms_db >= QUIET_THRESHOLD_DB {
            return vec![];
        }

        // Calculate multiplier to reach target
        let target_rms = 10.0f32.powf(TARGET_DB / 20.0);
        let fix_delta = (target_rms / rms) as f64;

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio is too quiet (RMS: {:.1}dB, threshold: {}dB)",
                rms_db, QUIET_THRESHOLD_DB
            ),
            format!(
                "Increase amplitude by ~{:.1}x to reach {}dB",
                fix_delta, TARGET_DB
            ),
        )
        .with_actual_value(format!("{:.1}dB", rms_db))
        .with_expected_range(format!(">= {}dB", QUIET_THRESHOLD_DB))
        .with_fix_delta(fix_delta)
        .with_fix_param("amplitude")]
    }
}

/// Detects near-clipping loudness.
pub struct TooLoudRule;

impl LintRule for TooLoudRule {
    fn id(&self) -> &'static str {
        "audio/too-loud"
    }

    fn description(&self) -> &'static str {
        "Audio is near clipping levels"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        let rms_db = analysis.rms_db();
        let peak = analysis.peak_amplitude();

        // -6dB RMS is very loud, likely to cause issues
        const LOUD_THRESHOLD_DB: f32 = -6.0;

        // Skip if already clipping (handled by clipping rule)
        if peak > 1.0 {
            return vec![];
        }

        if rms_db < LOUD_THRESHOLD_DB {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio is very loud (RMS: {:.1}dB, peak: {:.3})",
                rms_db, peak
            ),
            "Add a limiter to prevent potential clipping",
        )
        .with_actual_value(format!("{:.1}dB", rms_db))
        .with_expected_range(format!("< {}dB", LOUD_THRESHOLD_DB))
        .with_fix_template("add limiter()")]
    }
}

/// Detects excessive high-frequency energy.
pub struct HarshHighsRule;

impl LintRule for HarshHighsRule {
    fn id(&self) -> &'static str {
        "audio/harsh-highs"
    }

    fn description(&self) -> &'static str {
        "Excessive high-frequency energy above 8kHz"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        // Only relevant for sample rates that can represent 8kHz+
        if analysis.sample_rate < 16000 {
            return vec![];
        }

        let high_ratio = analysis.high_frequency_energy_ratio(8000.0);

        // More than 50% energy above 8kHz is harsh
        const HARSH_THRESHOLD: f32 = 0.5;

        if high_ratio < HARSH_THRESHOLD {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio has excessive high-frequency energy ({:.0}% above 8kHz)",
                high_ratio * 100.0
            ),
            "Apply a lowpass filter to reduce harshness",
        )
        .with_actual_value(format!("{:.0}%", high_ratio * 100.0))
        .with_expected_range(format!("< {}%", HARSH_THRESHOLD * 100.0))
        .with_fix_template("lowpass(cutoff=6000)")]
    }
}

/// Detects excessive low-mid frequency buildup.
pub struct MuddyLowsRule;

impl LintRule for MuddyLowsRule {
    fn id(&self) -> &'static str {
        "audio/muddy-lows"
    }

    fn description(&self) -> &'static str {
        "Excessive low-mid frequency energy (200-500Hz)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        // Only relevant for sample rates that can represent the frequency range
        if analysis.sample_rate < 1000 {
            return vec![];
        }

        let low_mid_ratio = analysis.low_mid_energy_ratio(200.0, 500.0);

        // More than 60% energy in 200-500Hz is muddy
        const MUDDY_THRESHOLD: f32 = 0.6;

        if low_mid_ratio < MUDDY_THRESHOLD {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio has excessive low-mid energy ({:.0}% in 200-500Hz)",
                low_mid_ratio * 100.0
            ),
            "Apply a highpass filter to reduce muddiness",
        )
        .with_actual_value(format!("{:.0}%", low_mid_ratio * 100.0))
        .with_expected_range(format!("< {}%", MUDDY_THRESHOLD * 100.0))
        .with_fix_template("highpass(cutoff=80)")]
    }
}

/// Detects abrupt endings without fade-out.
pub struct AbruptEndRule;

impl LintRule for AbruptEndRule {
    fn id(&self) -> &'static str {
        "audio/abrupt-end"
    }

    fn description(&self) -> &'static str {
        "Audio ends abruptly without fade-out"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        // Check final 10ms amplitude
        let final_amp = analysis.final_amplitude(10.0);
        const ABRUPT_THRESHOLD: f32 = 0.1;

        if final_amp < ABRUPT_THRESHOLD {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Audio ends abruptly (final 10ms amplitude: {:.3})",
                final_amp
            ),
            "Increase envelope release for a smoother ending",
        )
        .with_actual_value(format!("{:.3}", final_amp))
        .with_expected_range(format!("< {}", ABRUPT_THRESHOLD))]
    }
}

// =============================================================================
// Info-level rules
// =============================================================================

/// Detects dry signals with no spatial processing.
pub struct NoEffectsRule;

impl LintRule for NoEffectsRule {
    fn id(&self) -> &'static str {
        "audio/no-effects"
    }

    fn description(&self) -> &'static str {
        "Audio has no spatial effects (reverb, delay)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        // This rule requires spec context to check effects chain
        let Some(spec) = spec else {
            return vec![];
        };

        // Parse the spec to check for effects
        let has_effects = Self::spec_has_effects(spec);

        if has_effects {
            return vec![];
        }

        // Only check if we could parse the WAV (to confirm it's valid audio)
        if AudioAnalysis::from_wav(asset.bytes).is_none() {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            "Audio has no spatial effects applied",
            "Consider adding reverb for more natural sound",
        )
        .with_fix_template("reverb()")]
    }
}

impl NoEffectsRule {
    /// Checks if the spec contains any audio effects.
    fn spec_has_effects(spec: &Spec) -> bool {
        // Check the recipe for effects
        if let Some(recipe) = &spec.recipe {
            // Check for params.effects pattern in the recipe params JSON
            if let Some(effects) = recipe.params.get("effects") {
                if let Some(arr) = effects.as_array() {
                    if !arr.is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Detects stereo files for short SFX where mono would suffice.
pub struct MonoRecommendedRule;

impl LintRule for MonoRecommendedRule {
    fn id(&self) -> &'static str {
        "audio/mono-recommended"
    }

    fn description(&self) -> &'static str {
        "Short sound effect uses stereo when mono would suffice"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Audio]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, asset: &AssetData, _spec: Option<&Spec>) -> Vec<LintIssue> {
        let Some(analysis) = AudioAnalysis::from_wav(asset.bytes) else {
            return vec![];
        };

        // Only flag stereo files under 2 seconds
        const MAX_DURATION_FOR_MONO: f32 = 2.0;

        if analysis.channels < 2 || analysis.duration >= MAX_DURATION_FOR_MONO {
            return vec![];
        }

        vec![LintIssue::new(
            self.id(),
            self.default_severity(),
            format!(
                "Short audio ({:.2}s) uses stereo - mono would reduce file size",
                analysis.duration
            ),
            "Use mono output format for short sound effects",
        )
        .with_actual_value(format!(
            "{} channels, {:.2}s",
            analysis.channels, analysis.duration
        ))
        .with_expected_range(format!("mono for duration < {}s", MAX_DURATION_FOR_MONO))]
    }
}

/// Returns all audio lint rules.
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        // Error-level rules
        Box::new(ClippingRule),
        Box::new(DcOffsetRule),
        Box::new(SilenceRule),
        // Warning-level rules
        Box::new(TooQuietRule),
        Box::new(TooLoudRule),
        Box::new(HarshHighsRule),
        Box::new(MuddyLowsRule),
        Box::new(AbruptEndRule),
        // Info-level rules
        Box::new(NoEffectsRule),
        Box::new(MonoRecommendedRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;

    /// Creates a WAV file in memory with the given samples.
    fn create_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let spec = hound::WavSpec {
                channels,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };
            let mut writer = hound::WavWriter::new(cursor, spec).unwrap();
            for &sample in samples {
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
        }
        buffer
    }

    fn asset_data(bytes: &[u8]) -> AssetData<'_> {
        AssetData {
            path: Path::new("test.wav"),
            bytes,
        }
    }

    #[test]
    fn test_clipping_detection() {
        let rule = ClippingRule;

        // Non-clipping audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0).sin() * 0.9).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect clipping in normal audio"
        );

        // Clipping audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 1000.0).sin() * 1.5).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect clipping");
        assert_eq!(issues[0].rule_id, "audio/clipping");
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].fix_delta.is_some());
        assert_eq!(issues[0].fix_param, Some("amplitude".to_string()));
    }

    #[test]
    fn test_dc_offset_detection() {
        let rule = DcOffsetRule;

        // No DC offset - use exact number of complete sine cycles
        // Period = 2*PI, so 1000 cycles ensures the mean is effectively zero
        let samples: Vec<f32> = (0..10000)
            .map(|i| {
                let phase = i as f32 * 2.0 * std::f32::consts::PI / 100.0;
                phase.sin() * 0.5
            })
            .collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect DC offset in centered audio"
        );

        // With DC offset
        let samples: Vec<f32> = (0..10000)
            .map(|i| {
                let phase = i as f32 * 2.0 * std::f32::consts::PI / 100.0;
                phase.sin() * 0.5 + 0.3
            })
            .collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect DC offset");
        assert_eq!(issues[0].rule_id, "audio/dc-offset");
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(
            issues[0].fix_template,
            Some("add highpass(cutoff=20)".to_string())
        );
    }

    #[test]
    fn test_silence_detection() {
        let rule = SilenceRule;

        // Non-silent audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect silence in normal audio"
        );

        // Silent audio
        let samples: Vec<f32> = vec![0.0; 1000];
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect silence");
        assert_eq!(issues[0].rule_id, "audio/silence");
        assert_eq!(issues[0].severity, Severity::Error);

        // Nearly silent audio
        let samples: Vec<f32> = vec![0.0001; 1000];
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect near-silence");
    }

    #[test]
    fn test_too_quiet_detection() {
        let rule = TooQuietRule;

        // Normal volume audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect too-quiet in normal audio"
        );

        // Very quiet audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.01).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect too-quiet audio");
        assert_eq!(issues[0].rule_id, "audio/too-quiet");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].fix_delta.is_some());
    }

    #[test]
    fn test_too_loud_detection() {
        let rule = TooLoudRule;

        // Normal volume audio
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.3).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect too-loud in normal audio"
        );

        // Very loud audio (near clipping but not over)
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.95).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect too-loud audio");
        assert_eq!(issues[0].rule_id, "audio/too-loud");
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].fix_template, Some("add limiter()".to_string()));
    }

    #[test]
    fn test_abrupt_end_detection() {
        let rule = AbruptEndRule;

        // Audio with fade-out (ends quietly) - longer audio to ensure fade works
        // At 44100 Hz, 10ms = 441 samples. Use 4410 samples (100ms) with exponential fade
        let samples: Vec<f32> = (0..4410)
            .map(|i| {
                let t = i as f32 / 4410.0;
                // Exponential fade that reaches near-zero at the end
                let envelope = (-t * 5.0).exp(); // Drops to ~0.007 at t=1.0
                let phase = t * 440.0 * 2.0 * std::f32::consts::PI;
                phase.sin() * envelope * 0.5
            })
            .collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(
            issues.is_empty(),
            "Should not detect abrupt end in faded audio"
        );

        // Audio with abrupt end - constant amplitude sine wave
        let samples: Vec<f32> = (0..4410)
            .map(|i| {
                let t = i as f32 / 4410.0;
                let phase = t * 440.0 * 2.0 * std::f32::consts::PI;
                phase.sin() * 0.5
            })
            .collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should detect abrupt end");
        assert_eq!(issues[0].rule_id, "audio/abrupt-end");
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_mono_recommended() {
        let rule = MonoRecommendedRule;

        // Mono audio - no issue
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(issues.is_empty(), "Should not flag mono audio");

        // Short stereo audio - should flag
        let samples: Vec<f32> = (0..2000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect(); // Interleaved L/R
        let wav = create_wav(&samples, 44100, 2);
        let issues = rule.check(&asset_data(&wav), None);
        assert_eq!(issues.len(), 1, "Should flag short stereo audio");
        assert_eq!(issues[0].rule_id, "audio/mono-recommended");
        assert_eq!(issues[0].severity, Severity::Info);

        // Long stereo audio - no issue
        let samples: Vec<f32> = (0..200000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect();
        let wav = create_wav(&samples, 44100, 2);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(issues.is_empty(), "Should not flag long stereo audio");
    }

    #[test]
    fn test_no_effects_without_spec() {
        let rule = NoEffectsRule;

        // Without spec context, rule should not fire
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 50.0).sin() * 0.5).collect();
        let wav = create_wav(&samples, 44100, 1);
        let issues = rule.check(&asset_data(&wav), None);
        assert!(issues.is_empty(), "Should not fire without spec context");
    }

    #[test]
    fn test_all_rules_registered() {
        let rules = all_rules();
        assert_eq!(rules.len(), 10, "Should have 10 audio rules");

        // Verify rule IDs
        let ids: Vec<&str> = rules.iter().map(|r| r.id()).collect();
        assert!(ids.contains(&"audio/clipping"));
        assert!(ids.contains(&"audio/dc-offset"));
        assert!(ids.contains(&"audio/silence"));
        assert!(ids.contains(&"audio/too-quiet"));
        assert!(ids.contains(&"audio/too-loud"));
        assert!(ids.contains(&"audio/harsh-highs"));
        assert!(ids.contains(&"audio/muddy-lows"));
        assert!(ids.contains(&"audio/abrupt-end"));
        assert!(ids.contains(&"audio/no-effects"));
        assert!(ids.contains(&"audio/mono-recommended"));
    }

    #[test]
    fn test_severity_levels() {
        let rules = all_rules();

        let error_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.default_severity() == Severity::Error)
            .collect();
        assert_eq!(error_rules.len(), 3, "Should have 3 error-level rules");

        let warning_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.default_severity() == Severity::Warning)
            .collect();
        assert_eq!(warning_rules.len(), 5, "Should have 5 warning-level rules");

        let info_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.default_severity() == Severity::Info)
            .collect();
        assert_eq!(info_rules.len(), 2, "Should have 2 info-level rules");
    }
}
