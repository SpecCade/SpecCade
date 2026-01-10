//! Channel packing types for texture generation.
//!
//! This module provides an unopinionated channel packing system where users define
//! their own map keys (not predefined names like "albedo"), then reference those
//! keys when packing channels into output textures.
//!
//! # Core Concept
//!
//! ```text
//! recipe.params.maps = { "my_key": <generation_params> }  // User-defined keys
//! outputs[].channels = { "r": "my_key" }                   // Reference by key
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::validation::common::CommonValidationError;

/// A color component that can be extracted from an RGB/RGBA map.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ColorComponent {
    /// Red channel.
    R,
    /// Green channel.
    G,
    /// Blue channel.
    B,
    /// Alpha channel.
    A,
    /// Luminance (computed from RGB).
    Luminance,
}

impl ColorComponent {
    /// Returns true if this component requires an alpha channel.
    pub fn requires_alpha(&self) -> bool {
        matches!(self, ColorComponent::A)
    }
}

/// A channel source can be a map key reference or a constant value.
///
/// This enum supports three forms of specifying a channel source:
/// - Simple key reference: `"my_height_map"` - uses the map directly (luminance for RGB maps)
/// - Extended reference: `{ "key": "my_map", "component": "r", "invert": true }`
/// - Constant value: `{ "constant": 0.5 }` - fills with a constant 0.0-1.0 value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChannelSource {
    /// Simple key reference (for grayscale maps, uses luminance for RGB).
    Key(String),
    /// Extended reference with options.
    Extended {
        /// The map key to reference.
        key: String,
        /// Which color component to extract (defaults to luminance if not specified).
        #[serde(skip_serializing_if = "Option::is_none")]
        component: Option<ColorComponent>,
        /// Whether to invert the values (1.0 - value).
        #[serde(default)]
        invert: bool,
    },
    /// Constant value (0.0 to 1.0).
    Constant {
        /// The constant value to fill the channel with.
        constant: f32,
    },
}

impl ChannelSource {
    /// Creates a simple key reference.
    pub fn key(key: impl Into<String>) -> Self {
        ChannelSource::Key(key.into())
    }

    /// Creates an extended key reference.
    pub fn extended(key: impl Into<String>) -> ExtendedBuilder {
        ExtendedBuilder {
            key: key.into(),
            component: None,
            invert: false,
        }
    }

    /// Creates a constant value source.
    pub fn constant(value: f32) -> Self {
        ChannelSource::Constant { constant: value }
    }

    /// Returns the referenced key, if any.
    pub fn referenced_key(&self) -> Option<&str> {
        match self {
            ChannelSource::Key(key) => Some(key),
            ChannelSource::Extended { key, .. } => Some(key),
            ChannelSource::Constant { .. } => None,
        }
    }

    /// Returns true if this source uses inversion.
    pub fn is_inverted(&self) -> bool {
        match self {
            ChannelSource::Extended { invert, .. } => *invert,
            _ => false,
        }
    }

    /// Returns the color component extraction, if specified.
    pub fn component(&self) -> Option<ColorComponent> {
        match self {
            ChannelSource::Extended { component, .. } => *component,
            _ => None,
        }
    }
}

/// Builder for creating extended channel sources.
pub struct ExtendedBuilder {
    key: String,
    component: Option<ColorComponent>,
    invert: bool,
}

impl ExtendedBuilder {
    /// Sets the color component to extract.
    pub fn component(mut self, component: ColorComponent) -> Self {
        self.component = Some(component);
        self
    }

    /// Sets whether to invert the values.
    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    /// Builds the extended channel source.
    pub fn build(self) -> ChannelSource {
        ChannelSource::Extended {
            key: self.key,
            component: self.component,
            invert: self.invert,
        }
    }
}

/// Packed output specification defining how to pack map channels into RGBA.
///
/// This struct defines which source maps/values go into each output channel.
/// Each channel can reference a map key, extract a specific component, or use
/// a constant value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackedChannels {
    /// Red channel source.
    pub r: ChannelSource,
    /// Green channel source.
    pub g: ChannelSource,
    /// Blue channel source.
    pub b: ChannelSource,
    /// Alpha channel source (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a: Option<ChannelSource>,
}

impl PackedChannels {
    /// Creates a new PackedChannels with the given RGB sources.
    pub fn rgb(r: ChannelSource, g: ChannelSource, b: ChannelSource) -> Self {
        Self { r, g, b, a: None }
    }

    /// Creates a new PackedChannels with the given RGBA sources.
    pub fn rgba(
        r: ChannelSource,
        g: ChannelSource,
        b: ChannelSource,
        a: ChannelSource,
    ) -> Self {
        Self {
            r,
            g,
            b,
            a: Some(a),
        }
    }

