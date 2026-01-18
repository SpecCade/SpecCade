//! GLB/glTF accessor reading utilities.
//!
//! This module provides functions for reading vertex data from glTF accessors.

/// Extract the binary chunk from a GLB file.
pub fn extract_glb_binary_chunk(b: &[u8]) -> Option<&[u8]> {
    // GLB format: first 12 bytes are header, then chunks
    if b.len() > 12 {
        let json_chunk_length = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let bin_chunk_start = 12 + 8 + json_chunk_length;
        if b.len() > bin_chunk_start + 8 {
            let bin_chunk_length = u32::from_le_bytes([
                b[bin_chunk_start],
                b[bin_chunk_start + 1],
                b[bin_chunk_start + 2],
                b[bin_chunk_start + 3],
            ]) as usize;
            let bin_data_start = bin_chunk_start + 8;
            if b.len() >= bin_data_start + bin_chunk_length {
                return Some(&b[bin_data_start..bin_data_start + bin_chunk_length]);
            }
        }
    }
    None
}

/// Read vertex positions from an accessor.
pub fn read_positions(accessor: &gltf::Accessor, data: &[u8], positions: &mut Vec<[f32; 3]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(12); // 3 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 12 <= data.len() {
            let x = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let y = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            let z = f32::from_le_bytes([
                data[start + 8],
                data[start + 9],
                data[start + 10],
                data[start + 11],
            ]);
            positions.push([x, y, z]);
        }
    }
}

/// Read indices from an accessor.
pub fn read_indices(accessor: &gltf::Accessor, data: &[u8], indices: &mut Vec<u32>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let component_size = match accessor.data_type() {
        gltf::accessor::DataType::U8 => 1,
        gltf::accessor::DataType::U16 => 2,
        gltf::accessor::DataType::U32 => 4,
        _ => return,
    };
    let stride = view.stride().unwrap_or(component_size);

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        let index = match accessor.data_type() {
            gltf::accessor::DataType::U8 => {
                if start < data.len() {
                    data[start] as u32
                } else {
                    continue;
                }
            }
            gltf::accessor::DataType::U16 => {
                if start + 2 <= data.len() {
                    u16::from_le_bytes([data[start], data[start + 1]]) as u32
                } else {
                    continue;
                }
            }
            gltf::accessor::DataType::U32 => {
                if start + 4 <= data.len() {
                    u32::from_le_bytes([
                        data[start],
                        data[start + 1],
                        data[start + 2],
                        data[start + 3],
                    ])
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        indices.push(index);
    }
}

/// Read UV coordinates from an accessor.
pub fn read_uvs(accessor: &gltf::Accessor, data: &[u8], uvs: &mut Vec<[f32; 2]>) {
    let view = match accessor.view() {
        Some(v) => v,
        None => return,
    };

    let offset = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(8); // 2 * sizeof(f32)

    for i in 0..accessor.count() {
        let start = offset + i * stride;
        if start + 8 <= data.len() {
            let u = f32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            let v = f32::from_le_bytes([
                data[start + 4],
                data[start + 5],
                data[start + 6],
                data[start + 7],
            ]);
            uvs.push([u, v]);
        }
    }
}
