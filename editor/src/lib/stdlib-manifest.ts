import type * as monaco from "monaco-editor";

export type StdlibCategoryId = "core" | "audio" | "texture" | "mesh" | "music";

export interface StdlibFunction {
  name: string;
  signature: string;
  description: string;
  /**
   * Snippet inserted by both the snippets palette and Monaco completions.
   * Keep this plain-text (no Monaco tabstops) until the palette supports snippet insertion.
   */
  snippet: string;
  completionKind?: monaco.languages.CompletionItemKind;
}

export interface StdlibCategory {
  id: StdlibCategoryId;
  name: string;
  icon: string;
  functions: StdlibFunction[];
}

// Single source of truth for stdlib snippets + completions.
export const STDLIB_MANIFEST: StdlibCategory[] = [
  {
    id: "core",
    name: "Core",
    icon: "\u2699\uFE0F",
    functions: [
      {
        name: "spec",
        signature:
          'spec(asset_id, asset_type, seed, outputs, recipe=None, description=None, tags=None, license="CC0-1.0") -> dict',
        description: "Create a complete spec dictionary with all required fields",
        snippet: `spec(
  asset_id = "my-asset",
  asset_type = "audio",
  seed = 42,
  outputs = [output("path/to/file.wav", "wav")],
  recipe = {},
)`,
      },
      {
        name: "output",
        signature: 'output(path, format, kind="primary", source=None) -> dict',
        description: "Create an output specification with path and format",
        snippet: 'output("path/to/file.wav", "wav")',
      },
    ],
  },
  {
    id: "audio",
    name: "Audio",
    icon: "\uD83D\uDD0A",
    functions: [
      {
        name: "oscillator",
        signature:
          'oscillator(frequency, waveform="sine", sweep_to=None, curve="linear", detune=None, duty=None) -> dict',
        description:
          "Basic oscillator synthesis with frequency, waveform, and pitch sweep",
        snippet: 'oscillator(440, waveform = "sine", sweep_to = 220, curve = "linear")',
      },
      {
        name: "fm_synth",
        signature:
          "fm_synth(carrier, modulator, index, sweep_to=None) -> dict",
        description:
          "FM synthesis with carrier/modulator freqs and modulation index",
        snippet: 'fm_synth(440, 220, 4.0, sweep_to = 330)',
      },
      {
        name: "envelope",
        signature: "envelope(attack=0.01, decay=0.1, sustain=0.5, release=0.2) -> dict",
        description: "ADSR envelope (attack, decay, sustain level, release)",
        snippet: 'envelope(attack = 0.01, decay = 0.1, sustain = 0.5, release = 0.2)',
      },
      {
        name: "audio_layer",
        signature:
          "audio_layer(synthesis, envelope=None, volume=0.8, pan=0.0, filter=None, lfo=None, delay=None) -> dict",
        description:
          "Complete audio synthesis layer with synthesis, envelope, optional filter and LFO",
        snippet: `audio_layer(
  oscillator(440, "sine", 440, "linear"),
  envelope(0.01, 0.1, 0.7, 0.3),
  filter = lowpass(5000, resonance = 0.707, sweep_to = 500),
  lfo = None,
)`,
      },
      {
        name: "lowpass",
        signature: "lowpass(cutoff, resonance=0.707, sweep_to=None) -> dict",
        description: "Lowpass filter with cutoff, resonance, and sweep target",
        snippet: 'lowpass(5000, resonance = 0.707, sweep_to = 500)',
      },
      {
        name: "highpass",
        signature: "highpass(cutoff, resonance=0.707, sweep_to=None) -> dict",
        description: "Highpass filter with cutoff and resonance",
        snippet: 'highpass(200, resonance = 0.707)',
      },
      {
        name: "reverb",
        signature: "reverb(decay=0.5, wet=0.3, room_size=0.8, width=1.0) -> dict",
        description: "Reverb effect with mix, room size, and damping",
        snippet: 'reverb(decay = 0.6, wet = 0.25, room_size = 0.8, width = 1.0)',
      },
    ],
  },
  {
    id: "texture",
    name: "Texture",
    icon: "\uD83C\uDFA8",
    functions: [
      {
        name: "noise_node",
        signature:
          "noise_node(id, algorithm=\"perlin\", scale=0.1, octaves=4, persistence=0.5, lacunarity=2.0) -> dict",
        description: "Noise texture node with algorithm, scale, and octaves",
        snippet: 'noise_node("noise", "perlin", 0.05, 6)',
      },
      {
        name: "color_ramp_node",
        signature: "color_ramp_node(id, input, ramp) -> dict",
        description: "Color ramp mapping from grayscale to colors",
        snippet:
          'color_ramp_node("colored", "input_id", ["#000000", "#ffffff"])',
      },
      {
        name: "texture_graph",
        signature: "texture_graph(resolution, nodes, tileable=True) -> dict",
        description: "Complete texture graph with dimensions and nodes",
        snippet: `texture_graph(
  [256, 256],
  [
    noise_node("base", "perlin", 0.05, 6),
    color_ramp_node("out", "base", ["#000000", "#ffffff"]),
  ],
)`,
      },
    ],
  },
  {
    id: "mesh",
    name: "Mesh",
    icon: "\uD83D\uDCE6",
    functions: [
      {
        name: "mesh_recipe",
        signature: "mesh_recipe(primitive, dimensions, modifiers=None) -> dict",
        description: "Complete mesh recipe with primitive, dimensions, and modifiers",
        snippet: `mesh_recipe(
  "cube",
  [1.0, 1.0, 1.0],
  [bevel_modifier(0.1, 3)],
)`,
      },
      {
        name: "bevel_modifier",
        signature: "bevel_modifier(width=0.02, segments=2, angle_limit=None) -> dict",
        description: "Bevel modifier with width and segments",
        snippet: 'bevel_modifier(0.1, 3)',
      },
      {
        name: "subdivision_modifier",
        signature: "subdivision_modifier(levels=2, render_levels=None) -> dict",
        description: "Subdivision surface modifier with level",
        snippet: 'subdivision_modifier(2)',
      },
    ],
  },
  {
    id: "music",
    name: "Music",
    icon: "\uD83C\uDFB5",
    functions: [
      {
        name: "instrument_synthesis",
        signature: "instrument_synthesis(synth_type, duty_cycle=0.5, periodic=False) -> dict",
        description: "Create a tracker instrument synthesis configuration",
        snippet: 'instrument_synthesis("pulse", duty_cycle = 0.25)',
      },
      {
        name: "tracker_instrument",
        signature:
          "tracker_instrument(name=..., synthesis=None, wav=None, ref=None, base_note=None, sample_rate=None, envelope=None, loop_mode=None, default_volume=None, comment=None) -> dict",
        description: "Define a tracker instrument (synth, wav, or ref source)",
        snippet: `tracker_instrument(
  name = "lead",
  synthesis = instrument_synthesis("sawtooth"),
)`,
      },
      {
        name: "pattern_note",
        signature:
          "pattern_note(row, note, inst, channel=None, vol=None, effect=None, param=None, effect_name=None, effect_xy=None) -> dict",
        description: "Create a note event for a tracker pattern",
        snippet: 'pattern_note(0, "C4", 0, vol = 48)',
      },
      {
        name: "tracker_pattern",
        signature: "tracker_pattern(rows, notes=None, data=None) -> dict",
        description: "Create a tracker pattern (channel-keyed notes or flat data)",
        snippet: `tracker_pattern(
  64,
  notes = {
    "0": [pattern_note(0, "C4", 0)],
  },
)`,
      },
      {
        name: "arrangement_entry",
        signature: "arrangement_entry(pattern, repeat=1) -> dict",
        description: "Create an order-table entry for the arrangement",
        snippet: 'arrangement_entry("intro", repeat = 4)',
      },
      {
        name: "volume_fade",
        signature:
          "volume_fade(pattern, channel, start_row, end_row, start_vol, end_vol) -> dict",
        description: "Automation: fade a channel volume over rows",
        snippet: 'volume_fade("intro", 0, 0, 63, 64, 0)',
      },
      {
        name: "tempo_change",
        signature: "tempo_change(pattern, row, bpm) -> dict",
        description: "Automation: change tempo at a specific row",
        snippet: 'tempo_change("bridge", 32, 140)',
      },
      {
        name: "humanize_vol",
        signature: "humanize_vol(min_vol, max_vol, seed_salt) -> dict",
        description: "Transform: deterministic per-note volume variation",
        snippet: 'humanize_vol(48, 64, "velocity")',
      },
      {
        name: "swing",
        signature: "swing(amount_permille, stride, seed_salt) -> dict",
        description: "Transform: apply swing feel via note delays",
        snippet: 'swing(500, 2, "groove")',
      },
      {
        name: "it_options",
        signature: "it_options(stereo=True, global_volume=128, mix_volume=48) -> dict",
        description: "Create IT module playback/mix options",
        snippet: 'it_options()',
      },
      {
        name: "tracker_song",
        signature:
          "tracker_song(format=..., bpm=..., speed=..., channels=..., instruments=..., patterns=..., arrangement=..., name=None, title=None, loop=False, restart_position=None, automation=None, it_options=None) -> dict",
        description: "Create a complete tracker song recipe",
        snippet: `tracker_song(
  format = "xm",
  bpm = 120,
  speed = 6,
  channels = 4,
  instruments = [
    tracker_instrument(name = "lead", synthesis = instrument_synthesis("sawtooth")),
  ],
  patterns = {
    "intro": tracker_pattern(64, notes = {"0": [pattern_note(0, "C4", 0)]}),
  },
  arrangement = [
    arrangement_entry("intro", repeat = 1),
  ],
)`,
      },
      {
        name: "music_spec",
        signature:
          "music_spec(asset_id=..., seed=..., output_path=..., format=..., bpm=..., speed=..., channels=..., instruments=..., patterns=..., arrangement=..., name=None, title=None, loop=False, description=None, tags=None, license=\"CC0-1.0\") -> dict",
        description: "Create a full music spec (asset + tracker song)",
        snippet: `music_spec(
  asset_id = "song-01",
  seed = 42,
  output_path = "music/song.xm",
  format = "xm",
  bpm = 120,
  speed = 6,
  channels = 4,
  instruments = [
    tracker_instrument(name = "lead", synthesis = instrument_synthesis("square")),
  ],
  patterns = {
    "intro": tracker_pattern(64, notes = {"0": [pattern_note(0, "C4", 0)]}),
  },
  arrangement = [
    arrangement_entry("intro", repeat = 1),
  ],
)`,
      },
      {
        name: "loop_low",
        signature:
          "loop_low(name=..., bpm=90, measures=8, rows_per_beat=4, channels=4, format=\"xm\") -> dict",
        description: "Cue template: low intensity looping music",
        snippet: 'loop_low(name = "explore_ambient")',
      },
      {
        name: "loop_main",
        signature:
          "loop_main(name=..., bpm=120, measures=8, rows_per_beat=4, channels=8, format=\"xm\") -> dict",
        description: "Cue template: main intensity looping music",
        snippet: 'loop_main(name = "gameplay_theme")',
      },
      {
        name: "loop_hi",
        signature:
          "loop_hi(name=..., bpm=140, measures=8, rows_per_beat=4, channels=12, format=\"xm\") -> dict",
        description: "Cue template: high intensity looping music",
        snippet: 'loop_hi(name = "boss_battle")',
      },
      {
        name: "stinger",
        signature:
          "stinger(name=..., stinger_type=\"custom\", duration_beats=4, bpm=120, rows_per_beat=4, channels=4, format=\"xm\", tail_beats=0) -> dict",
        description: "Cue template: one-shot musical event",
        snippet:
          'stinger(name = "coin_pickup", stinger_type = "pickup", duration_beats = 2)',
      },
      {
        name: "transition",
        signature:
          "transition(name=..., transition_type=\"bridge\", from_intensity=\"main\", to_intensity=\"main\", measures=2, bpm=120, rows_per_beat=4, channels=8, format=\"xm\") -> dict",
        description: "Cue template: musical transition between intensities",
        snippet:
          'transition(name = "to_combat", transition_type = "build", from_intensity = "main", to_intensity = "hi")',
      },
      {
        name: "loop_cue",
        signature:
          "loop_cue(name=..., intensity=..., bpm=None, measures=8, rows_per_beat=4, channels=None, format=\"xm\") -> dict",
        description: "Cue template: generic loop with explicit intensity",
        snippet: 'loop_cue(name = "ambient", intensity = "low", bpm = 80)',
      },
    ],
  },
];
