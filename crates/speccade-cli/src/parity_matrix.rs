//! Parser for the legacy parity matrix markdown file.
//!
//! This module provides logic to parse markdown tables containing `Key` columns
//! from the parity matrix documentation. It extracts section names, table names,
//! key names, required status, and implementation status for each key.

/// Represents the implementation status of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    /// Fully implemented (checkmark symbol)
    Implemented,
    /// Partially implemented (tilde symbol)
    Partial,
    /// Not implemented (x symbol)
    NotImplemented,
    /// Deprecated (dash symbol)
    Deprecated,
}

impl KeyStatus {
    /// Parse a status string from the markdown table.
    pub fn parse_cell(s: &str) -> Option<Self> {
        let s = s.trim();
        // Check for the actual Unicode characters used in the markdown
        if s.contains('\u{2713}') || s.contains('\u{2714}') || s == "checkmark" {
            Some(KeyStatus::Implemented)
        } else if s.contains('~') {
            Some(KeyStatus::Partial)
        } else if s.contains('\u{2717}') || s.contains('\u{2718}') || s.contains('\u{2715}') {
            Some(KeyStatus::NotImplemented)
        } else if s == "-" || s.is_empty() {
            Some(KeyStatus::Deprecated)
        } else {
            None
        }
    }

    /// Convert to a Rust code representation for code generation.
    pub fn as_code_str(self) -> &'static str {
        match self {
            KeyStatus::Implemented => "KeyStatus::Implemented",
            KeyStatus::Partial => "KeyStatus::Partial",
            KeyStatus::NotImplemented => "KeyStatus::NotImplemented",
            KeyStatus::Deprecated => "KeyStatus::Deprecated",
        }
    }
}

/// A parsed key entry from the parity matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedKey {
    /// The section name (from ## headings, e.g., "SOUND (audio_sfx)")
    pub section: String,
    /// The table name (from ### headings, e.g., "Layer Keys")
    pub table: String,
    /// The key name (with backticks stripped)
    pub key: String,
    /// Whether the key is required (anything starting with "Yes")
    pub required: bool,
    /// The implementation status
    pub status: KeyStatus,
}

/// Result of parsing a parity matrix markdown file.
#[derive(Debug, Clone, Default)]
pub struct ParsedMatrix {
    /// All parsed key entries
    pub keys: Vec<ParsedKey>,
}

impl ParsedMatrix {
    /// Create a new empty matrix.
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Find a key by section, table, and key name.
    pub fn find(&self, section: &str, table: &str, key: &str) -> Option<&ParsedKey> {
        self.keys
            .iter()
            .find(|k| k.section == section && k.table == table && k.key == key)
    }
}

/// Parse a parity matrix markdown string.
///
/// This function:
/// 1. Tracks section headings (`## ...`)
/// 2. Tracks table headings (`### ...`)
/// 3. Parses tables that have a `Key` column
/// 4. Extracts key, required, and status fields from each row
pub fn parse_parity_matrix(content: &str) -> ParsedMatrix {
    let mut result = ParsedMatrix::new();
    let mut current_section = String::new();
    let mut current_table = String::new();
    let mut in_table = false;
    let mut header_indices: Option<TableColumns> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headings (## ...)
        if let Some(stripped) = trimmed.strip_prefix("## ") {
            current_section = stripped.trim().to_string();
            current_table.clear();
            in_table = false;
            header_indices = None;
            continue;
        }

        // Track table headings (### ...)
        if let Some(stripped) = trimmed.strip_prefix("### ") {
            current_table = stripped.trim().to_string();
            in_table = false;
            header_indices = None;
            continue;
        }

        // Skip empty lines or non-table lines
        if !trimmed.starts_with('|') {
            in_table = false;
            header_indices = None;
            continue;
        }

        // Parse table rows
        let cells: Vec<&str> = trimmed
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if cells.is_empty() {
            continue;
        }

        // Check if this is a header row (contains "Key")
        if !in_table {
            if let Some(indices) = find_table_columns(&cells) {
                header_indices = Some(indices);
                in_table = true;
            }
            continue;
        }

        // Skip separator rows (contain only dashes and colons)
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }

        // Parse data row
        if let Some(ref indices) = header_indices {
            if let Some(parsed_key) =
                parse_table_row(&cells, indices, &current_section, &current_table)
            {
                result.keys.push(parsed_key);
            }
        }
    }

    result
}

/// Column indices for a table with a Key column.
#[derive(Debug, Clone)]
struct TableColumns {
    key_idx: usize,
    required_idx: Option<usize>,
    status_idx: Option<usize>,
}

/// Find the column indices for Key, Required, and Status columns.
fn find_table_columns(header_cells: &[&str]) -> Option<TableColumns> {
    let mut key_idx = None;
    let mut required_idx = None;
    let mut status_idx = None;

    for (i, cell) in header_cells.iter().enumerate() {
        let cell_lower = cell.to_lowercase();
        if cell_lower == "key" {
            key_idx = Some(i);
        } else if cell_lower == "required" {
            required_idx = Some(i);
        } else if cell_lower == "status" {
            status_idx = Some(i);
        }
    }

    key_idx.map(|key_idx| TableColumns {
        key_idx,
        required_idx,
        status_idx,
    })
}

