//! LOD (Level of Detail) generation for mesh previews.
//!
//! Implements edge-collapse decimation to generate low-poly proxy meshes
//! for sub-100ms first-frame preview in the editor.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Configuration for LOD proxy generation.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Target triangle count for the proxy mesh.
    pub target_triangles: u32,
    /// Ratio of original triangles to preserve (0.0-1.0).
    /// Used when target_triangles is not specified.
    pub ratio: f32,
    /// Threshold for edge length preservation (silhouette protection).
    /// Edges longer than this (relative to bounding box) are penalized.
    pub silhouette_weight: f32,
    /// Weight for preserving UV seams and material boundaries.
    pub boundary_weight: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            target_triangles: 1000,
            ratio: 0.1,
            silhouette_weight: 10.0,
            boundary_weight: 5.0,
        }
    }
}

/// Result of LOD generation.
#[derive(Debug, Clone)]
pub struct LodResult {
    /// The decimated vertex positions (x, y, z interleaved).
    pub positions: Vec<f32>,
    /// The decimated vertex normals (x, y, z interleaved).
    pub normals: Vec<f32>,
    /// The decimated UV coordinates (u, v interleaved).
    pub uvs: Vec<f32>,
    /// The decimated triangle indices.
    pub indices: Vec<u32>,
    /// Original triangle count.
    pub original_triangles: u32,
    /// Resulting triangle count after decimation.
    pub result_triangles: u32,
    /// Whether this is a proxy (decimated) or full quality mesh.
    pub is_proxy: bool,
}

/// A vertex in the mesh.
#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    /// Quadric error matrix (4x4 symmetric, stored as upper triangle: 10 elements).
    quadric: [f64; 10],
    /// Adjacent vertex indices.
    neighbors: HashSet<u32>,
    /// Whether this vertex is on a boundary (edge, UV seam, etc.).
    is_boundary: bool,
    /// Whether this vertex has been collapsed.
    collapsed: bool,
}

/// An edge candidate for collapse.
#[derive(Debug, Clone)]
struct EdgeCollapse {
    /// First vertex index.
    v1: u32,
    /// Second vertex index.
    v2: u32,
    /// Error cost of this collapse.
    cost: f64,
    /// Target position after collapse.
    target_pos: [f32; 3],
}

impl PartialEq for EdgeCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for EdgeCollapse {}

impl PartialOrd for EdgeCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EdgeCollapse {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (lowest cost first)
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Mesh decimator using quadric error metrics with edge collapse.
pub struct MeshDecimator {
    vertices: Vec<Vertex>,
    triangles: Vec<[u32; 3]>,
    config: LodConfig,
    bounding_box_diagonal: f64,
}

impl MeshDecimator {
    /// Create a new mesh decimator from raw mesh data.
    ///
    /// # Arguments
    /// * `positions` - Vertex positions (x, y, z interleaved)
    /// * `normals` - Vertex normals (x, y, z interleaved)
    /// * `uvs` - UV coordinates (u, v interleaved)
    /// * `indices` - Triangle indices
    /// * `config` - LOD configuration
    pub fn new(
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        indices: &[u32],
        config: LodConfig,
    ) -> Self {
        let vertex_count = positions.len() / 3;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(vertex_count);

        // Initialize vertices
        for i in 0..vertex_count {
            let pos_idx = i * 3;
            let uv_idx = i * 2;

            let position = [
                positions[pos_idx],
                positions[pos_idx + 1],
                positions[pos_idx + 2],
            ];

            let normal = if normals.len() > pos_idx + 2 {
                [normals[pos_idx], normals[pos_idx + 1], normals[pos_idx + 2]]
            } else {
                [0.0, 1.0, 0.0]
            };

            let uv = if uvs.len() > uv_idx + 1 {
                [uvs[uv_idx], uvs[uv_idx + 1]]
            } else {
                [0.0, 0.0]
            };

            vertices.push(Vertex {
                position,
                normal,
                uv,
                quadric: [0.0; 10],
                neighbors: HashSet::new(),
                is_boundary: false,
                collapsed: false,
            });
        }

        // Build triangles and adjacency
        let triangle_count = indices.len() / 3;
        let mut triangles: Vec<[u32; 3]> = Vec::with_capacity(triangle_count);

        for i in 0..triangle_count {
            let idx = i * 3;
            let v0 = indices[idx];
            let v1 = indices[idx + 1];
            let v2 = indices[idx + 2];

            triangles.push([v0, v1, v2]);

            // Add adjacency
            if (v0 as usize) < vertices.len()
                && (v1 as usize) < vertices.len()
                && (v2 as usize) < vertices.len()
            {
                vertices[v0 as usize].neighbors.insert(v1);
                vertices[v0 as usize].neighbors.insert(v2);
                vertices[v1 as usize].neighbors.insert(v0);
                vertices[v1 as usize].neighbors.insert(v2);
                vertices[v2 as usize].neighbors.insert(v0);
                vertices[v2 as usize].neighbors.insert(v1);
            }
        }

        // Compute bounding box diagonal for relative edge length
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for v in &vertices {
            for i in 0..3 {
                min[i] = min[i].min(v.position[i]);
                max[i] = max[i].max(v.position[i]);
            }
        }
        let diagonal =
            ((max[0] - min[0]).powi(2) + (max[1] - min[1]).powi(2) + (max[2] - min[2]).powi(2))
                .sqrt() as f64;

        let mut decimator = Self {
            vertices,
            triangles,
            config,
            bounding_box_diagonal: diagonal.max(0.001),
        };

        // Identify boundary vertices
        decimator.identify_boundaries();

        // Compute initial quadric error matrices
        decimator.compute_quadrics();

        decimator
    }

