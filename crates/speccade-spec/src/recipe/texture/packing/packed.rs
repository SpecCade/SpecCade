//! Packed output specification for combining multiple channel sources.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::validation::common::CommonValidationError;
use super::types::ChannelSource;

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
    pub fn rgba(r: ChannelSource, g: ChannelSource, b: ChannelSource, a: ChannelSource) -> Self {
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
        fn check_constant(
            source: &ChannelSource,
            channel: &str,
        ) -> Result<(), CommonValidationError> {
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
            if let ChannelSource::Extended {
                key,
                component: Some(comp),
                ..
            } = source
            {
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
