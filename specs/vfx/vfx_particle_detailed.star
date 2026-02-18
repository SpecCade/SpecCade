# VFX particle profile golden spec
#
# Exercises vfx.particle_profile_v1 with different profile types, tints,
# and distortion settings.

# Distortion/refraction profile (heat haze)
spec(
    asset_id = "golden-vfx-particle-distort-01",
    asset_type = "vfx",
    seed = 6001,
    description = "Heat haze distortion particle profile with full distortion settings",
    tags = ["golden", "vfx", "particle", "distortion"],
    outputs = [
        output("vfx/particle_distort.json", "json")
    ],
    recipe = {
        "kind": "vfx.particle_profile_v1",
        "params": {
            "profile": "distort",
            "color_tint": [0.9, 0.95, 1.0],
            "intensity": 0.8,
            "distortion_strength": 0.6
        }
    }
)

# Soft/smoke particle profile
spec(
    asset_id = "golden-vfx-particle-soft-01",
    asset_type = "vfx",
    seed = 6002,
    description = "Soft smoke particle profile with muted tint",
    tags = ["golden", "vfx", "particle", "smoke"],
    outputs = [
        output("vfx/particle_soft_smoke.json", "json")
    ],
    recipe = {
        "kind": "vfx.particle_profile_v1",
        "params": {
            "profile": "soft",
            "color_tint": [0.7, 0.7, 0.75],
            "intensity": 0.6
        }
    }
)

# Screen blend profile (lens flare)
spec(
    asset_id = "golden-vfx-particle-screen-01",
    asset_type = "vfx",
    seed = 6003,
    description = "Screen blend lens flare particle profile",
    tags = ["golden", "vfx", "particle", "screen"],
    outputs = [
        output("vfx/particle_screen_flare.json", "json")
    ],
    recipe = {
        "kind": "vfx.particle_profile_v1",
        "params": {
            "profile": "screen",
            "color_tint": [1.0, 0.95, 0.8],
            "intensity": 2.0
        }
    }
)