    /// Returns all unique map keys referenced by this packing specification.
    pub fn referenced_keys(&self) -> HashSet<&str> {
        let mut keys = HashSet::new();
        if let Some(key) = self.r.referenced_key() {
            keys.insert(key);
        }
        if let Some(key) = self.g.referenced_key() {
            keys.insert(key);
        }
        if let Some(key) = self.b.referenced_key() {
            keys.insert(key);
        }
        if let Some(ref a) = self.a {
            if let Some(key) = a.referenced_key() {
                keys.insert(key);
            }
        }
        keys
    }

    /// Validates that all referenced keys exist in the provided set of available map keys.
    ///
    /// # Arguments
    /// * `available_keys` - Set of map keys that are defined in the recipe
    ///
    /// # Returns
    /// * `Ok(())` if all referenced keys exist
    /// * `Err(CommonValidationError)` with details about missing keys
    pub fn validate_key_references(
        &self,
        available_keys: &HashSet<&str>,
    ) -> Result<(), CommonValidationError> {
        let referenced = self.referenced_keys();
        let missing: Vec<_> = referenced
            .iter()
            .filter(|k| !available_keys.contains(*k))
            .collect();

        if !missing.is_empty() {
            return Err(CommonValidationError::new(format!(
                "channel packing references undefined map keys: {:?}",
                missing
            )));
        }
        Ok(())
    }

    /// Validates constant values are in the valid range [0.0, 1.0].
    ///
    /// # Returns
    /// * `Ok(())` if all constant values are valid
    /// * `Err(CommonValidationError)` if any constant is out of range
    pub fn validate_constants(&self) -> Result<(), CommonValidationError> {
        fn check_constant(source: &ChannelSource, channel: &str) -> Result<(), CommonValidationError> {
            if let ChannelSource::Constant { constant } = source {
                if !constant.is_finite() {
                    return Err(CommonValidationError::new(format!(
                        "{} channel constant must be finite, got {}",
                        channel, constant
                    )));
                }
                if !(0.0..=1.0).contains(constant) {
                    return Err(CommonValidationError::new(format!(
                        "{} channel constant must be in [0.0, 1.0], got {}",
                        channel, constant
                    )));
                }
            }
            Ok(())
        }

        check_constant(&self.r, "r")?;
        check_constant(&self.g, "g")?;
        check_constant(&self.b, "b")?;
        if let Some(ref a) = self.a {
            check_constant(a, "a")?;
        }
        Ok(())
    }

    /// Validates component extraction usage.
    ///
    /// This checks that component extraction is only used with maps that
    /// have the required channels (e.g., alpha component requires RGBA map).
    ///
    /// # Arguments
    /// * `rgba_keys` - Set of map keys that produce RGBA output
    /// * `rgb_keys` - Set of map keys that produce RGB output
    ///
    /// # Returns
    /// * `Ok(())` if all component extractions are valid
    /// * `Err(CommonValidationError)` if component extraction is invalid
    pub fn validate_component_extraction(
        &self,
        rgba_keys: &HashSet<&str>,
        rgb_keys: &HashSet<&str>,
    ) -> Result<(), CommonValidationError> {
        fn check_component(
            source: &ChannelSource,
            channel: &str,
            rgba_keys: &HashSet<&str>,
            rgb_keys: &HashSet<&str>,
        ) -> Result<(), CommonValidationError> {
            if let ChannelSource::Extended { key, component: Some(comp), .. } = source {
                // Check if extracting alpha from a non-RGBA map
                if comp.requires_alpha()
                    && rgb_keys.contains(key.as_str())
                    && !rgba_keys.contains(key.as_str())
                {
                    return Err(CommonValidationError::new(format!(
                        "{} channel extracts alpha from RGB-only map '{}'; alpha requires RGBA map",
                        channel, key
                    )));
                }
            }
            Ok(())
        }

        check_component(&self.r, "r", rgba_keys, rgb_keys)?;
        check_component(&self.g, "g", rgba_keys, rgb_keys)?;
        check_component(&self.b, "b", rgba_keys, rgb_keys)?;
        if let Some(ref a) = self.a {
            check_component(a, "a", rgba_keys, rgb_keys)?;
        }
        Ok(())
    }

