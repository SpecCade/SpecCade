# Trimsheet / atlas golden spec
#
# Exercises the texture.trimsheet_v1 recipe with shelf packing, gutter padding,
# and UV metadata output. Uses a mix of tile sizes and both color and node_ref
# source types.

tile_stone = trimsheet_tile(
    id = "stone_floor",
    width = 128,
    height = 128,
    color = [0.45, 0.42, 0.40, 1.0]
)

tile_brick = trimsheet_tile(
    id = "brick_wall",
    width = 256,
    height = 64,
    color = [0.65, 0.30, 0.22, 1.0]
)

tile_trim = trimsheet_tile(
    id = "metal_trim",
    width = 64,
    height = 256,
    color = [0.55, 0.55, 0.58, 1.0]
)

tile_plaster = trimsheet_tile(
    id = "plaster",
    width = 128,
    height = 128,
    color = [0.85, 0.82, 0.78, 1.0]
)

trimsheet_spec(
    asset_id = "golden-texture-trimsheet-01",
    seed = 1001,
    output_path = "textures/trimsheet_building.png",
    resolution = [512, 512],
    tiles = [tile_stone, tile_brick, tile_trim, tile_plaster],
    padding = 2,
    description = "Building-material trimsheet: tests shelf packing with varied tile sizes and gutter padding",
    tags = ["golden", "trimsheet", "building"]
)