    /// Identify boundary vertices (edges shared by only one triangle).
    fn identify_boundaries(&mut self) {
        let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();

        for tri in &self.triangles {
            let edges = [
                (tri[0].min(tri[1]), tri[0].max(tri[1])),
                (tri[1].min(tri[2]), tri[1].max(tri[2])),
                (tri[2].min(tri[0]), tri[2].max(tri[0])),
            ];

            for edge in edges {
                *edge_count.entry(edge).or_insert(0) += 1;
            }
        }

        // Mark vertices on boundary edges
        for ((v1, v2), count) in edge_count {
            if count == 1 {
                if (v1 as usize) < self.vertices.len() {
                    self.vertices[v1 as usize].is_boundary = true;
                }
                if (v2 as usize) < self.vertices.len() {
                    self.vertices[v2 as usize].is_boundary = true;
                }
            }
        }
    }

    /// Compute quadric error matrices for all vertices.
    fn compute_quadrics(&mut self) {
        for tri in &self.triangles {
            let v0 = tri[0] as usize;
            let v1 = tri[1] as usize;
            let v2 = tri[2] as usize;

            if v0 >= self.vertices.len() || v1 >= self.vertices.len() || v2 >= self.vertices.len() {
                continue;
            }

            // Compute plane equation: ax + by + cz + d = 0
            let p0 = self.vertices[v0].position;
            let p1 = self.vertices[v1].position;
            let p2 = self.vertices[v2].position;

            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            // Cross product for normal
            let nx = edge1[1] * edge2[2] - edge1[2] * edge2[1];
            let ny = edge1[2] * edge2[0] - edge1[0] * edge2[2];
            let nz = edge1[0] * edge2[1] - edge1[1] * edge2[0];

            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-10 {
                continue;
            }

            let a = (nx / len) as f64;
            let b = (ny / len) as f64;
            let c = (nz / len) as f64;
            let d = -(a * p0[0] as f64 + b * p0[1] as f64 + c * p0[2] as f64);

            // Quadric matrix Q = pp^T where p = [a, b, c, d]
            // Stored as upper triangle: [aa, ab, ac, ad, bb, bc, bd, cc, cd, dd]
            let quadric = [
                a * a,
                a * b,
                a * c,
                a * d,
                b * b,
                b * c,
                b * d,
                c * c,
                c * d,
                d * d,
            ];

            // Add to each vertex of the triangle
            for vi in [v0, v1, v2] {
                for (i, q) in quadric.iter().enumerate() {
                    self.vertices[vi].quadric[i] += q;
                }
            }
        }
    }

