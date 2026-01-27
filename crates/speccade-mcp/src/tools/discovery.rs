use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ListTemplatesParams {
    /// Filter by asset type (e.g. "sprite", "tilemap")
    pub asset_type: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetTemplateParams {
    /// Template identifier
    pub template_id: String,
    /// Asset type context
    pub asset_type: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSpecsParams {
    /// Directory to search (defaults to ".")
    pub directory: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadSpecParams {
    /// Path to the .star spec file
    pub path: String,
}
