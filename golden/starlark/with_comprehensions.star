# Starlark spec demonstrating list comprehensions
# Shows how to generate multiple outputs or variants programmatically.

# Generate output specs for multiple file names
file_names = ["kick", "snare", "hihat", "tom"]

outputs = [
    {
        "kind": "primary",
        "format": "wav",
        "path": "drums/" + name + ".wav"
    }
    for name in file_names
]

# Generate style tags by transforming a list
raw_tags = ["drum", "percussion", "kit"]
style_tags = [tag.upper() for tag in raw_tags]

{
    "spec_version": 1,
    "asset_id": "starlark-comprehension-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 999,
    "description": "Drum kit generated using list comprehensions",
    "style_tags": style_tags,
    "outputs": outputs
}
