//! CSV formatting for analyze command
//!
//! Provides CSV output formatting for batch analysis results.

use anyhow::Result;
use std::collections::BTreeMap;
use std::io::Write;

use super::json_output::BatchAnalyzeItem;

/// Format results as CSV with flattened metrics.
pub fn format_csv(results: &[BatchAnalyzeItem], include_embeddings: bool) -> Result<String> {
    let mut output = Vec::new();

    // Collect all unique metric keys across all results
    let mut metric_keys: Vec<String> = Vec::new();
    for item in results {
        if let Some(ref metrics) = item.metrics {
            collect_metric_keys(metrics, "", &mut metric_keys);
        }
    }
    metric_keys.sort();
    metric_keys.dedup();

    // Write header
    let mut header = vec![
        "input".to_string(),
        "success".to_string(),
        "asset_type".to_string(),
        "input_hash".to_string(),
        "error_code".to_string(),
        "error_message".to_string(),
    ];
    header.extend(metric_keys.iter().cloned());
    if include_embeddings {
        header.push("embedding".to_string());
    }
    writeln!(output, "{}", header.join(","))?;

    // Write rows
    for item in results {
        // Basic fields
        let (error_code, error_message) = if let Some(ref error) = item.error {
            (csv_escape(&error.code), csv_escape(&error.message))
        } else {
            (String::new(), String::new())
        };

        let mut row: Vec<String> = vec![
            csv_escape(&item.input),
            item.success.to_string(),
            item.asset_type.as_deref().unwrap_or("").to_string(),
            item.input_hash.as_deref().unwrap_or("").to_string(),
            error_code,
            error_message,
        ];

        // Metric fields
        for key in &metric_keys {
            let value = item
                .metrics
                .as_ref()
                .and_then(|m| get_nested_value(m, key))
                .map(|v| format_csv_value(&v))
                .unwrap_or_default();
            row.push(value);
        }

        // Embedding (as JSON array string if present)
        if include_embeddings {
            let emb_str = item
                .embedding
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default())
                .unwrap_or_default();
            row.push(csv_escape(&emb_str));
        }

        writeln!(output, "{}", row.join(","))?;
    }

    String::from_utf8(output).map_err(|e| anyhow::anyhow!("UTF-8 error: {}", e))
}

/// Recursively collect all metric keys with dot notation.
fn collect_metric_keys(
    map: &BTreeMap<String, serde_json::Value>,
    prefix: &str,
    keys: &mut Vec<String>,
) {
    for (key, value) in map {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            serde_json::Value::Object(obj) => {
                // Convert to BTreeMap for recursive call
                let nested: BTreeMap<String, serde_json::Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                collect_metric_keys(&nested, &full_key, keys);
            }
            _ => {
                keys.push(full_key);
            }
        }
    }
}

/// Get a nested value using dot notation (returns cloned value).
fn get_nested_value(
    map: &BTreeMap<String, serde_json::Value>,
    path: &str,
) -> Option<serde_json::Value> {
    let parts: Vec<&str> = path.splitn(2, '.').collect();
    match parts.as_slice() {
        [key] => map.get(*key).cloned(),
        [key, rest] => map.get(*key).and_then(|v| {
            if let serde_json::Value::Object(obj) = v {
                let nested: BTreeMap<String, serde_json::Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                get_nested_value(&nested, rest)
            } else {
                None
            }
        }),
        _ => None,
    }
}

/// Format a JSON value for CSV.
fn format_csv_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => csv_escape(s),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            csv_escape(&serde_json::to_string(value).unwrap_or_default())
        }
    }
}

/// Escape a string for CSV (quote if contains comma, quote, or newline).
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_csv_escape_quote() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_format_csv_value_null() {
        assert_eq!(format_csv_value(&serde_json::Value::Null), "");
    }

    #[test]
    fn test_format_csv_value_bool() {
        assert_eq!(format_csv_value(&serde_json::Value::Bool(true)), "true");
    }

    #[test]
    fn test_format_csv_value_number() {
        assert_eq!(format_csv_value(&serde_json::json!(42)), "42");
    }

    #[test]
    fn test_format_csv_value_string() {
        assert_eq!(
            format_csv_value(&serde_json::Value::String("test".to_string())),
            "test"
        );
    }

    #[test]
    fn test_get_nested_value_simple() {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), serde_json::json!("value"));
        assert_eq!(
            get_nested_value(&map, "key"),
            Some(serde_json::json!("value"))
        );
    }

    #[test]
    fn test_get_nested_value_nested() {
        let mut map = BTreeMap::new();
        map.insert("outer".to_string(), serde_json::json!({"inner": "value"}));
        assert_eq!(
            get_nested_value(&map, "outer.inner"),
            Some(serde_json::json!("value"))
        );
    }

    #[test]
    fn test_get_nested_value_missing() {
        let map = BTreeMap::new();
        assert_eq!(get_nested_value(&map, "missing"), None);
    }

    #[test]
    fn test_collect_metric_keys_flat() {
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), serde_json::json!(1));
        map.insert("b".to_string(), serde_json::json!(2));
        let mut keys = Vec::new();
        collect_metric_keys(&map, "", &mut keys);
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_collect_metric_keys_nested() {
        let mut map = BTreeMap::new();
        map.insert("outer".to_string(), serde_json::json!({"inner": 1}));
        let mut keys = Vec::new();
        collect_metric_keys(&map, "", &mut keys);
        assert_eq!(keys, vec!["outer.inner"]);
    }
}
