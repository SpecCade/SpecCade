//! Tests for mesh analysis module.

use super::analyze::analyze_manifold;
use super::analyze::{
    calculate_bounds, estimate_uv_islands, round_f64, triangle_area_2d, triangle_area_3d,
};

#[test]
fn test_triangle_area_3d() {
    // Unit triangle in XY plane
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [1.0, 0.0, 0.0];
    let p2 = [0.0, 1.0, 0.0];
    let area = triangle_area_3d(p0, p1, p2);
    assert!((area - 0.5).abs() < 1e-6);
}

#[test]
fn test_triangle_area_2d() {
    let p0 = [0.0, 0.0];
    let p1 = [1.0, 0.0];
    let p2 = [0.0, 1.0];
    let area = triangle_area_2d(p0, p1, p2);
    assert!((area - 0.5).abs() < 1e-6);
}

#[test]
fn test_calculate_bounds() {
    let positions = vec![[-1.0, -2.0, -3.0], [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]];
    let bounds = calculate_bounds(&positions);

    assert_eq!(bounds.bounds_min, [-1.0, -2.0, -3.0]);
    assert_eq!(bounds.bounds_max, [1.0, 2.0, 3.0]);
    assert_eq!(bounds.size, [2.0, 4.0, 6.0]);
    assert_eq!(bounds.center, [0.0, 0.0, 0.0]);
}

#[test]
fn test_analyze_manifold_degenerate() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    // Degenerate triangle with duplicate vertex
    let indices = vec![0, 0, 1];
    let manifold = analyze_manifold(&positions, &indices);

    assert!(!manifold.manifold);
    assert_eq!(manifold.degenerate_face_count, 1);
}

#[test]
fn test_estimate_uv_islands() {
    // Two separate UV triangles
    let uvs = vec![
        [0.0, 0.0],
        [0.1, 0.0],
        [0.0, 0.1],
        [0.9, 0.9],
        [1.0, 0.9],
        [0.9, 1.0],
    ];
    let islands = estimate_uv_islands(&uvs);
    assert!(islands >= 1);
}

#[test]
fn test_round_f64() {
    assert_eq!(round_f64(1.23456789, 6), 1.234568);
    assert_eq!(round_f64(1.23456789, 2), 1.23);
}
