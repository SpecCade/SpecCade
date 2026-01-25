/**
 * Starlark token provider for Monaco editor syntax highlighting.
 *
 * This file contains the Monarch tokenizer rules that define how Starlark
 * source code is tokenized and colored in the editor.
 */
import type * as monaco from "monaco-editor";

/**
 * Starlark language keywords.
 *
 * These are the core control flow and definition keywords in Starlark.
 */
export const STARLARK_KEYWORDS = [
  "and",
  "break",
  "continue",
  "def",
  "elif",
  "else",
  "for",
  "if",
  "in",
  "lambda",
  "load",
  "not",
  "or",
  "pass",
  "return",
  "while",
];

/**
 * Starlark built-in constants and functions, plus SpecCade stdlib.
 *
 * This includes:
 * - Starlark built-in values: True, False, None
 * - Starlark built-in functions: len, range, str, etc.
 * - SpecCade core functions: spec, output
 * - SpecCade audio synthesis and effects
 * - SpecCade texture nodes and recipes
 * - SpecCade mesh primitives and modifiers
 * - SpecCade music/tracker functions
 */
export const STARLARK_BUILTINS = [
  // Starlark built-in values
  "True",
  "False",
  "None",

  // Starlark built-in functions
  "abs",
  "all",
  "any",
  "bool",
  "dict",
  "dir",
  "enumerate",
  "fail",
  "float",
  "getattr",
  "hasattr",
  "hash",
  "int",
  "len",
  "list",
  "max",
  "min",
  "print",
  "range",
  "repr",
  "reversed",
  "sorted",
  "str",
  "tuple",
  "type",
  "zip",

  // SpecCade Core
  "spec",
  "output",

  // Audio Synthesis
  "envelope",
  "oscillator",
  "fm_synth",
  "am_synth",
  "noise_burst",
  "karplus_strong",
  "additive",
  "supersaw_unison",
  "wavetable",
  "granular",
  "modal",
  "metallic",
  "vocoder",
  "formant_synth",
  "vector_synth",
  "waveguide",
  "bowed_string",
  "pulsar",
  "vosim",
  "spectral_freeze",
  "pitched_body",

  // Audio Filters
  "lowpass",
  "highpass",
  "bandpass",
  "notch",
  "allpass",
  "comb_filter",
  "formant_filter",
  "ladder",
  "shelf_low",
  "shelf_high",

  // Audio Effects
  "reverb",
  "delay",
  "compressor",
  "limiter",
  "chorus",
  "phaser",
  "flanger",
  "bitcrush",
  "waveshaper",
  "parametric_eq",
  "stereo_widener",
  "multi_tap_delay",
  "tape_saturation",
  "transient_shaper",
  "auto_filter",
  "cabinet_sim",
  "rotary_speaker",
  "ring_modulator",
  "granular_delay",

  // Audio Modulation
  "lfo",
  "lfo_modulation",
  "pitch_envelope",

  // Audio Layers & Helpers
  "audio_layer",
  "audio_spec",
  "oneshot_envelope",
  "loop_envelope",
  "with_loop_config",

  // Foley
  "impact_builder",
  "whoosh_builder",

  // Texture Nodes
  "noise_node",
  "gradient_node",
  "constant_node",
  "threshold_node",
  "invert_node",
  "color_ramp_node",
  "add_node",
  "multiply_node",
  "lerp_node",
  "clamp_node",
  "stripes_node",
  "checkerboard_node",
  "grayscale_node",
  "palette_node",
  "compose_rgba_node",
  "normal_from_height_node",
  "wang_tiles_node",
  "texture_bomb_node",

  // Texture Graph & Recipes
  "texture_graph",
  "texture_spec",
  "matcap_v1",
  "material_preset_v1",
  "decal_spec",
  "decal_metadata",
  "trimsheet_spec",
  "trimsheet_tile",
  "splat_set_spec",
  "splat_layer",

  // Mesh Primitives & Recipe
  "mesh_primitive",
  "mesh_recipe",

  // Mesh Modifiers
  "bevel_modifier",
  "subdivision_modifier",
  "decimate_modifier",
  "edge_split_modifier",
  "mirror_modifier",
  "array_modifier",
  "solidify_modifier",
  "triangulate_modifier",

  // Mesh Baking
  "baking_settings",

  // Character
  "body_part",
  "custom_bone",
  "material_slot",
  "skinning_config",
  "skeletal_export_settings",
  "skeletal_constraints",
  "skeletal_texturing",
  "skeletal_mesh_spec",

  // Animation
  "bone_transform",
  "ik_target_transform",
  "animation_keyframe",
  "ik_keyframe",
  "animation_export_settings",
  "skeletal_animation_spec",

  // Music Instruments
  "instrument_synthesis",
  "tracker_instrument",

  // Music Patterns
  "pattern_note",
  "tracker_pattern",
  "arrangement_entry",

  // Music Song
  "it_options",
  "volume_fade",
  "tempo_change",
  "tracker_song",
  "music_spec",
];