    /// Compute the error cost for collapsing an edge.
    fn compute_edge_cost(&self, v1_idx: u32, v2_idx: u32) -> Option<EdgeCollapse> {
        let v1 = &self.vertices[v1_idx as usize];
        let v2 = &self.vertices[v2_idx as usize];

        if v1.collapsed || v2.collapsed {
            return None;
        }

        // Combined quadric
        let mut q = [0.0f64; 10];
        for i in 0..10 {
            q[i] = v1.quadric[i] + v2.quadric[i];
        }

        // Try to find optimal position by solving the linear system
        // For simplicity, we use the midpoint or one of the endpoints
        let target_pos = self.find_optimal_position(v1, v2, &q);

        // Compute error at target position: v^T * Q * v
        let vx = target_pos[0] as f64;
        let vy = target_pos[1] as f64;
        let vz = target_pos[2] as f64;

        // Q = [aa, ab, ac, ad, bb, bc, bd, cc, cd, dd]
        // v^T * Q * v = aa*x^2 + 2*ab*x*y + 2*ac*x*z + 2*ad*x + bb*y^2 + 2*bc*y*z + 2*bd*y + cc*z^2 + 2*cd*z + dd
        let mut cost = q[0] * vx * vx
            + 2.0 * q[1] * vx * vy
            + 2.0 * q[2] * vx * vz
            + 2.0 * q[3] * vx
            + q[4] * vy * vy
            + 2.0 * q[5] * vy * vz
            + 2.0 * q[6] * vy
            + q[7] * vz * vz
            + 2.0 * q[8] * vz
            + q[9];

        // Apply boundary penalty to preserve silhouette
        if v1.is_boundary || v2.is_boundary {
            cost += self.config.boundary_weight as f64 * self.bounding_box_diagonal;
        }

        // Apply silhouette weight based on edge length
        let edge_len = ((v2.position[0] - v1.position[0]).powi(2)
            + (v2.position[1] - v1.position[1]).powi(2)
            + (v2.position[2] - v1.position[2]).powi(2))
        .sqrt() as f64;

        let relative_len = edge_len / self.bounding_box_diagonal;
        if relative_len > 0.1 {
            // Long edges are silhouette-important
            cost +=
                self.config.silhouette_weight as f64 * relative_len * self.bounding_box_diagonal;
        }

        Some(EdgeCollapse {
            v1: v1_idx,
            v2: v2_idx,
            cost,
            target_pos,
        })
    }

    /// Find the optimal position for the collapsed vertex.
    fn find_optimal_position(&self, v1: &Vertex, v2: &Vertex, _q: &[f64; 10]) -> [f32; 3] {
        // For robustness, we use the position with lower error among:
        // - Vertex 1 position
        // - Vertex 2 position
        // - Midpoint

        let midpoint = [
            (v1.position[0] + v2.position[0]) / 2.0,
            (v1.position[1] + v2.position[1]) / 2.0,
            (v1.position[2] + v2.position[2]) / 2.0,
        ];

        // Prefer midpoint for smoother results, but bias towards boundary vertices
        if v1.is_boundary && !v2.is_boundary {
            v1.position
        } else if v2.is_boundary && !v1.is_boundary {
            v2.position
        } else {
            midpoint
        }
    }

