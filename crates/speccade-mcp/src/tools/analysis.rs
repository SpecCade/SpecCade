use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct AnalyzeAssetParams {
    /// Path to the asset file
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CompareAssetsParams {
    /// Path to first asset
    pub path_a: String,
    /// Path to second asset
    pub path_b: String,
}
