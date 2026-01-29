# Organic sculpt mesh - demonstrates metaball-based organic mesh generation
#
# This example creates a simple blob shape using metaballs with
# voxel remeshing, smoothing, and optional displacement noise.
#
# [VALIDATION]
# SHAPE: Blob-like organic form made from merged metaballs - main body sphere with 3 protrusions
# PROPORTIONS: Main body ~1.0 radius, extensions ~0.5-0.6 radius, small bump ~0.35 radius
# ORIENTATION: Main body centered at origin, extensions asymmetrically placed
# FRONT VIEW: Irregular blob silhouette, not perfectly round
# BACK VIEW: Different silhouette than front due to asymmetric extensions
# LEFT VIEW: Shows extension at position (-0.4, 0.3, 0.1)
# RIGHT VIEW: Shows extension at position (0.6, 0.0, 0.2)
# TOP VIEW: Shows small bump at (0, -0.4, 0.5) and overall irregular shape
# ISO VIEW: 3D organic form clearly visible, not a simple sphere
# NOTES: Displacement noise (0.05 strength) adds surface detail, should look organic not mechanical

spec(
    asset_id = "stdlib-organic-blob-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/organic_blob.glb", "glb")],
    recipe = {
        "kind": "static_mesh.organic_sculpt_v1",
        "params": {
            "metaballs": [
                # Main body
                {
                    "position": [0.0, 0.0, 0.0],
                    "radius": 1.0,
                    "stiffness": 2.0
                },
                # Extension 1
                {
                    "position": [0.6, 0.0, 0.2],
                    "radius": 0.6,
                    "stiffness": 2.0
                },
                # Extension 2
                {
                    "position": [-0.4, 0.3, 0.1],
                    "radius": 0.5,
                    "stiffness": 2.5
                },
                # Small bump
                {
                    "position": [0.0, -0.4, 0.5],
                    "radius": 0.35,
                    "stiffness": 3.0
                }
            ],
            "remesh_voxel_size": 0.08,
            "smooth_iterations": 2,
            "displacement": {
                "strength": 0.05,
                "scale": 3.0,
                "octaves": 4,
                "seed": 42
            },
            "export": {
                "apply_modifiers": True,
                "triangulate": True,
                "include_normals": True,
                "include_uvs": True
            }
        }
    }
)