    /// Performs full validation of the PackedChannels.
    ///
    /// # Arguments
    /// * `available_keys` - Set of all defined map keys
    /// * `rgba_keys` - Set of map keys that produce RGBA output
    /// * `rgb_keys` - Set of map keys that produce RGB output (grayscale maps not included)
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(CommonValidationError)` with the first validation error found
    pub fn validate(
        &self,
        available_keys: &HashSet<&str>,
        rgba_keys: &HashSet<&str>,
        rgb_keys: &HashSet<&str>,
    ) -> Result<(), CommonValidationError> {
        self.validate_key_references(available_keys)?;
        self.validate_constants()?;
        self.validate_component_extraction(rgba_keys, rgb_keys)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ColorComponent Tests
    // ========================================================================

    #[test]
    fn test_color_component_serde() {
        let component = ColorComponent::R;
        let json = serde_json::to_string(&component).unwrap();
        assert_eq!(json, "\"r\"");

        let parsed: ColorComponent = serde_json::from_str("\"luminance\"").unwrap();
        assert_eq!(parsed, ColorComponent::Luminance);
    }

    #[test]
    fn test_color_component_all_variants() {
        for (variant, expected_json) in [
            (ColorComponent::R, "\"r\""),
            (ColorComponent::G, "\"g\""),
            (ColorComponent::B, "\"b\""),
            (ColorComponent::A, "\"a\""),
            (ColorComponent::Luminance, "\"luminance\""),
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json);
            let parsed: ColorComponent = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn test_color_component_requires_alpha() {
        assert!(!ColorComponent::R.requires_alpha());
        assert!(!ColorComponent::G.requires_alpha());
        assert!(!ColorComponent::B.requires_alpha());
        assert!(ColorComponent::A.requires_alpha());
        assert!(!ColorComponent::Luminance.requires_alpha());
    }

    // ========================================================================
    // ChannelSource Tests
    // ========================================================================

    #[test]
    fn test_channel_source_key_serde() {
        let source = ChannelSource::Key("my_height".to_string());
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"my_height\"");

        let parsed: ChannelSource = serde_json::from_str("\"roughness_map\"").unwrap();
        assert_eq!(parsed, ChannelSource::Key("roughness_map".to_string()));
    }

    #[test]
    fn test_channel_source_extended_serde() {
        let source = ChannelSource::Extended {
            key: "my_map".to_string(),
            component: Some(ColorComponent::R),
            invert: true,
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"key\":\"my_map\""));
        assert!(json.contains("\"component\":\"r\""));
        assert!(json.contains("\"invert\":true"));

        let parsed: ChannelSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, source);
    }

    #[test]
    fn test_channel_source_extended_minimal_serde() {
        // Extended with only key (no component, invert defaults to false)
        let json = r#"{"key":"my_map"}"#;
        let parsed: ChannelSource = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            ChannelSource::Extended {
                key: "my_map".to_string(),
                component: None,
                invert: false,
            }
        );
    }

    #[test]
    fn test_channel_source_constant_serde() {
        let source = ChannelSource::Constant { constant: 0.5 };
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#"{"constant":0.5}"#);

        let parsed: ChannelSource = serde_json::from_str(r#"{"constant":1.0}"#).unwrap();
        assert_eq!(parsed, ChannelSource::Constant { constant: 1.0 });
    }

    #[test]
    fn test_channel_source_builders() {
        let key_source = ChannelSource::key("height");
        assert_eq!(key_source.referenced_key(), Some("height"));

        let extended = ChannelSource::extended("normal")
            .component(ColorComponent::G)
            .invert(true)
            .build();
        assert_eq!(extended.referenced_key(), Some("normal"));
        assert_eq!(extended.component(), Some(ColorComponent::G));
        assert!(extended.is_inverted());

        let constant = ChannelSource::constant(0.75);
        assert_eq!(constant.referenced_key(), None);
    }

    #[test]
    fn test_channel_source_referenced_key() {
        assert_eq!(
            ChannelSource::Key("test".to_string()).referenced_key(),
            Some("test")
        );
        assert_eq!(
            ChannelSource::Extended {
                key: "test2".to_string(),
                component: None,
                invert: false
            }
            .referenced_key(),
            Some("test2")
        );
        assert_eq!(
            ChannelSource::Constant { constant: 0.5 }.referenced_key(),
            None
        );
    }

    // ========================================================================
    // PackedChannels Tests
    // ========================================================================

    #[test]
    fn test_packed_channels_rgb() {
        let packed = PackedChannels::rgb(
            ChannelSource::key("height"),
            ChannelSource::key("roughness"),
            ChannelSource::constant(1.0),
        );
        assert!(packed.a.is_none());
    }

    #[test]
    fn test_packed_channels_rgba() {
        let packed = PackedChannels::rgba(
            ChannelSource::key("height"),
            ChannelSource::key("roughness"),
            ChannelSource::key("metallic"),
            ChannelSource::constant(1.0),
        );
        assert!(packed.a.is_some());
    }

    #[test]
    fn test_packed_channels_serde_roundtrip() {
        let packed = PackedChannels {
            r: ChannelSource::Key("height".to_string()),
            g: ChannelSource::Extended {
                key: "normal".to_string(),
                component: Some(ColorComponent::G),
                invert: false,
            },
            b: ChannelSource::Constant { constant: 0.5 },
            a: Some(ChannelSource::constant(1.0)),
        };

        let json = serde_json::to_string_pretty(&packed).unwrap();
        let parsed: PackedChannels = serde_json::from_str(&json).unwrap();
        assert_eq!(packed, parsed);
    }

    #[test]
    fn test_packed_channels_referenced_keys() {
        let packed = PackedChannels {
            r: ChannelSource::Key("height".to_string()),
            g: ChannelSource::Key("roughness".to_string()),
            b: ChannelSource::Extended {
                key: "height".to_string(), // duplicate key
                component: Some(ColorComponent::R),
                invert: true,
            },
            a: Some(ChannelSource::Constant { constant: 1.0 }),
        };

        let keys = packed.referenced_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("height"));
        assert!(keys.contains("roughness"));
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_key_references_success() {
        let packed = PackedChannels::rgb(
            ChannelSource::key("height"),
            ChannelSource::key("roughness"),
            ChannelSource::constant(1.0),
        );

        let available: HashSet<&str> = ["height", "roughness", "metallic"].into_iter().collect();
        assert!(packed.validate_key_references(&available).is_ok());
    }

    #[test]
    fn test_validate_key_references_missing() {
        let packed = PackedChannels::rgb(
            ChannelSource::key("height"),
            ChannelSource::key("missing_key"),
            ChannelSource::constant(1.0),
        );

        let available: HashSet<&str> = ["height", "roughness"].into_iter().collect();
        let result = packed.validate_key_references(&available);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing_key"));
    }

    #[test]
    fn test_validate_constants_valid() {
        let packed = PackedChannels::rgba(
            ChannelSource::constant(0.0),
            ChannelSource::constant(0.5),
            ChannelSource::constant(1.0),
            ChannelSource::constant(0.75),
        );
        assert!(packed.validate_constants().is_ok());
    }

    #[test]
    fn test_validate_constants_out_of_range() {
        let packed = PackedChannels::rgb(
            ChannelSource::constant(1.5), // out of range
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );
        let result = packed.validate_constants();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("[0.0, 1.0]"));
    }

    #[test]
    fn test_validate_constants_negative() {
        let packed = PackedChannels::rgb(
            ChannelSource::constant(-0.1), // negative
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );
        let result = packed.validate_constants();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_constants_nan() {
        let packed = PackedChannels::rgb(
            ChannelSource::constant(f32::NAN),
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );
        let result = packed.validate_constants();
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("finite"));
    }

    #[test]
    fn test_validate_component_extraction_valid() {
        let packed = PackedChannels::rgb(
            ChannelSource::extended("rgba_map")
                .component(ColorComponent::A)
                .build(),
            ChannelSource::extended("rgb_map")
                .component(ColorComponent::R)
                .build(),
            ChannelSource::constant(1.0),
        );

        let rgba_keys: HashSet<&str> = ["rgba_map"].into_iter().collect();
        let rgb_keys: HashSet<&str> = ["rgb_map"].into_iter().collect();

        assert!(packed
            .validate_component_extraction(&rgba_keys, &rgb_keys)
            .is_ok());
    }

    #[test]
    fn test_validate_component_extraction_alpha_from_rgb() {
        let packed = PackedChannels::rgb(
            ChannelSource::extended("rgb_only")
                .component(ColorComponent::A)
                .build(),
            ChannelSource::constant(0.5),
            ChannelSource::constant(0.5),
        );

        let rgba_keys: HashSet<&str> = HashSet::new();
        let rgb_keys: HashSet<&str> = ["rgb_only"].into_iter().collect();

        let result = packed.validate_component_extraction(&rgba_keys, &rgb_keys);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("alpha"));
    }

    #[test]
    fn test_validate_full_success() {
        let packed = PackedChannels::rgb(
            ChannelSource::key("height"),
            ChannelSource::extended("normal")
                .component(ColorComponent::G)
                .build(),
            ChannelSource::constant(0.5),
        );

        let available: HashSet<&str> = ["height", "normal"].into_iter().collect();
        let rgba_keys: HashSet<&str> = HashSet::new();
        let rgb_keys: HashSet<&str> = ["normal"].into_iter().collect();

        assert!(packed.validate(&available, &rgba_keys, &rgb_keys).is_ok());
    }

    #[test]
    fn test_validate_full_multiple_errors() {
        // This test verifies that validate returns the first error found
        let packed = PackedChannels::rgb(
            ChannelSource::key("missing"), // will fail key reference
            ChannelSource::constant(2.0),  // will fail constant validation
            ChannelSource::constant(0.5),
        );

        let available: HashSet<&str> = ["height"].into_iter().collect();
        let rgba_keys: HashSet<&str> = HashSet::new();
        let rgb_keys: HashSet<&str> = HashSet::new();

        // Should fail on key reference first
        let result = packed.validate(&available, &rgba_keys, &rgb_keys);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing"));
    }
}
