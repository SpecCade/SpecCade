# Audio effects chain

`audio_v1` exposes a post-mix `effects` list; order matters. Layer filters and the post-mix chain are separate controls. Confirm exact helper fields/defaults with `speccade stdlib dump --format json` and `crates/speccade-spec/src/recipe/audio/effects/mod.rs`.

Reuse an existing effect recipe under `specs/audio/`. Do not infer fields such as `dry`, `voices`, `output_gain`, or filter-envelope shapes from another plugin's UI. A descriptive effect name is not its JSON contract.

- First render dry with gain headroom, then add one effect and compare.
- Distortion/bit reduction change timbre; compression changes dynamics; delay/reverb add tails; modulation changes movement/width. Choose for the asset's role, not for a large chain.
- Check finite, decodable output, clipping, tail truncation, loop boundaries and mono compatibility.
- Run validate/generate with an explicit root and target budget; listen before acceptance.

For ZX WAVs, preserve 22050 Hz mono signed 16-bit PCM. The downstream raw packer does not reliably convert other WAV formats. See `nethercore-zx-integration.md`.
