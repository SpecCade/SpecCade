# Music compose IR

Compose is the compact authoring layer for tracker music. Keep reusable patterns, arrangement and variation in the authored source instead of manually maintaining expanded note events.

Source authority: `crates/speccade-spec/src/recipe/music/compose.rs` and its related music modules. Confirm Starlark helpers with `speccade stdlib dump --format json`. Reuse `docs/examples/music/compose_eurobeat_4bars.json` when it fits, then validate it against the current executable; it is a starting example, not a style requirement.

```bash
speccade expand --spec song.json
speccade validate --spec song.json --budget nethercore --json
speccade generate --spec song.json --out-root ./output --budget nethercore --json
```

Here `nethercore` is the ZX target; choose another appropriate preset for other consumers. Inspect expanded events for instrument/channel references, ordering, timing and arrangement length before blaming audio rendering. Operator names, units and fields must come from the current contract—not an older operator catalogue.

Keep seed and inputs fixed while changing one musical dimension. Test generated XM/IT in the target runtime and listen to the complete transition/loop. See [tracker integration](music-tracker.md).
