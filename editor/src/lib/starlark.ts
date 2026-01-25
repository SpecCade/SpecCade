/**
 * Starlark language support for Monaco editor.
 *
 * This module re-exports from the split module files for backwards compatibility:
 * - starlark-language.ts: Language configuration (comments, brackets, indentation)
 * - starlark-tokens.ts: Monarch tokenizer (syntax highlighting rules)
 *
 * @module starlark
 */
import * as monaco from "monaco-editor";
import {
  STARLARK_LANGUAGE_ID,
  STARLARK_EXTENSIONS,
  starlarkLanguageConfiguration,
} from "./starlark-language";
import {
  starlarkMonarchTokens,
  STARLARK_KEYWORDS,
  STARLARK_BUILTINS,
  STARLARK_OPERATORS,
} from "./starlark-tokens";

// Re-export for backwards compatibility
export {
  STARLARK_LANGUAGE_ID,
  STARLARK_EXTENSIONS,
  starlarkLanguageConfiguration,
  starlarkMonarchTokens,
  STARLARK_KEYWORDS,
  STARLARK_BUILTINS,
  STARLARK_OPERATORS,
};

// Registration state
let registered = false;

/**
 * Register the Starlark language with Monaco.
 *
 * This function is idempotent - calling it multiple times has no effect.
 */
export function registerStarlarkLanguage(): void {
  if (registered) return;

  // Register the language
  monaco.languages.register({
    id: STARLARK_LANGUAGE_ID,
    extensions: STARLARK_EXTENSIONS,
    aliases: ["Starlark", "starlark", "star"],
    mimetypes: ["text/x-starlark"],
  });

  // Set language configuration (brackets, comments, indentation, etc.)
  monaco.languages.setLanguageConfiguration(
    STARLARK_LANGUAGE_ID,
    starlarkLanguageConfiguration
  );

  // Set token provider (syntax highlighting)
  monaco.languages.setMonarchTokensProvider(
    STARLARK_LANGUAGE_ID,
    starlarkMonarchTokens
  );

  registered = true;
}
