# VFX particle profile test
#
# Tests the vfx.particle_profile_v1 recipe with Additive profile.
# This is a metadata-only output (JSON) for particle rendering hints.

{
    "spec_version": 1,
    "asset_id": "vfx-fire-particles",
    "asset_type": "vfx",
    "seed": 42,
    "license": "CC0-1.0",
    "description": "Fire particle rendering profile with warm orange tint",
    "style_tags": ["vfx", "particles", "fire", "additive"],
    "outputs": [
        {
            "kind": "primary",
            "format": "json",
            "path": "vfx/fire_particles.json"
        }
    ],
    "recipe": {
        "kind": "vfx.particle_profile_v1",
        "params": {
            "profile": "additive",
            "color_tint": [1.0, 0.6, 0.2],
            "intensity": 1.5
        }
    }
}
