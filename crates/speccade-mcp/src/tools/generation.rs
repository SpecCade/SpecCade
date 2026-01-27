use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ValidateSpecParams {
    /// Path to the .star spec file
    pub path: String,
    /// Budget profile (e.g. "fast", "full")
    pub budget: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeneratePreviewParams {
    /// Path to the .star spec file
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GenerateFullParams {
    /// Path to the .star spec file
    pub path: String,
    /// Output directory
    pub out_dir: Option<String>,
}
