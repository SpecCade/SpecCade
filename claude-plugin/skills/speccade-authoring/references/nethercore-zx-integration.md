# Nethercore ZX Integration

SpecCade produces source artifacts. Nethercore acceptance is a separate path: `nether pack` reads the files declared in `nether.toml`, then the game must load and use the resulting IDs at runtime.

## Manifest inputs

Declare raw generated files with an explicit table ID and path. Paths are relative to the project directory:

```toml
[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.sounds]]
id = "jump"
path = "sounds/jump.wav"

[[assets.trackers]]
id = "music"
path = "music/level.xm"

[[assets.meshes]]
id = "player_mesh"
path = "meshes/player.glb"

[[assets.skeletons]]
id = "player_skeleton"
path = "meshes/player.glb"
skin_name = "CharacterSkin"

[[assets.keyframes]]
id = "player_idle"
path = "meshes/player.glb"
animation_name = "Idle"
skin_name = "CharacterSkin"
```

`assets.animations` is an alias for `assets.keyframes`. Trackers may be XM or IT. Mesh, skeleton, and keyframe loaders accept GLB/glTF; the latter two can select a named skin or animation. Do not use the old standalone `.nczxtex`/`.nczxsnd` files as PNG/WAV loader inputs, and do not confuse this `nether.toml` with a deprecated exporter `assets.toml`.

## Audio contract

For ZX sounds, make the source WAV **22,050 Hz, mono, signed 16-bit PCM** before packing. The current Nethercore packer checks RIFF/WAVE and reads the `data` bytes as little-endian `i16`; its loader does not inspect the WAV `fmt` fields or resample/downmix, despite the conversion comment. A successful load therefore does not prove that an incorrectly formatted source will play correctly.

Nethercore first extracts embedded XM/IT samples, converts them through its tracker conversion path, and merges them with explicit sounds. Non-empty instrument references must resolve in that combined sound map; separate WAV declarations are not always required. Prefer stable lowercase IDs; inspect naming/deduplication diagnostics and reject conflicting same-ID content. Explicit WAV sources still need 22,050 Hz mono signed 16-bit PCM. See `tools/nether-cli/src/pack/assets/mod.rs` and `audio.rs` in Nethercore.

## GLB selection and limits

The current GLB mesh converter reads the **first mesh's first primitive**. Keep the intended runtime mesh there, or provide a separate GLB. For skeletons and animations, specify `skin_name` and `animation_name` when the intended item is not first; otherwise the loader selects the first matching item.

Nethercore export limits are not the same for every file:

- Skeleton export: at most **256 bones**.
- Animation export: at most **255 bones** and 65,535 sampled frames.

Inspect the generated GLB and the selected skin/animation instead of treating Blender generation success as import success.

## Verification boundary

Use separate gates, in order:

1. `speccade validate --spec FILE --budget nethercore` before generation; generate into an explicit output root and inspect the reported files.
2. Confirm source formats and paths, including WAV metadata and tracker sample IDs.
3. Run `nether pack` with the real `nether.toml`; this verifies manifest parsing and asset loaders, not game behavior.
4. Run the game and verify initialization resolves the expected asset IDs/handles.
5. Exercise the runtime path: render the texture/mesh, play the sound/tracker, or play the animation. Record runtime failures separately from generation and packing failures.

Generation success, budget compliance, pack/load success, initialization, runtime behavior, and human quality are different claims; report only the gates that were actually exercised.
