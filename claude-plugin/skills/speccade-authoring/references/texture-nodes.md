# Procedural texture nodes

Use the current node helpers rather than maintaining a duplicate JSON node catalogue. Source authority: `crates/speccade-spec/src/recipe/texture/procedural.rs`; discover helper signatures with `speccade stdlib dump --format json`. `docs/spec-reference/texture.md` and `specs/texture/` provide broader examples.

Adapted from `specs/texture/texture_colored.star`, without its unused gradient node:

```starlark
spec(
    asset_id = "zx-texture-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/colored.png", "png", source = "colored")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph([128, 128], [
            noise_node("n", "simplex", 0.08, 6),
            color_ramp_node("colored", "n", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"]),
        ]),
    },
)
```

`outputs[].source` is a terminal **node ID**, not a reserved material name. Ensure every referenced node exists and all intended outputs select the right terminal nodes. Validate the graph, generate to an explicit root, decode the PNG and inspect the actual material in the target renderer.

For ZX, start small, account for the4MiB loaded VRAM budget and actual compression settings. Normal-mapped assets need the correct UV/tangent/material pipeline; a normal-looking PNG does not establish correct shading. See [ZX integration](nethercore-zx-integration.md).