/// Parse a single table row into a ParsedKey.
fn parse_table_row(
    cells: &[&str],
    indices: &TableColumns,
    section: &str,
    table: &str,
) -> Option<ParsedKey> {
    // Get the key name
    let key_raw = cells.get(indices.key_idx)?;
    let key = strip_backticks(key_raw);

    // Skip if key is empty or looks like a header/separator
    if key.is_empty() || key == "Key" || key.chars().all(|c| c == '-' || c == ':') {
        return None;
    }

    // Get the required status
    let required = indices
        .required_idx
        .and_then(|i| cells.get(i))
        .map(|s| s.trim_start().to_lowercase().starts_with("yes"))
        .unwrap_or(false);

    // Get the implementation status
    let status = indices
        .status_idx
        .and_then(|i| cells.get(i))
        .and_then(|s| KeyStatus::parse_cell(s))
        .unwrap_or(KeyStatus::NotImplemented);

    Some(ParsedKey {
        section: section.to_string(),
        table: table.to_string(),
        key,
        required,
        status,
    })
}

/// Strip backticks from a key name.
///
/// Handles strings like "`key_name`" and returns "key_name".
fn strip_backticks(s: &str) -> String {
    s.trim().trim_matches('`').to_string()
}

/// Generate Rust code for the parsed matrix.
///
/// This generates:
/// - `enum KeyStatus { Implemented, Partial, NotImplemented, Deprecated }`
/// - `struct ParityKey { section: &'static str, table: &'static str, key: &'static str }`
/// - `struct KeyInfo { key: ParityKey, required: bool, status: KeyStatus }`
/// - `static ALL_KEYS: &[KeyInfo]`
/// - `fn find(section: &str, table: &str, key: &str) -> Option<&'static KeyInfo>`
pub fn generate_rust_code(matrix: &ParsedMatrix) -> String {
    let mut code = String::new();

    // Header comment
    code.push_str("// Auto-generated by build.rs from PARITY_MATRIX.md\n");
    code.push_str("// Do not edit manually!\n\n");

    // KeyStatus enum
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("pub enum KeyStatus {\n");
    code.push_str("    Implemented,\n");
    code.push_str("    Partial,\n");
    code.push_str("    NotImplemented,\n");
    code.push_str("    Deprecated,\n");
    code.push_str("}\n\n");

    // ParityKey struct
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("pub struct ParityKey {\n");
    code.push_str("    pub section: &'static str,\n");
    code.push_str("    pub table: &'static str,\n");
    code.push_str("    pub key: &'static str,\n");
    code.push_str("}\n\n");

    // KeyInfo struct
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    code.push_str("pub struct KeyInfo {\n");
    code.push_str("    pub key: ParityKey,\n");
    code.push_str("    pub required: bool,\n");
    code.push_str("    pub status: KeyStatus,\n");
    code.push_str("}\n\n");

    // ALL_KEYS static array
    code.push_str("pub static ALL_KEYS: &[KeyInfo] = &[\n");
    for key in &matrix.keys {
        code.push_str("    KeyInfo {\n");
        code.push_str("        key: ParityKey {\n");
        code.push_str(&format!("            section: {:?},\n", key.section));
        code.push_str(&format!("            table: {:?},\n", key.table));
        code.push_str(&format!("            key: {:?},\n", key.key));
        code.push_str("        },\n");
        code.push_str(&format!("        required: {},\n", key.required));
        code.push_str(&format!("        status: {},\n", key.status.as_code_str()));
        code.push_str("    },\n");
    }
    code.push_str("];\n\n");

    // find() helper function
    code.push_str("/// Find a key by section, table, and key name.\n");
    code.push_str(
        "pub fn find(section: &str, table: &str, key: &str) -> Option<&'static KeyInfo> {\n",
    );
    code.push_str("    ALL_KEYS.iter().find(|info| {\n");
    code.push_str(
        "        info.key.section == section && info.key.table == table && info.key.key == key\n",
    );
    code.push_str("    })\n");
    code.push_str("}\n");

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_status_parse_cell() {
        // Unicode checkmark
        assert_eq!(
            KeyStatus::parse_cell("\u{2713}"),
            Some(KeyStatus::Implemented)
        );
        assert_eq!(
            KeyStatus::parse_cell("\u{2714}"),
            Some(KeyStatus::Implemented)
        );

        // Tilde for partial
        assert_eq!(KeyStatus::parse_cell("~"), Some(KeyStatus::Partial));

        // Unicode x marks
        assert_eq!(
            KeyStatus::parse_cell("\u{2717}"),
            Some(KeyStatus::NotImplemented)
        );
        assert_eq!(
            KeyStatus::parse_cell("\u{2718}"),
            Some(KeyStatus::NotImplemented)
        );
        assert_eq!(
            KeyStatus::parse_cell("\u{2715}"),
            Some(KeyStatus::NotImplemented)
        );

        // Dash for deprecated
        assert_eq!(KeyStatus::parse_cell("-"), Some(KeyStatus::Deprecated));
        assert_eq!(KeyStatus::parse_cell(""), Some(KeyStatus::Deprecated));
    }

    #[test]
    fn test_strip_backticks() {
        assert_eq!(strip_backticks("`name`"), "name");
        assert_eq!(strip_backticks("name"), "name");
        assert_eq!(strip_backticks("  `name`  "), "name");
        assert_eq!(strip_backticks("`key.subkey`"), "key.subkey");
    }

    #[test]
    fn test_parse_simple_table() {
        let markdown = r#"
## SOUND (audio_sfx)

### Top-Level Keys

| Key | Required | Type | Status |
|-----|----------|------|--------|
| `name` | Yes | `str` | ✓ |
| `duration` | No | `float` | ✓ |
| `layers` | No | `list` | ~ |
| `deprecated_key` | No | `str` | - |
"#;

        let matrix = parse_parity_matrix(markdown);
        assert_eq!(matrix.keys.len(), 4);

        let name_key = matrix
            .find("SOUND (audio_sfx)", "Top-Level Keys", "name")
            .unwrap();
        assert!(name_key.required);
        assert_eq!(name_key.status, KeyStatus::Implemented);

        let duration_key = matrix
            .find("SOUND (audio_sfx)", "Top-Level Keys", "duration")
            .unwrap();
        assert!(!duration_key.required);
        assert_eq!(duration_key.status, KeyStatus::Implemented);

        let layers_key = matrix
            .find("SOUND (audio_sfx)", "Top-Level Keys", "layers")
            .unwrap();
        assert!(!layers_key.required);
        assert_eq!(layers_key.status, KeyStatus::Partial);

        let deprecated_key = matrix
            .find("SOUND (audio_sfx)", "Top-Level Keys", "deprecated_key")
            .unwrap();
        assert!(!deprecated_key.required);
        assert_eq!(deprecated_key.status, KeyStatus::Deprecated);
    }

    #[test]
    fn test_parse_yes_variations() {
        let markdown = r#"
## TEST

### Keys

| Key | Required | Status |
|-----|----------|--------|
| `a` | Yes | ✓ |
| `b` | Yes (or mirror) | ✓ |
| `c` | No | ✓ |
"#;

        let matrix = parse_parity_matrix(markdown);
        assert_eq!(matrix.keys.len(), 3);

        let a = matrix.find("TEST", "Keys", "a").unwrap();
        assert!(a.required);

        let b = matrix.find("TEST", "Keys", "b").unwrap();
        assert!(b.required);

        let c = matrix.find("TEST", "Keys", "c").unwrap();
        assert!(!c.required);
    }

    #[test]
    fn test_parse_multiple_sections() {
        let markdown = r#"
## Section One

### Table A

| Key | Required | Status |
|-----|----------|--------|
| `key1` | Yes | ✓ |

### Table B

| Key | Required | Status |
|-----|----------|--------|
| `key2` | No | ~ |

## Section Two

### Table C

| Key | Required | Status |
|-----|----------|--------|
| `key3` | Yes | ✗ |
"#;

        let matrix = parse_parity_matrix(markdown);
        assert_eq!(matrix.keys.len(), 3);

        assert!(matrix.find("Section One", "Table A", "key1").is_some());
        assert!(matrix.find("Section One", "Table B", "key2").is_some());
        assert!(matrix.find("Section Two", "Table C", "key3").is_some());
    }

    #[test]
    fn test_skip_tables_without_key_column() {
        let markdown = r#"
## Section

### Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented (✓) | 30 | 79% |

### Keys

| Key | Required | Status |
|-----|----------|--------|
| `name` | Yes | ✓ |
"#;

        let matrix = parse_parity_matrix(markdown);
        // Should only have 1 key (the second table with Key column)
        assert_eq!(matrix.keys.len(), 1);
        assert!(matrix.find("Section", "Keys", "name").is_some());
    }

    #[test]
    fn test_generate_rust_code() {
        let matrix = ParsedMatrix {
            keys: vec![ParsedKey {
                section: "SOUND".to_string(),
                table: "Top-Level Keys".to_string(),
                key: "name".to_string(),
                required: true,
                status: KeyStatus::Implemented,
            }],
        };

        let code = generate_rust_code(&matrix);

        // Check that generated code contains expected elements
        assert!(code.contains("pub enum KeyStatus"));
        assert!(code.contains("pub struct ParityKey"));
        assert!(code.contains("pub struct KeyInfo"));
        assert!(code.contains("pub static ALL_KEYS: &[KeyInfo]"));
        assert!(code.contains("pub fn find("));
        assert!(code.contains("section: \"SOUND\""));
        assert!(code.contains("table: \"Top-Level Keys\""));
        assert!(code.contains("key: \"name\""));
        assert!(code.contains("required: true"));
        assert!(code.contains("status: KeyStatus::Implemented"));
    }

    #[test]
    fn test_find_returns_none_for_missing() {
        let matrix = ParsedMatrix::new();
        assert!(matrix.find("SOUND", "Keys", "nonexistent").is_none());
    }
}