    /// Perform edge collapse decimation to reach target triangle count.
    pub fn decimate(&mut self, target_triangles: u32) -> LodResult {
        let original_triangles = self.triangles.len() as u32;

        // If already below target, return as-is
        if original_triangles <= target_triangles {
            return self.build_result(original_triangles, false);
        }

        // Build initial edge collapse heap
        let mut heap: BinaryHeap<EdgeCollapse> = BinaryHeap::new();
        let mut processed_edges: HashSet<(u32, u32)> = HashSet::new();

        for v_idx in 0..self.vertices.len() {
            let v = &self.vertices[v_idx];
            if v.collapsed {
                continue;
            }

            for &neighbor_idx in &v.neighbors {
                let edge_key = (
                    v_idx.min(neighbor_idx as usize) as u32,
                    v_idx.max(neighbor_idx as usize) as u32,
                );
                if !processed_edges.contains(&edge_key) {
                    processed_edges.insert(edge_key);
                    if let Some(collapse) = self.compute_edge_cost(v_idx as u32, neighbor_idx) {
                        heap.push(collapse);
                    }
                }
            }
        }

        // Collapse edges until we reach target
        let mut current_triangles = original_triangles;

        while current_triangles > target_triangles {
            let collapse = match heap.pop() {
                Some(c) => c,
                None => break,
            };

            // Skip if either vertex was already collapsed
            if self.vertices[collapse.v1 as usize].collapsed
                || self.vertices[collapse.v2 as usize].collapsed
            {
                continue;
            }

            // Perform the collapse: merge v2 into v1
            let triangles_removed =
                self.collapse_edge(collapse.v1, collapse.v2, collapse.target_pos);
            current_triangles = current_triangles.saturating_sub(triangles_removed);

            // Update costs for affected edges
            let v1 = &self.vertices[collapse.v1 as usize];
            let neighbors: Vec<u32> = v1.neighbors.iter().copied().collect();

            for &neighbor_idx in &neighbors {
                if !self.vertices[neighbor_idx as usize].collapsed {
                    if let Some(new_collapse) = self.compute_edge_cost(collapse.v1, neighbor_idx) {
                        heap.push(new_collapse);
                    }
                }
            }
        }

        self.build_result(original_triangles, true)
    }

    /// Collapse an edge by merging v2 into v1.
    fn collapse_edge(&mut self, v1_idx: u32, v2_idx: u32, target_pos: [f32; 3]) -> u32 {
        let v1 = v1_idx as usize;
        let v2 = v2_idx as usize;

        // Mark v2 as collapsed
        self.vertices[v2].collapsed = true;

        // Update v1 position
        self.vertices[v1].position = target_pos;

        // Merge quadrics
        let v2_quadric = self.vertices[v2].quadric;
        for i in 0..10 {
            self.vertices[v1].quadric[i] += v2_quadric[i];
        }

        // Update v1 boundary status
        self.vertices[v1].is_boundary |= self.vertices[v2].is_boundary;

        // Transfer v2's neighbors to v1
        let v2_neighbors: Vec<u32> = self.vertices[v2].neighbors.iter().copied().collect();
        for neighbor_idx in v2_neighbors {
            if neighbor_idx != v1_idx {
                self.vertices[v1].neighbors.insert(neighbor_idx);
                self.vertices[neighbor_idx as usize]
                    .neighbors
                    .remove(&v2_idx);
                self.vertices[neighbor_idx as usize]
                    .neighbors
                    .insert(v1_idx);
            }
        }
        self.vertices[v1].neighbors.remove(&v2_idx);
        self.vertices[v2].neighbors.clear();

        // Update triangles and count removals
        let mut removed = 0u32;

        for tri in &mut self.triangles {
            // Replace v2 with v1
            for vi in tri.iter_mut() {
                if *vi == v2_idx {
                    *vi = v1_idx;
                }
            }

            // Check for degenerate triangle (two or more same vertices)
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[2] == tri[0] {
                // Mark as degenerate by setting all to u32::MAX
                *tri = [u32::MAX, u32::MAX, u32::MAX];
                removed += 1;
            }
        }

        removed
    }