/**
 * Operators supported in Starlark.
 */
export const STARLARK_OPERATORS = [
  "=",
  ">",
  "<",
  "!",
  "~",
  "?",
  ":",
  "==",
  "<=",
  ">=",
  "!=",
  "&&",
  "||",
  "++",
  "--",
  "+",
  "-",
  "*",
  "/",
  "&",
  "|",
  "^",
  "%",
  "<<",
  ">>",
  ">>>",
  "+=",
  "-=",
  "*=",
  "/=",
  "&=",
  "|=",
  "^=",
  "%=",
  "<<=",
  ">>=",
  ">>>=",
];

/**
 * Monarch tokenizer for Starlark syntax highlighting.
 *
 * This tokenizer handles:
 * - Keywords (def, if, for, return, while, etc.)
 * - Built-in functions and SpecCade stdlib (highlighted as type.identifier)
 * - Strings (single, double, and triple-quoted)
 * - Numbers (integer, float, hex, octal, binary)
 * - Comments (line comments starting with #)
 * - Operators and delimiters
 */
export const starlarkMonarchTokens: monaco.languages.IMonarchLanguage = {
  defaultToken: "",
  tokenPostfix: ".star",

  keywords: STARLARK_KEYWORDS,
  builtins: STARLARK_BUILTINS,
  operators: STARLARK_OPERATORS,

  symbols: /[=><!~?:&|+\-*/^%]+/,
  escapes:
    /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Identifiers and keywords
      [
        /[a-zA-Z_]\w*/,
        {
          cases: {
            "@keywords": "keyword",
            "@builtins": "type.identifier",
            "@default": "identifier",
          },
        },
      ],

      // Whitespace
      { include: "@whitespace" },

      // Delimiters and operators
      [/[{}()[\]]/, "@brackets"],
      [/[<>](?!@symbols)/, "@brackets"],
      [
        /@symbols/,
        {
          cases: {
            "@operators": "operator",
            "@default": "",
          },
        },
      ],

      // Numbers
      [/\d*\.\d+([eE][-+]?\d+)?/, "number.float"],
      [/0[xX][0-9a-fA-F]+/, "number.hex"],
      [/0[oO][0-7]+/, "number.octal"],
      [/0[bB][01]+/, "number.binary"],
      [/\d+/, "number"],

      // Delimiter: after number because of .\d floats
      [/[;,.]/, "delimiter"],

      // Strings
      [/"""/, "string", "@string_triple_double"],
      [/'''/, "string", "@string_triple_single"],
      [/"([^"\\]|\\.)*$/, "string.invalid"], // non-terminated string
      [/'([^'\\]|\\.)*$/, "string.invalid"], // non-terminated string
      [/"/, "string", "@string_double"],
      [/'/, "string", "@string_single"],
    ],

    whitespace: [
      [/[ \t\r\n]+/, ""],
      [/#.*$/, "comment"],
    ],

    string_double: [
      [/[^\\"]+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/"/, "string", "@pop"],
    ],

    string_single: [
      [/[^\\']+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/'/, "string", "@pop"],
    ],

    string_triple_double: [
      [/"""/, "string", "@pop"],
      [/[^"]+/, "string"],
      [/"/, "string"],
    ],

    string_triple_single: [
      [/'''/, "string", "@pop"],
      [/[^']+/, "string"],
      [/'/, "string"],
    ],
  },
};
