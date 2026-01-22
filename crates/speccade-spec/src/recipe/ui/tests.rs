//! Tests for UI recipe types.

use super::*;

#[test]
fn test_nine_slice_params_builder() {
    let params = UiNineSliceV1Params::new(256, 256, 16, 16)
        .with_padding(4)
        .with_background([0.1, 0.1, 0.1, 1.0]);

    assert_eq!(params.resolution, [256, 256]);
    assert_eq!(params.padding, 4);
    assert_eq!(params.regions.corner_size, [16, 16]);
    assert!(params.background_color.is_some());
}

#[test]
fn test_nine_slice_regions_edge_sizes() {
    let mut regions = NineSliceRegions {
        corner_size: [16, 16],
        top_left: [1.0, 0.0, 0.0, 1.0],
        top_right: [0.0, 1.0, 0.0, 1.0],
        bottom_left: [0.0, 0.0, 1.0, 1.0],
        bottom_right: [1.0, 1.0, 0.0, 1.0],
        top_edge: [0.5, 0.5, 0.5, 1.0],
        bottom_edge: [0.5, 0.5, 0.5, 1.0],
        left_edge: [0.5, 0.5, 0.5, 1.0],
        right_edge: [0.5, 0.5, 0.5, 1.0],
        center: [1.0, 1.0, 1.0, 1.0],
        edge_width: None,
        edge_height: None,
    };

    assert_eq!(regions.get_edge_width(), 16);
    assert_eq!(regions.get_edge_height(), 16);

    regions.edge_width = Some(8);
    regions.edge_height = Some(12);

    assert_eq!(regions.get_edge_width(), 8);
    assert_eq!(regions.get_edge_height(), 12);
}

#[test]
fn test_uv_rect_from_pixels() {
    let uv = UvRect::from_pixels(0, 0, 64, 64, 256, 256);

    assert_eq!(uv.u_min, 0.0);
    assert_eq!(uv.v_min, 0.0);
    assert_eq!(uv.u_max, 0.25);
    assert_eq!(uv.v_max, 0.25);
    assert_eq!(uv.width, 64);
    assert_eq!(uv.height, 64);
}

#[test]
fn test_icon_set_params_builder() {
    let params = UiIconSetV1Params::new(512, 512)
        .with_padding(4)
        .with_icon(IconEntry::new("close", 32, 32, [1.0, 0.0, 0.0, 1.0]))
        .with_icon(
            IconEntry::new("settings", 48, 48, [0.5, 0.5, 0.5, 1.0]).with_category("system"),
        );

    assert_eq!(params.resolution, [512, 512]);
    assert_eq!(params.padding, 4);
    assert_eq!(params.icons.len(), 2);
    assert_eq!(params.icons[0].id, "close");
    assert_eq!(params.icons[1].category.as_deref(), Some("system"));
}

#[test]
fn test_icon_entry_creation() {
    let icon = IconEntry::new("heart", 24, 24, [1.0, 0.0, 0.0, 1.0]).with_category("social");

    assert_eq!(icon.id, "heart");
    assert_eq!(icon.width, 24);
    assert_eq!(icon.height, 24);
    assert_eq!(icon.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(icon.category.as_deref(), Some("social"));
}

#[test]
fn test_nine_slice_serde() {
    let params = UiNineSliceV1Params::new(256, 256, 16, 16);
    let json = serde_json::to_string(&params).unwrap();
    let parsed: UiNineSliceV1Params = serde_json::from_str(&json).unwrap();

    assert_eq!(params, parsed);
}

#[test]
fn test_icon_set_serde() {
    let params = UiIconSetV1Params::new(512, 512).with_icon(IconEntry::new(
        "test",
        32,
        32,
        [1.0, 1.0, 1.0, 1.0],
    ));

    let json = serde_json::to_string(&params).unwrap();
    let parsed: UiIconSetV1Params = serde_json::from_str(&json).unwrap();

    assert_eq!(params, parsed);
}
