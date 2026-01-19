//! Conversion from Starlark values to JSON values.
//!
//! This module handles the transformation of Starlark evaluation results
//! into `serde_json::Value` for subsequent parsing into a canonical Spec.

use super::error::CompileError;
use starlark::values::{dict::DictRef, list::ListRef, Value};

/// Converts a Starlark Value to a serde_json::Value.
///
/// Supported conversions:
/// - `NoneType` -> `null`
/// - `bool` -> boolean
/// - `int` -> number (integer)
/// - `float` -> number (float)
/// - `string` -> string
/// - `list` -> array
/// - `dict` -> object
///
/// # Arguments
/// * `value` - The Starlark value to convert
///
/// # Returns
/// * `Ok(serde_json::Value)` - Successfully converted value
/// * `Err(CompileError::JsonConversion)` - Unsupported type or conversion failure
pub fn starlark_to_json(value: Value) -> Result<serde_json::Value, CompileError> {
    convert_value(value)
}

fn convert_value(value: Value) -> Result<serde_json::Value, CompileError> {
    // Check for None first
    if value.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // Try boolean
    if let Some(b) = value.unpack_bool() {
        return Ok(serde_json::Value::Bool(b));
    }

    // Try integer
    if let Some(i) = value.unpack_i32() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    // starlark 0.12.0 supports ints beyond i32, but `unpack_i32` only covers a subset.
    // Fall back to parsing the string form for larger values so we can represent seeds (u32) and similar.
    if value.get_type() == "int" {
        let s = value.to_str();
        if let Ok(i) = s.parse::<i64>() {
            return Ok(serde_json::Value::Number(serde_json::Number::from(i)));
        }
        if let Ok(u) = s.parse::<u64>() {
            return Ok(serde_json::Value::Number(serde_json::Number::from(u)));
        }
        return Err(CompileError::JsonConversion {
            message: format!("cannot represent int {} as JSON number", s),
        });
    }

    // Try string
    if let Some(s) = value.unpack_str() {
        return Ok(serde_json::Value::String(s.to_string()));
    }

    // Try float - Starlark floats are f64
    // Use string representation since unpack_num() is private in starlark 0.12.0
    if value.get_type() == "float" {
        let s = value.to_str();
        if let Ok(f) = s.parse::<f64>() {
            return serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| CompileError::JsonConversion {
                    message: format!("cannot represent float {} as JSON number", f),
                });
        }
    }

    // Try list
    if let Some(list) = ListRef::from_value(value) {
        let items: Result<Vec<_>, _> = list.iter().map(convert_value).collect();
        return Ok(serde_json::Value::Array(items?));
    }

    // Try dict
    if let Some(dict) = DictRef::from_value(value) {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key = k
                .unpack_str()
                .ok_or_else(|| CompileError::JsonConversion {
                    message: format!("dict keys must be strings, got {}", k.get_type()),
                })?
                .to_string();
            let val = convert_value(v)?;
            map.insert(key, val);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Unsupported type
    Err(CompileError::JsonConversion {
        message: format!("unsupported Starlark type: {}", value.get_type()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::environment::{Globals, Module};
    use starlark::eval::Evaluator;
    use starlark::syntax::{AstModule, Dialect};

    /// Helper to evaluate a Starlark expression and convert to JSON
    fn eval_to_json(expr: &str) -> Result<serde_json::Value, CompileError> {
        let ast =
            AstModule::parse("test.star", expr.to_string(), &Dialect::Standard).map_err(|e| {
                CompileError::Syntax {
                    location: "test.star".to_string(),
                    message: e.to_string(),
                }
            })?;

        let module = Module::new();
        let globals = Globals::standard();
        let mut eval = Evaluator::new(&module);

        let value = eval
            .eval_module(ast, &globals)
            .map_err(|e| CompileError::Runtime {
                location: "test.star".to_string(),
                message: e.to_string(),
            })?;

        starlark_to_json(value)
    }

    #[test]
    fn test_convert_none() {
        let result = eval_to_json("None").unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_convert_bool() {
        assert_eq!(eval_to_json("True").unwrap(), serde_json::json!(true));
        assert_eq!(eval_to_json("False").unwrap(), serde_json::json!(false));
    }

    #[test]
    fn test_convert_int() {
        assert_eq!(eval_to_json("42").unwrap(), serde_json::json!(42));
        assert_eq!(eval_to_json("-123").unwrap(), serde_json::json!(-123));
        assert_eq!(eval_to_json("0").unwrap(), serde_json::json!(0));
        // > i32::MAX, still representable as JSON integer
        assert_eq!(
            eval_to_json("4294967295").unwrap(),
            serde_json::json!(4294967295_i64)
        );
    }

    #[test]
    fn test_convert_float() {
        let result = eval_to_json("42.5").unwrap();
        assert!(result.is_number());
        let expected = 42.5_f64;
        assert!((result.as_f64().unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_convert_string() {
        assert_eq!(
            eval_to_json("\"hello\"").unwrap(),
            serde_json::json!("hello")
        );
        assert_eq!(eval_to_json("''").unwrap(), serde_json::json!(""));
    }

    #[test]
    fn test_convert_list() {
        assert_eq!(
            eval_to_json("[1, 2, 3]").unwrap(),
            serde_json::json!([1, 2, 3])
        );
        assert_eq!(eval_to_json("[]").unwrap(), serde_json::json!([]));
        assert_eq!(
            eval_to_json("[\"a\", \"b\"]").unwrap(),
            serde_json::json!(["a", "b"])
        );
    }

    #[test]
    fn test_convert_dict() {
        assert_eq!(
            eval_to_json("{\"a\": 1, \"b\": 2}").unwrap(),
            serde_json::json!({"a": 1, "b": 2})
        );
        assert_eq!(eval_to_json("{}").unwrap(), serde_json::json!({}));
    }

    #[test]
    fn test_convert_nested() {
        let result = eval_to_json("{\"items\": [1, 2], \"nested\": {\"x\": True}}").unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "items": [1, 2],
                "nested": {"x": true}
            })
        );
    }

    #[test]
    fn test_convert_mixed_list() {
        let result = eval_to_json("[1, \"two\", 3.0, None, True]").unwrap();
        assert_eq!(result, serde_json::json!([1, "two", 3.0, null, true]));
    }
}
