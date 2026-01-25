/**
 * Starlark language configuration for Monaco editor.
 *
 * This file contains the language ID and configuration (comments, brackets,
 * auto-closing pairs, folding, indentation rules) for Starlark files.
 */
import type * as monaco from "monaco-editor";

/**
 * The language identifier for Starlark in Monaco.
 */
export const STARLARK_LANGUAGE_ID = "starlark";

/**
 * File extensions associated with Starlark.
 */
export const STARLARK_EXTENSIONS = [".star", ".bzl", ".bazel"];

/**
 * Starlark language configuration for Monaco.
 *
 * Defines:
 * - Comment syntax (line comments with #)
 * - Bracket pairs for matching and highlighting
 * - Auto-closing pairs for quotes and brackets
 * - Surrounding pairs for selection wrapping
 * - Folding markers (region/endregion)
 * - Indentation rules for Python-like syntax
 */
export const starlarkLanguageConfiguration: monaco.languages.LanguageConfiguration =
  {
    comments: {
      lineComment: "#",
    },
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '"""', close: '"""' },
      { open: "'''", close: "'''" },
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    folding: {
      markers: {
        start: /^\s*#\s*region\b/,
        end: /^\s*#\s*endregion\b/,
      },
    },
    indentationRules: {
      increaseIndentPattern: /^\s*(def|if|elif|else|for|while)\b.*:\s*$/,
      decreaseIndentPattern: /^\s*(pass|return|break|continue)\b/,
    },
    wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\#\%\^\&\*\(\)\-\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/g,
  };