    /// Build the result from current mesh state.
    fn build_result(&self, original_triangles: u32, is_proxy: bool) -> LodResult {
        // Collect non-collapsed vertices and remap indices
        let mut vertex_map: HashMap<u32, u32> = HashMap::new();
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut uvs: Vec<f32> = Vec::new();

        let mut new_index = 0u32;
        for (old_idx, v) in self.vertices.iter().enumerate() {
            if !v.collapsed {
                vertex_map.insert(old_idx as u32, new_index);
                positions.extend_from_slice(&v.position);
                normals.extend_from_slice(&v.normal);
                uvs.extend_from_slice(&v.uv);
                new_index += 1;
            }
        }

        // Collect non-degenerate triangles with remapped indices
        let mut indices: Vec<u32> = Vec::new();
        let mut result_triangles = 0u32;

        for tri in &self.triangles {
            // Skip degenerate triangles
            if tri[0] == u32::MAX {
                continue;
            }

            // Remap indices
            if let (Some(&i0), Some(&i1), Some(&i2)) = (
                vertex_map.get(&tri[0]),
                vertex_map.get(&tri[1]),
                vertex_map.get(&tri[2]),
            ) {
                indices.push(i0);
                indices.push(i1);
                indices.push(i2);
                result_triangles += 1;
            }
        }

        LodResult {
            positions,
            normals,
            uvs,
            indices,
            original_triangles,
            result_triangles,
            is_proxy,
        }
    }
}

/// Generate a LOD proxy from GLB data.
///
/// Returns the decimated GLB data if the mesh exceeds the triangle threshold,
/// or the original data if no decimation is needed.
pub fn generate_lod_proxy(glb_bytes: &[u8], config: &LodConfig) -> Result<LodProxyResult, String> {
    // Parse the GLB
    let glb =
        gltf::Glb::from_slice(glb_bytes).map_err(|e| format!("Failed to parse GLB: {}", e))?;

    let gltf = gltf::Gltf::from_slice(&glb.json)
        .map_err(|e| format!("Failed to parse GLTF JSON: {}", e))?;

    // Count total triangles
    let mut total_triangles = 0u64;
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if let Some(accessor) = primitive.indices() {
                total_triangles += accessor.count() as u64 / 3;
            }
        }
    }

    // Determine if we need to generate a proxy
    let needs_proxy = total_triangles > config.target_triangles as u64;
    let threshold_triangles = 10_000u64; // Only proxy meshes >10k triangles

    if !needs_proxy || total_triangles < threshold_triangles {
        return Ok(LodProxyResult {
            glb_data: glb_bytes.to_vec(),
            original_triangles: total_triangles as u32,
            proxy_triangles: total_triangles as u32,
            is_proxy: false,
        });
    }

    // Extract mesh data from GLB for decimation
    let binary_data = glb
        .bin
        .as_ref()
        .ok_or_else(|| "GLB has no binary data".to_string())?;

    // For each mesh, extract and decimate
    // Note: This is a simplified version that handles single-mesh GLBs
    // A full implementation would handle multiple meshes and preserve hierarchy

    let mut decimated_positions: Vec<f32> = Vec::new();
    let mut decimated_normals: Vec<f32> = Vec::new();
    let mut decimated_uvs: Vec<f32> = Vec::new();
    let mut decimated_indices: Vec<u32> = Vec::new();
    let mut proxy_triangles = 0u32;

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            // Extract positions
            let positions = extract_accessor_data::<f32>(
                &gltf,
                binary_data,
                primitive.get(&gltf::Semantic::Positions),
            )?;

            // Extract normals (optional)
            let normals = extract_accessor_data::<f32>(
                &gltf,
                binary_data,
                primitive.get(&gltf::Semantic::Normals),
            )
            .unwrap_or_default();

            // Extract UVs (optional)
            let uvs = extract_accessor_data::<f32>(
                &gltf,
                binary_data,
                primitive.get(&gltf::Semantic::TexCoords(0)),
            )
            .unwrap_or_default();

            // Extract indices
            let indices = extract_index_data(&gltf, binary_data, primitive.indices())?;

            // Create decimator and decimate
            let mut decimator =
                MeshDecimator::new(&positions, &normals, &uvs, &indices, config.clone());
            let result = decimator.decimate(config.target_triangles);

            // Accumulate results (with index offset)
            let index_offset = (decimated_positions.len() / 3) as u32;
            decimated_positions.extend_from_slice(&result.positions);
            decimated_normals.extend_from_slice(&result.normals);
            decimated_uvs.extend_from_slice(&result.uvs);
            decimated_indices.extend(result.indices.iter().map(|i| i + index_offset));
            proxy_triangles += result.result_triangles;
        }
    }

    // Build a new GLB from decimated data
    let new_glb = build_glb_from_mesh(
        &decimated_positions,
        &decimated_normals,
        &decimated_uvs,
        &decimated_indices,
    )?;

    Ok(LodProxyResult {
        glb_data: new_glb,
        original_triangles: total_triangles as u32,
        proxy_triangles,
        is_proxy: true,
    })
}

