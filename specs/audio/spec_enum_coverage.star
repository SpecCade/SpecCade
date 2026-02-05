# Enum coverage examples - demonstrates output/spec enum parameters
#
# This file exercises various enum values for format, kind, asset_type, and loop_mode
# to ensure comprehensive coverage of the stdlib API.
#
# Covered enums:
# - format: wav, png, glb, gltf, json, xm, it
# - kind: primary, metadata, preview
# - asset_type: audio, music, texture, sprite, vfx, ui, font, static_mesh, skeletal_mesh, skeletal_animation
# - loop_mode: auto, none, forward, pingpong (in tracker instruments)

# === Audio spec example (format::wav, asset_type::audio) ===
spec(
    asset_id = "enum-coverage-audio-01",
    asset_type = "audio",
    seed = 99101,
    outputs = [
        output("audio/enum_coverage.wav", "wav", kind = "primary"),
        output("audio/enum_coverage.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sine"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    volume = 0.8
                )
            ]
        }
    },
    description = "Audio enum coverage - wav format, primary and metadata outputs"
)

# === Music spec with loop_mode variants ===
# Using tracker_instrument with different loop_mode values
spec(
    asset_id = "enum-coverage-music-01",
    asset_type = "music",
    seed = 99102,
    description = "Music enum coverage - loop_mode variants (auto, none, forward, pingpong)",
    outputs = [output("music/enum_coverage_loops.xm", "xm")],
    recipe = {
        "kind": "music.tracker_v1",
        "params": {
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "instruments": [
                tracker_instrument(
                    name = "inst_auto",
                    synthesis = instrument_synthesis("sine"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    loop_mode = "auto"
                ),
                tracker_instrument(
                    name = "inst_none",
                    synthesis = instrument_synthesis("triangle"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    loop_mode = "none"
                ),
                tracker_instrument(
                    name = "inst_forward",
                    synthesis = instrument_synthesis("sawtooth"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    loop_mode = "forward"
                ),
                tracker_instrument(
                    name = "inst_pingpong",
                    synthesis = instrument_synthesis("pulse", 0.5),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    loop_mode = "pingpong"
                )
            ],
            "patterns": {
                "main": tracker_pattern(64, notes = {
                    "0": [pattern_note(0, "C4", 0)],
                    "1": [pattern_note(0, "E4", 1)],
                    "2": [pattern_note(0, "G4", 2)],
                    "3": [pattern_note(0, "C5", 3)]
                })
            },
            "arrangement": [arrangement_entry("main", 2)]
        }
    }
)

# === Texture spec example (format::png, asset_type::texture) ===
spec(
    asset_id = "enum-coverage-texture-01",
    asset_type = "texture",
    seed = 99103,
    outputs = [
        output("textures/enum_coverage.png", "png", kind = "primary"),
        output("textures/enum_coverage_preview.png", "png", kind = "preview")
    ],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [noise_node("height", "perlin", 0.1, 4, 0.5, 2.0)],
            True
        )
    },
    description = "Texture enum coverage - png format, primary and preview outputs"
)

# === Static mesh spec example (format::glb, format::gltf, asset_type::static_mesh) ===
spec(
    asset_id = "enum-coverage-mesh-glb-01",
    asset_type = "static_mesh",
    seed = 99104,
    outputs = [output("meshes/enum_coverage.glb", "glb", kind = "primary")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe("cube", [1.0, 1.0, 1.0], [])
    },
    description = "Static mesh enum coverage - glb format"
)

spec(
    asset_id = "enum-coverage-mesh-gltf-01",
    asset_type = "static_mesh",
    seed = 99105,
    outputs = [output("meshes/enum_coverage.gltf", "gltf", kind = "primary")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe("sphere", [0.5, 0.5, 0.5], [])
    },
    description = "Static mesh enum coverage - gltf format"
)

# === Sprite spec example (asset_type::sprite) ===
spec(
    asset_id = "enum-coverage-sprite-01",
    asset_type = "sprite",
    seed = 99106,
    outputs = [
        output("sprites/enum_coverage.png", "png", kind = "primary"),
        output("sprites/enum_coverage.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "sprite.sheet_v1",
        "params": {
            "resolution": [128, 128],
            "padding": 2,
            "frames": [
                {"id": "frame_0", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [1.0, 0.0, 0.0, 1.0]},
                {"id": "frame_1", "width": 32, "height": 32, "pivot": [0.5, 1.0], "color": [0.0, 1.0, 0.0, 1.0]}
            ]
        }
    },
    description = "Sprite enum coverage - sprite asset_type, png and json outputs"
)

# === VFX spec example (asset_type::vfx) ===
spec(
    asset_id = "enum-coverage-vfx-01",
    asset_type = "vfx",
    seed = 99107,
    outputs = [
        output("vfx/enum_coverage.png", "png", kind = "primary"),
        output("vfx/enum_coverage.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.particle_profile_v1",
        "params": {
            "profile": "additive",
            "color_tint": [1.0, 0.5, 0.2],
            "intensity": 1.0
        }
    },
    description = "VFX enum coverage - vfx asset_type"
)

# === UI spec example (asset_type::ui) ===
spec(
    asset_id = "enum-coverage-ui-01",
    asset_type = "ui",
    seed = 99108,
    outputs = [
        output("ui/enum_coverage.png", "png", kind = "primary"),
        output("ui/enum_coverage.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "ui.nine_slice_v1",
        "params": {
            "resolution": [64, 64],
            "border": 8,
            "corner_radius": 4,
            "fill_color": [0.2, 0.2, 0.2, 1.0],
            "border_color": [0.8, 0.8, 0.8, 1.0]
        }
    },
    description = "UI enum coverage - ui asset_type"
)

# === Skeletal mesh spec example (format::glb, asset_type::skeletal_mesh) ===
spec(
    asset_id = "enum-coverage-skeletal-01",
    asset_type = "skeletal_mesh",
    seed = 99109,
    description = "Skeletal mesh enum coverage - skeletal_mesh asset_type, glb format",
    outputs = [output("characters/enum_coverage.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "bone_meshes": {
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": 0.1,
                    "material_index": 0
                }
            },
            "material_slots": [
                material_slot(name = "body", base_color = [0.8, 0.6, 0.5, 1.0])
            ]
        }
    }
)

# === Skeletal animation spec example (asset_type::skeletal_animation) ===
spec(
    asset_id = "enum-coverage-anim-01",
    asset_type = "skeletal_animation",
    seed = 99110,
    description = "Skeletal animation enum coverage - skeletal_animation asset_type",
    outputs = [output("animations/enum_coverage.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "idle",
            "duration_seconds": 1.0,
            "fps": 30,
            "loop": False,
            "interpolation": "linear",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]}
                    }
                }
            ]
        }
    }
)
