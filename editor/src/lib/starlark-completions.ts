/**
 * Monaco autocomplete provider for SpecCade Starlark stdlib functions.
 *
 * Provides IntelliSense-style completions for all stdlib functions including:
 * - Core functions (spec, output)
 * - Audio synthesis, filters, effects, modulation, and layers
 * - Texture nodes and graphs
 * - Mesh primitives and modifiers
 * - Music/tracker functions
 */
import * as monaco from "monaco-editor";
import { STARLARK_LANGUAGE_ID } from "./starlark-language";

/**
 * Stdlib completion item with snippet support.
 */
interface StdlibCompletion {
  label: string;
  kind: monaco.languages.CompletionItemKind;
  insertText: string;
  insertTextRules: monaco.languages.CompletionItemInsertTextRule;
  documentation: string;
  detail: string;
}

/**
 * All SpecCade stdlib function completions.
 */
const STDLIB_COMPLETIONS: StdlibCompletion[] = [
  // ============================================================
  // Core Functions
  // ============================================================
  {
    label: "spec",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'spec(\n\tasset_id = "${1:my-asset}",\n\tasset_type = "${2|audio,texture,static_mesh,skeletal_mesh,skeletal_animation,music|}",\n\tseed = ${3:42},\n\toutputs = [${4}],\n\trecipe = {${5}}\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Create a complete spec dictionary with all required fields",
    detail: "(asset_id, asset_type, seed, outputs, recipe) -> dict",
  },
  {
    label: "output",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'output("${1:path/to/file}", "${2|wav,png,glb,xm,it|}")',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Create an output specification with path and format",
    detail: "(path, format) -> dict",
  },

  // ============================================================
  // Audio Synthesis Functions
  // ============================================================
  {
    label: "oscillator",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'oscillator(${1:440}, "${2|sine,square,sawtooth,triangle|}", ${3:220}, "${4|linear,exponential|}")',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation:
      "Basic oscillator synthesis with frequency, waveform, and pitch sweep",
    detail: "(freq, waveform, end_freq, sweep) -> dict",
  },
  {
    label: "envelope",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "envelope(${1:0.01}, ${2:0.1}, ${3:0.7}, ${4:0.3})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "ADSR envelope (attack, decay, sustain level, release)",
    detail: "(attack, decay, sustain, release) -> dict",
  },
  {
    label: "audio_layer",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      "audio_layer(\n\t${1:oscillator(440, \"sine\")},\n\t${2:envelope(0.01, 0.1, 0.7, 0.3)},\n\tfilter = ${3:None},\n\tlfo = ${4:None}\n)",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation:
      "Complete audio synthesis layer with synthesis, envelope, optional filter and LFO",
    detail: "(synthesis, envelope, filter?, lfo?) -> dict",
  },
  {
    label: "fm_synth",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      "fm_synth(${1:440}, ${2:2.0}, ${3:500}, ${4:0.5}, ${5:0.3})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "FM synthesis with carrier frequency, ratio, depth, feedback, and decay",
    detail: "(carrier_freq, ratio, mod_depth, feedback, decay) -> dict",
  },
  {
    label: "granular",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'granular("${1|sine,square,sawtooth,triangle|}", ${2:0.05}, ${3:20}, ${4:0.3}, ${5:440})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Granular synthesis with grain parameters",
    detail: "(source, grain_size, density, randomness, freq) -> dict",
  },
  {
    label: "karplus_strong",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "karplus_strong(${1:440}, ${2:0.5}, ${3:0.99})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Karplus-Strong plucked string synthesis",
    detail: "(freq, brightness, decay) -> dict",
  },
  {
    label: "noise_burst",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'noise_burst("${1|white,pink,brown|}", ${2:0.05}, ${3:1.0})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Noise burst synthesis for percussion sounds",
    detail: "(noise_type, burst_time, amplitude) -> dict",
  },

  // ============================================================
  // Audio Filter Functions
  // ============================================================
  {
    label: "lowpass",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "lowpass(${1:5000}, ${2:0.707}, ${3:500})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Lowpass filter with cutoff, resonance, and sweep target",
    detail: "(cutoff, resonance, end_cutoff?) -> dict",
  },
  {
    label: "highpass",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "highpass(${1:200}, ${2:0.707})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Highpass filter with cutoff and resonance",
    detail: "(cutoff, resonance) -> dict",
  },
  {
    label: "bandpass",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "bandpass(${1:1000}, ${2:2.0})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Bandpass filter with center frequency and Q",
    detail: "(center_freq, q) -> dict",
  },
  {
    label: "ladder",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "ladder(${1:2000}, ${2:0.8}, ${3:4})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Moog-style ladder filter",
    detail: "(cutoff, resonance, poles) -> dict",
  },

  // ============================================================
  // Audio Effect Functions
  // ============================================================
  {
    label: "reverb",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "reverb(${1:0.3}, ${2:0.5}, ${3:0.5})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Reverb effect with mix, room size, and damping",
    detail: "(mix, room_size, damping) -> dict",
  },
  {
    label: "delay",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "delay(${1:0.25}, ${2:0.4}, ${3:0.3})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Delay effect with time, feedback, and mix",
    detail: "(delay_time, feedback, mix) -> dict",
  },
  {
    label: "chorus",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "chorus(${1:1.0}, ${2:0.003}, ${3:0.5})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Chorus effect with rate, depth, and mix",
    detail: "(rate, depth, mix) -> dict",
  },
  {
    label: "compressor",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "compressor(${1:-20}, ${2:4.0}, ${3:0.01}, ${4:0.1})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Compressor with threshold, ratio, attack, and release",
    detail: "(threshold, ratio, attack, release) -> dict",
  },
  {
    label: "bitcrush",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "bitcrush(${1:8}, ${2:0.5})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Bitcrusher for lo-fi effects",
    detail: "(bits, mix) -> dict",
  },

  // ============================================================
  // Audio Modulation Functions
  // ============================================================
  {
    label: "lfo",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'lfo("${1|sine,square,triangle,sawtooth|}", ${2:5.0}, ${3:0.5})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "LFO configuration with waveform, rate, and depth",
    detail: "(waveform, rate, depth) -> dict",
  },
  {
    label: "lfo_modulation",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'lfo_modulation("${1|filter_cutoff,amplitude,pitch|}", "${2|sine,triangle|}", ${3:5.0}, ${4:0.5})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "LFO modulation with target parameter",
    detail: "(target, waveform, rate, depth) -> dict",
  },

  // ============================================================
  // Texture Node Functions
  // ============================================================
  {
    label: "noise_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'noise_node("${1:noise}", "${2|perlin,simplex,voronoi,white|}", ${3:0.05}, ${4:6})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Noise texture node with algorithm, scale, and octaves",
    detail: "(id, algorithm, scale, octaves) -> dict",
  },
  {
    label: "gradient_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'gradient_node("${1:gradient}", "${2|linear,radial,angular|}", ${3:0.0})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Gradient node with type and angle",
    detail: "(id, gradient_type, angle) -> dict",
  },
  {
    label: "color_ramp_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'color_ramp_node("${1:colored}", "${2:input_id}", ["${3:#000000}", "${4:#ffffff}"])',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Color ramp mapping from grayscale to colors",
    detail: "(id, input, colors) -> dict",
  },
  {
    label: "threshold_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'threshold_node("${1:mask}", "${2:input_id}", ${3:0.5})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Threshold operation for creating masks",
    detail: "(id, input, threshold) -> dict",
  },
  {
    label: "checkerboard_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'checkerboard_node("${1:checker}", ${2:8})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Checkerboard pattern with tile count",
    detail: "(id, tiles) -> dict",
  },
  {
    label: "stripes_node",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'stripes_node("${1:stripes}", ${2:0.1}, ${3:0.0})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Stripe pattern with frequency and angle",
    detail: "(id, frequency, angle) -> dict",
  },

  // ============================================================
  // Texture Graph Function
  // ============================================================
  {
    label: "texture_graph",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      "texture_graph(\n\t[${1:256}, ${2:256}],\n\t[\n\t\t${3:noise_node(\"base\", \"perlin\", 0.05, 6)},\n\t\t${4:color_ramp_node(\"colored\", \"base\", [\"#000000\", \"#ffffff\"])}\n\t]\n)",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete texture graph with dimensions and nodes",
    detail: "(dimensions, nodes) -> dict",
  },

  // ============================================================
  // Mesh Primitive and Recipe Functions
  // ============================================================
  {
    label: "mesh_primitive",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'mesh_primitive("${1|cube,sphere,cylinder,cone,torus,plane|}", [${2:1.0}, ${3:1.0}, ${4:1.0}])',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Base mesh primitive with type and dimensions",
    detail: "(primitive_type, dimensions) -> dict",
  },
  {
    label: "mesh_recipe",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'mesh_recipe(\n\t"${1|cube,sphere,cylinder,cone,torus,plane|}",\n\t[${2:1.0}, ${3:1.0}, ${4:1.0}],\n\t[${5}]\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete mesh recipe with primitive, dimensions, and modifiers",
    detail: "(primitive, dimensions, modifiers) -> dict",
  },

  // ============================================================
  // Mesh Modifier Functions
  // ============================================================
  {
    label: "bevel_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "bevel_modifier(${1:0.1}, ${2:3})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Bevel modifier with width and segments",
    detail: "(width, segments) -> dict",
  },
  {
    label: "subdivision_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "subdivision_modifier(${1:2})",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Subdivision surface modifier with level",
    detail: "(level) -> dict",
  },
  {
    label: "decimate_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'decimate_modifier(${1:0.5}, "${2|collapse,unsubdiv,dissolve|}")',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Decimate modifier to reduce polygon count",
    detail: "(ratio, mode) -> dict",
  },
  {
    label: "mirror_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'mirror_modifier("${1|x,y,z|}", ${2:True})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Mirror modifier along an axis",
    detail: "(axis, merge) -> dict",
  },
  {
    label: "array_modifier",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: "array_modifier(${1:3}, [${2:1.0}, ${3:0.0}, ${4:0.0}])",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Array modifier for duplicating geometry",
    detail: "(count, offset) -> dict",
  },
  {
    label: "baking_settings",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'baking_settings(\n\t["${1|normal,ao,curvature|}"],\n\tray_distance = ${2:0.1},\n\tmargin = ${3:16},\n\tresolution = [${4:1024}, ${5:1024}]\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Texture map baking settings",
    detail: "(maps, ray_distance, margin, resolution) -> dict",
  },

  // ============================================================
  // Music/Tracker Functions
  // ============================================================
  {
    label: "tracker_instrument",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'tracker_instrument(\n\tname = "${1:lead}",\n\tsynthesis = ${2:instrument_synthesis("sawtooth")}\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Tracker instrument definition",
    detail: "(name, synthesis) -> dict",
  },
  {
    label: "instrument_synthesis",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'instrument_synthesis("${1|sine,square,sawtooth,triangle|}")',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Tracker instrument synthesis configuration",
    detail: "(waveform) -> dict",
  },
  {
    label: "pattern_note",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText: 'pattern_note(${1:0}, "${2:C4}", ${3:0})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Pattern note event with row, note, and instrument",
    detail: "(row, note, instrument_index) -> dict",
  },
  {
    label: "tracker_pattern",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'tracker_pattern(${1:64}, notes = {\n\t"0": [\n\t\t${2:pattern_note(0, "C4", 0)}\n\t]\n})',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Tracker pattern with rows and channel notes",
    detail: "(rows, notes) -> dict",
  },
  {
    label: "music_spec",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'music_spec(\n\tasset_id = "${1:song-01}",\n\tseed = ${2:42},\n\toutput_path = "${3:music/song.xm}",\n\tformat = "${4|xm,it|}",\n\tbpm = ${5:120},\n\tspeed = ${6:6},\n\tchannels = ${7:4},\n\tinstruments = [${8}],\n\tpatterns = {${9}},\n\tarrangement = [${10}]\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete music spec wrapper",
    detail: "(asset_id, seed, output_path, ...) -> dict",
  },

  // ============================================================
  // Character/Skeletal Mesh Functions
  // ============================================================
  {
    label: "skeletal_mesh_spec",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'skeletal_mesh_spec(\n\tasset_id = "${1:character-01}",\n\tseed = ${2:42},\n\toutput_path = "${3:characters/model.glb}",\n\tformat = "glb",\n\tskeleton_preset = "${4|humanoid_basic_v1,quadruped_basic_v1|}",\n\tbody_parts = [${5}],\n\tmaterial_slots = [${6}]\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete skeletal mesh spec",
    detail: "(asset_id, seed, output_path, ...) -> dict",
  },
  {
    label: "body_part",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'body_part(\n\tbone = "${1:chest}",\n\tprimitive = "${2|cylinder,sphere,cube|}",\n\tdimensions = [${3:0.3}, ${4:0.3}, ${5:0.3}],\n\tsegments = ${6:8},\n\tmaterial_index = ${7:0}\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Attach mesh primitive to skeleton bone",
    detail: "(bone, primitive, dimensions, ...) -> dict",
  },
  {
    label: "material_slot",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'material_slot(name = "${1:body}", base_color = [${2:0.8}, ${3:0.6}, ${4:0.5}, ${5:1.0}])',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Material slot definition with name and color",
    detail: "(name, base_color) -> dict",
  },

  // ============================================================
  // Animation Functions
  // ============================================================
  {
    label: "skeletal_animation_spec",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'skeletal_animation_spec(\n\tasset_id = "${1:anim-01}",\n\tseed = ${2:42},\n\toutput_path = "${3:animations/clip.glb}",\n\tformat = "glb",\n\tskeleton_preset = "${4:humanoid_basic_v1}",\n\tclip_name = "${5:idle}",\n\tduration_seconds = ${6:1.0},\n\tfps = ${7:24},\n\tloop = ${8:True},\n\tkeyframes = [${9}]\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Complete skeletal animation spec",
    detail: "(asset_id, seed, output_path, ...) -> dict",
  },
  {
    label: "animation_keyframe",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      'animation_keyframe(\n\ttime = ${1:0.0},\n\tbones = {\n\t\t"${2:spine}": ${3:bone_transform(rotation = [0.0, 0.0, 0.0])}\n\t}\n)',
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Animation keyframe with bone transforms",
    detail: "(time, bones) -> dict",
  },
  {
    label: "bone_transform",
    kind: monaco.languages.CompletionItemKind.Function,
    insertText:
      "bone_transform(\n\tposition = [${1:0.0}, ${2:0.0}, ${3:0.0}],\n\trotation = [${4:0.0}, ${5:0.0}, ${6:0.0}],\n\tscale = [${7:1.0}, ${8:1.0}, ${9:1.0}]\n)",
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: "Bone position, rotation, and scale transform",
    detail: "(position, rotation, scale) -> dict",
  },
];

/**
 * Register Starlark stdlib completions with Monaco.
 *
 * Call this once during editor initialization to enable
 * autocomplete for SpecCade stdlib functions.
 */
export function registerStarlarkCompletions(): void {
  monaco.languages.registerCompletionItemProvider(STARLARK_LANGUAGE_ID, {
    provideCompletionItems: (
      model: monaco.editor.ITextModel,
      position: monaco.Position
    ): monaco.languages.ProviderResult<monaco.languages.CompletionList> => {
      const word = model.getWordUntilPosition(position);
      const range: monaco.IRange = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      const suggestions: monaco.languages.CompletionItem[] =
        STDLIB_COMPLETIONS.map((item) => ({
          label: item.label,
          kind: item.kind,
          insertText: item.insertText,
          insertTextRules: item.insertTextRules,
          documentation: {
            value: item.documentation,
          },
          detail: item.detail,
          range,
        }));

      return { suggestions };
    },
  });
}
