# Starlark spec demonstrating functions and variables
# Shows how Starlark can reduce repetition in spec definitions.

# Helper function to create an output specification
def make_output(name, format = "wav"):
    return {
        "kind": "primary",
        "format": format,
        "path": "sounds/" + name + "." + format
    }

# Variables for shared values
BASE_ID = "starlark-functions"
SEED = 12345
LICENSE = "CC0-1.0"

# Tags shared across related assets
COMMON_TAGS = ["retro", "scifi", "effects"]

# The spec definition uses the helper and variables
{
    "spec_version": 1,
    "asset_id": BASE_ID + "-01",
    "asset_type": "audio",
    "license": LICENSE,
    "seed": SEED,
    "description": "Audio asset defined using Starlark functions",
    "style_tags": COMMON_TAGS + ["laser"],
    "outputs": [
        make_output("laser_blast"),
        make_output("laser_charge")
    ]
}
