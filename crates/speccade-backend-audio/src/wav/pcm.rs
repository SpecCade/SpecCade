//! PCM data extraction and hashing utilities.

/// Extracts PCM data from a WAV file buffer.
///
/// Used for comparing WAV files by their audio content only.
///
/// # Arguments
/// * `wav_data` - Complete WAV file bytes
///
/// # Returns
/// PCM data if found, or None if the format is invalid
pub fn extract_pcm_data(wav_data: &[u8]) -> Option<&[u8]> {
    if wav_data.len() < 44 {
        return None;
    }

    // Verify RIFF header
    if &wav_data[0..4] != b"RIFF" || &wav_data[8..12] != b"WAVE" {
        return None;
    }

    // Find data chunk
    let mut pos = 12;
    while pos + 8 <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            wav_data[pos + 4],
            wav_data[pos + 5],
            wav_data[pos + 6],
            wav_data[pos + 7],
        ]) as usize;

        if chunk_id == b"data" {
            let data_start = pos + 8;
            let data_end = data_start + chunk_size;
            if data_end <= wav_data.len() {
                return Some(&wav_data[data_start..data_end]);
            }
        }

        pos += 8 + chunk_size;
        // Align to word boundary
        if !chunk_size.is_multiple_of(2) {
            pos += 1;
        }
    }

    None
}

/// Computes the PCM hash of a WAV file.
///
/// # Arguments
/// * `wav_data` - Complete WAV file bytes
///
/// # Returns
/// BLAKE3 hash of PCM data, or None if format is invalid
pub fn compute_pcm_hash(wav_data: &[u8]) -> Option<String> {
    extract_pcm_data(wav_data).map(|pcm| blake3::hash(pcm).to_hex().to_string())
}
