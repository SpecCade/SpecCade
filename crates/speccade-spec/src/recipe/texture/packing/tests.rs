//! Tests for channel packing types and validation.

use std::collections::HashSet;

use super::types::{ColorComponent, ChannelSource};
use super::packed::PackedChannels;

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