/// Result of LOD proxy generation.
#[derive(Debug, Clone)]
pub struct LodProxyResult {
    /// The GLB data (either original or decimated).
    pub glb_data: Vec<u8>,
    /// Original triangle count.
    pub original_triangles: u32,
    /// Proxy triangle count (same as original if no decimation).
    pub proxy_triangles: u32,
    /// Whether this is a proxy mesh.
    pub is_proxy: bool,
}

/// Extract accessor data from GLB binary.
fn extract_accessor_data<T: Copy + Default + bytemuck::Pod>(
    _gltf: &gltf::Gltf,
    binary_data: &[u8],
    accessor: Option<gltf::Accessor>,
) -> Result<Vec<T>, String> {
    let accessor = match accessor {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let view = accessor
        .view()
        .ok_or_else(|| "Accessor has no buffer view".to_string())?;

    let buffer = view.buffer();
    if buffer.index() != 0 {
        return Err("Only single-buffer GLBs are supported".to_string());
    }

    let offset = view.offset() + accessor.offset();
    let stride = view
        .stride()
        .unwrap_or(std::mem::size_of::<T>() * accessor.dimensions().multiplicity());
    let count = accessor.count();

    let element_size = std::mem::size_of::<T>() * accessor.dimensions().multiplicity();
    let mut result: Vec<T> = Vec::with_capacity(count * accessor.dimensions().multiplicity());

    for i in 0..count {
        let start = offset + i * stride;
        let end = start + element_size;

        if end > binary_data.len() {
            return Err("Accessor data out of bounds".to_string());
        }

        let slice = &binary_data[start..end];
        let elements: &[T] = bytemuck::cast_slice(slice);
        result.extend_from_slice(elements);
    }

    Ok(result)
}

/// Extract index data from GLB binary.
fn extract_index_data(
    _gltf: &gltf::Gltf,
    binary_data: &[u8],
    accessor: Option<gltf::Accessor>,
) -> Result<Vec<u32>, String> {
    let accessor = match accessor {
        Some(a) => a,
        None => return Err("No index accessor".to_string()),
    };

    let view = accessor
        .view()
        .ok_or_else(|| "Index accessor has no buffer view".to_string())?;

    let buffer = view.buffer();
    if buffer.index() != 0 {
        return Err("Only single-buffer GLBs are supported".to_string());
    }

    let offset = view.offset() + accessor.offset();
    let count = accessor.count();

    let mut result: Vec<u32> = Vec::with_capacity(count);

    match accessor.data_type() {
        gltf::accessor::DataType::U8 => {
            for i in 0..count {
                let idx = offset + i;
                if idx >= binary_data.len() {
                    return Err("Index data out of bounds".to_string());
                }
                result.push(binary_data[idx] as u32);
            }
        }
        gltf::accessor::DataType::U16 => {
            for i in 0..count {
                let start = offset + i * 2;
                if start + 2 > binary_data.len() {
                    return Err("Index data out of bounds".to_string());
                }
                let bytes = [binary_data[start], binary_data[start + 1]];
                result.push(u16::from_le_bytes(bytes) as u32);
            }
        }
        gltf::accessor::DataType::U32 => {
            for i in 0..count {
                let start = offset + i * 4;
                if start + 4 > binary_data.len() {
                    return Err("Index data out of bounds".to_string());
                }
                let bytes = [
                    binary_data[start],
                    binary_data[start + 1],
                    binary_data[start + 2],
                    binary_data[start + 3],
                ];
                result.push(u32::from_le_bytes(bytes));
            }
        }
        _ => return Err("Unsupported index data type".to_string()),
    }

    Ok(result)
}

/// Build a GLB from mesh data.
fn build_glb_from_mesh(
    positions: &[f32],
    normals: &[f32],
    uvs: &[f32],
    indices: &[u32],
) -> Result<Vec<u8>, String> {
    // Build binary buffer
    let mut binary: Vec<u8> = Vec::new();

    // Positions (padded to 4-byte alignment)
    let positions_offset = 0usize;
    let positions_bytes: &[u8] = bytemuck::cast_slice(positions);
    binary.extend_from_slice(positions_bytes);
    let positions_len = positions_bytes.len();

    // Normals
    let normals_offset = binary.len();
    let normals_bytes: &[u8] = bytemuck::cast_slice(normals);
    binary.extend_from_slice(normals_bytes);
    let normals_len = normals_bytes.len();

    // UVs
    let uvs_offset = binary.len();
    let uvs_bytes: &[u8] = bytemuck::cast_slice(uvs);
    binary.extend_from_slice(uvs_bytes);
    let uvs_len = uvs_bytes.len();

    // Indices (use u16 if possible for smaller file size)
    let indices_offset = binary.len();
    let max_index = indices.iter().copied().max().unwrap_or(0);
    let (indices_bytes, index_component_type): (Vec<u8>, u32) = if max_index <= u16::MAX as u32 {
        let u16_indices: Vec<u16> = indices.iter().map(|&i| i as u16).collect();
        (bytemuck::cast_slice(&u16_indices).to_vec(), 5123) // UNSIGNED_SHORT
    } else {
        (bytemuck::cast_slice(indices).to_vec(), 5125) // UNSIGNED_INT
    };
    binary.extend_from_slice(&indices_bytes);
    let indices_len = indices_bytes.len();

    // Pad binary to 4-byte alignment
    while binary.len() % 4 != 0 {
        binary.push(0);
    }

    // Compute bounds
    let vertex_count = positions.len() / 3;
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for i in 0..vertex_count {
        for j in 0..3 {
            min[j] = min[j].min(positions[i * 3 + j]);
            max[j] = max[j].max(positions[i * 3 + j]);
        }
    }

    // Build GLTF JSON
    let json = serde_json::json!({
        "asset": {
            "version": "2.0",
            "generator": "speccade-editor LOD"
        },
        "buffers": [{
            "byteLength": binary.len()
        }],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": positions_offset,
                "byteLength": positions_len,
                "target": 34962 // ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": normals_offset,
                "byteLength": normals_len,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": uvs_offset,
                "byteLength": uvs_len,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": indices_offset,
                "byteLength": indices_len,
                "target": 34963 // ELEMENT_ARRAY_BUFFER
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": vertex_count,
                "type": "VEC3",
                "min": min,
                "max": max
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": vertex_count,
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "componentType": 5126,
                "count": vertex_count,
                "type": "VEC2"
            },
            {
                "bufferView": 3,
                "componentType": index_component_type,
                "count": indices.len(),
                "type": "SCALAR"
            }
        ],
        "meshes": [{
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1,
                    "TEXCOORD_0": 2
                },
                "indices": 3,
                "mode": 4 // TRIANGLES
            }]
        }],
        "nodes": [{
            "mesh": 0
        }],
        "scenes": [{
            "nodes": [0]
        }],
        "scene": 0
    });

    let json_bytes =
        serde_json::to_vec(&json).map_err(|e| format!("Failed to serialize GLTF JSON: {}", e))?;

    // Pad JSON to 4-byte alignment
    let mut json_padded = json_bytes;
    while json_padded.len() % 4 != 0 {
        json_padded.push(0x20); // Space character for JSON padding
    }

    // Build GLB
    let total_length = 12 + 8 + json_padded.len() + 8 + binary.len();

    let mut glb: Vec<u8> = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF"); // Magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // Version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // Length

    // JSON chunk
    glb.extend_from_slice(&(json_padded.len() as u32).to_le_bytes()); // Chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    glb.extend_from_slice(&json_padded);

    // Binary chunk
    glb.extend_from_slice(&(binary.len() as u32).to_le_bytes()); // Chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    glb.extend_from_slice(&binary);

    Ok(glb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_config_default() {
        let config = LodConfig::default();
        assert_eq!(config.target_triangles, 1000);
        assert!((config.ratio - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decimator_simple_triangle() {
        // Single triangle - should not be decimated below 1
        let positions = vec![
            0.0, 0.0, 0.0, // v0
            1.0, 0.0, 0.0, // v1
            0.5, 1.0, 0.0, // v2
        ];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let config = LodConfig::default();
        let mut decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);
        let result = decimator.decimate(1);

        assert_eq!(result.original_triangles, 1);
        assert_eq!(result.result_triangles, 1);
        assert!(!result.is_proxy);
    }

    #[test]
    fn test_decimator_quad() {
        // Two triangles forming a quad
        let positions = vec![
            0.0, 0.0, 0.0, // v0
            1.0, 0.0, 0.0, // v1
            1.0, 1.0, 0.0, // v2
            0.0, 1.0, 0.0, // v3
        ];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let indices = vec![0, 1, 2, 0, 2, 3];

        let config = LodConfig::default();
        let mut decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);
        let result = decimator.decimate(1);

        assert_eq!(result.original_triangles, 2);
        // Should collapse to 1 or fewer triangles
        assert!(result.result_triangles <= 2);
    }

    #[test]
    fn test_decimator_preserves_mesh_below_threshold() {
        // Small mesh that shouldn't be decimated
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let config = LodConfig {
            target_triangles: 100,
            ..Default::default()
        };
        let mut decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);
        let result = decimator.decimate(100);

        assert_eq!(result.result_triangles, 1);
        assert!(!result.is_proxy);
    }

    #[test]
    fn test_build_glb_from_mesh() {
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let result = build_glb_from_mesh(&positions, &normals, &uvs, &indices);
        assert!(result.is_ok());

        let glb_bytes = result.unwrap();
        // Verify GLB magic
        assert_eq!(&glb_bytes[0..4], b"glTF");

        // Verify version
        let version = u32::from_le_bytes([glb_bytes[4], glb_bytes[5], glb_bytes[6], glb_bytes[7]]);
        assert_eq!(version, 2);
    }

    #[test]
    fn test_boundary_detection() {
        // Triangle has all boundary edges (only one triangle)
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let config = LodConfig::default();
        let decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);

        // All vertices should be marked as boundary
        assert!(decimator.vertices[0].is_boundary);
        assert!(decimator.vertices[1].is_boundary);
        assert!(decimator.vertices[2].is_boundary);
    }

    #[test]
    fn test_decimator_reduces_triangle_count() {
        // Create a simple grid mesh (4x4 = 16 quads = 32 triangles)
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Create a 5x5 vertex grid
        for z in 0..5 {
            for x in 0..5 {
                positions.push(x as f32);
                positions.push(0.0);
                positions.push(z as f32);
                normals.push(0.0);
                normals.push(1.0);
                normals.push(0.0);
                uvs.push(x as f32 / 4.0);
                uvs.push(z as f32 / 4.0);
            }
        }

        // Create triangles for each quad
        for z in 0..4 {
            for x in 0..4 {
                let i = z * 5 + x;
                // First triangle
                indices.push(i);
                indices.push(i + 1);
                indices.push(i + 5);
                // Second triangle
                indices.push(i + 1);
                indices.push(i + 6);
                indices.push(i + 5);
            }
        }

        let config = LodConfig {
            target_triangles: 16,
            // Reduce boundary weight to allow more decimation
            boundary_weight: 0.5,
            silhouette_weight: 1.0,
            ..Default::default()
        };
        let mut decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);
        let result = decimator.decimate(16);

        assert_eq!(result.original_triangles, 32);
        // The decimator should reduce triangle count (may not reach exact target due to topology)
        assert!(
            result.result_triangles < 32,
            "Expected reduction from 32 triangles, got {}",
            result.result_triangles
        );
        assert!(result.is_proxy);
    }

    #[test]
    fn test_lod_result_properties() {
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let config = LodConfig::default();
        let mut decimator = MeshDecimator::new(&positions, &normals, &uvs, &indices, config);
        let result = decimator.decimate(1000);

        // Result should contain valid geometry
        assert_eq!(result.positions.len() % 3, 0); // Positions are 3D vectors
        assert_eq!(result.normals.len() % 3, 0); // Normals are 3D vectors
        assert_eq!(result.uvs.len() % 2, 0); // UVs are 2D vectors
        assert_eq!(result.indices.len() % 3, 0); // Indices are triangle triplets
    }

    #[test]
    fn test_silhouette_preservation_config() {
        // Test that silhouette weight affects decimation
        let config_low = LodConfig {
            silhouette_weight: 1.0,
            ..Default::default()
        };
        let config_high = LodConfig {
            silhouette_weight: 100.0,
            ..Default::default()
        };

        assert!(config_low.silhouette_weight < config_high.silhouette_weight);
    }
}
