//! Core types for channel packing: color components and channel sources.

use serde::{Deserialize, Serialize};

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
