# Mesh primitives coverage example
#
# Demonstrates mesh stdlib functions that create complete specs and primitives.
# Covers: mesh_primitive, mesh_spec, baking_settings, triangulate_modifier

# mesh_primitive creates a base mesh primitive specification
# Parameters: primitive_type, dimensions [x, y, z]
primitive = mesh_primitive("cube", [1.0, 1.0, 1.0])

# baking_settings creates texture baking settings for PBR workflows
bake = baking_settings(
    ["normal", "ao"],
    margin = 4,
    resolution = [1024, 1024]
)

# triangulate_modifier ensures export-ready geometry
tri_mod = triangulate_modifier(
    ngon_method = "beauty",
    quad_method = "beauty"
)

# mesh_spec creates a complete spec using the mesh_primitive and modifiers
mesh_spec(
    asset_id = "stdlib-mesh-primitives-coverage-01",
    seed = 42,
    primitive = "cylinder",
    dimensions = [1.0, 2.0, 1.0],
    modifiers = [
        bevel_modifier(0.02, 2),
        triangulate_modifier("beauty", "fixed")
    ],
    output_path = "meshes/coverage_cylinder.glb",
    format = "glb"
)
