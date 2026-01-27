use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct WriteSpecParams {
    /// Path to write the spec file
    pub path: String,
    /// Starlark spec content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct EvalSpecParams {
    /// Path to the .star spec file
    pub path: String,
}
