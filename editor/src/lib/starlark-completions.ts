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
import { STDLIB_MANIFEST } from "./stdlib-manifest";

const STDLIB_BASE_SUGGESTIONS: Omit<monaco.languages.CompletionItem, "range">[] =
  STDLIB_MANIFEST.flatMap((category) =>
    category.functions.map((fn) => ({
      label: fn.name,
      kind: fn.completionKind ?? monaco.languages.CompletionItemKind.Function,
      insertText: fn.snippet,
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: { value: fn.description },
      detail: fn.signature,
    }))
  );

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
        STDLIB_BASE_SUGGESTIONS.map((item) => ({ ...item, range }));

      return { suggestions };
    },
  });
}
