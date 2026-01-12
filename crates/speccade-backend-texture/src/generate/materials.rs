//! Material pattern application for texture generation.
//!
//! This module handles applying material-specific base patterns to height maps
//! based on the material type (brick, wood, metal, etc.).

use speccade_spec::recipe::texture::{BaseMaterial, MaterialType};

use crate::maps::GrayscaleBuffer;
use crate::noise::{Fbm, Noise2D, PerlinNoise};
use crate::pattern::{BrickPattern, WoodGrainPattern};

use super::helpers::{apply_pattern_to_buffer, apply_transform};

/// Apply material-specific base pattern to height map.
pub fn apply_material_pattern(
    height_map: &mut GrayscaleBuffer,
    material: &BaseMaterial,
    width: u32,
    height: u32,
    seed: u32,
) {
    match material.material_type {
        MaterialType::Brick => {
            let mut brick = BrickPattern::new(width, height).with_seed(seed);

            // Apply brick pattern params from spec if provided
            if let Some(ref params) = material.brick_pattern {
                brick = brick
                    .with_brick_size(params.brick_width, params.brick_height)
                    .with_row_offset(params.offset);

                // Mortar depth comes from normal_params if provided
                let mortar_depth = material
                    .normal_params
                    .as_ref()
                    .map(|np| np.mortar_depth)
                    .unwrap_or(0.3);

                brick = brick.with_mortar(params.mortar_width, mortar_depth);
            }

            apply_pattern_to_buffer(&brick, height_map);
        }
        MaterialType::Wood => {
            let wood = WoodGrainPattern::new(width, height, seed);
            apply_pattern_to_buffer(&wood, height_map);
        }
        MaterialType::Metal | MaterialType::Stone | MaterialType::Concrete => {
            // Add noise-based height variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(4)
                .with_persistence(0.5);

            apply_transform(height_map, |x, y, _| {
                let nx = x as f64 * 0.02;
                let ny = y as f64 * 0.02;
                noise.sample_01(nx, ny)
            });
        }
        _ => {
            // Default: slight noise variation
            let noise = Fbm::new(PerlinNoise::new(seed))
                .with_octaves(2)
                .with_persistence(0.5);

            apply_transform(height_map, |x, y, _| {
                let nx = x as f64 * 0.01;
                let ny = y as f64 * 0.01;
                let v = 0.5 + noise.sample(nx, ny) * 0.1;
                v.clamp(0.0, 1.0)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_material(material_type: MaterialType) -> BaseMaterial {
        BaseMaterial {
            material_type,
            base_color: [0.5, 0.5, 0.5],
            roughness_range: None,
            metallic: None,
            brick_pattern: None,
            normal_params: None,
        }
    }

    fn assert_modified_and_deterministic(material: &BaseMaterial) {
        let (w, h) = (32u32, 32u32);

        let mut a = GrayscaleBuffer::new(w, h, 0.5);
        let mut b = GrayscaleBuffer::new(w, h, 0.5);

        apply_material_pattern(&mut a, material, w, h, 123);
        apply_material_pattern(&mut b, material, w, h, 123);

        assert_eq!(
            a.data, b.data,
            "material {:?} not deterministic",
            material.material_type
        );
        assert!(
            a.data.iter().any(|&v| (v - 0.5).abs() > 1e-9),
            "material {:?} did not modify buffer",
            material.material_type
        );
        assert!(
            a.data.iter().all(|&v| (0.0..=1.0).contains(&v)),
            "material {:?} produced out-of-range values",
            material.material_type
        );
    }

    #[test]
    fn apply_material_pattern_brick_is_deterministic_and_nontrivial() {
        assert_modified_and_deterministic(&base_material(MaterialType::Brick));
    }

    #[test]
    fn apply_material_pattern_wood_is_deterministic_and_nontrivial() {
        assert_modified_and_deterministic(&base_material(MaterialType::Wood));
    }

    #[test]
    fn apply_material_pattern_metal_is_deterministic_and_nontrivial() {
        assert_modified_and_deterministic(&base_material(MaterialType::Metal));
    }

    #[test]
    fn apply_material_pattern_procedural_is_deterministic_and_nontrivial() {
        assert_modified_and_deterministic(&base_material(MaterialType::Procedural));
    }
}
