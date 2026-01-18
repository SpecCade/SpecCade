//! Stage timing types for profiling support.

use serde::{Deserialize, Serialize};

/// Timing information for a single generation stage.
///
/// Captures the duration of a named stage during asset generation,
/// emitted when the `--profile` flag is used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageTiming {
    /// Name of the stage (e.g., "parse_params", "render_layers", "encode_output").
    pub stage: String,
    /// Duration of this stage in milliseconds.
    pub duration_ms: u64,
}

impl StageTiming {
    /// Creates a new stage timing entry.
    ///
    /// # Arguments
    /// * `stage` - Name identifying the stage
    /// * `duration_ms` - Duration of the stage in milliseconds
    ///
    /// # Example
    /// ```
    /// use speccade_spec::report::StageTiming;
    ///
    /// let timing = StageTiming::new("render_layers", 42);
    /// assert_eq!(timing.stage, "render_layers");
    /// assert_eq!(timing.duration_ms, 42);
    /// ```
    pub fn new(stage: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            stage: stage.into(),
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_timing_new() {
        let timing = StageTiming::new("test_stage", 123);
        assert_eq!(timing.stage, "test_stage");
        assert_eq!(timing.duration_ms, 123);
    }

    #[test]
    fn test_stage_timing_serialize() {
        let timing = StageTiming::new("encode_output", 50);
        let json = serde_json::to_string(&timing).unwrap();
        assert!(json.contains("\"stage\":\"encode_output\""));
        assert!(json.contains("\"duration_ms\":50"));
    }

    #[test]
    fn test_stage_timing_deserialize() {
        let json = r#"{"stage":"parse_params","duration_ms":10}"#;
        let timing: StageTiming = serde_json::from_str(json).unwrap();
        assert_eq!(timing.stage, "parse_params");
        assert_eq!(timing.duration_ms, 10);
    }
}
