use speccade_mcp::tools::SpeccadeMcp;
use std::collections::HashSet;

/// All tools must be registered in the tool router.
#[test]
fn all_tools_registered() {
    let server = SpeccadeMcp::new();
    let tools = server.router().list_all();
    let names: HashSet<&str> = tools.iter().map(|t| t.name.as_ref()).collect();

    let expected = [
        // Discovery
        "stdlib_reference",
        "list_templates",
        "get_template",
        "list_specs",
        "read_spec",
        // Authoring
        "write_spec",
        "eval_spec",
        // Generation
        "validate_spec",
        "generate_preview",
        "generate_full",
        "generate_png_outputs",
        // Analysis
        "analyze_asset",
        "compare_assets",
    ];

    assert_eq!(
        tools.len(),
        expected.len(),
        "Expected {} tools, got {}: {:?}",
        expected.len(),
        tools.len(),
        names
    );

    for name in &expected {
        assert!(names.contains(name), "Missing tool: {name}");
    }
}

/// Every tool must have a non-empty description (from doc comments).
#[test]
fn all_tools_have_descriptions() {
    let server = SpeccadeMcp::new();
    let tools = server.router().list_all();

    for tool in &tools {
        let desc = tool.description.as_deref().unwrap_or("");
        assert!(!desc.is_empty(), "Tool '{}' has no description", tool.name);
    }
}

/// Tools that accept parameters must have a non-trivial input schema.
#[test]
fn parameterized_tools_have_input_schema() {
    let server = SpeccadeMcp::new();
    let tools = server.router().list_all();

    // Tools that take parameters (all except stdlib_reference)
    let parameterized = [
        "list_templates",
        "get_template",
        "list_specs",
        "read_spec",
        "write_spec",
        "eval_spec",
        "validate_spec",
        "generate_preview",
        "generate_full",
        "generate_png_outputs",
        "analyze_asset",
        "compare_assets",
    ];

    for tool in &tools {
        if parameterized.contains(&tool.name.as_ref()) {
            let schema = serde_json::to_value(&*tool.input_schema).unwrap();
            // Should have "properties" key with at least one property
            let props = schema.get("properties");
            assert!(
                props.is_some(),
                "Tool '{}' should have properties in input schema, got: {}",
                tool.name,
                serde_json::to_string_pretty(&schema).unwrap()
            );
            let props = props.unwrap().as_object().unwrap();
            assert!(
                !props.is_empty(),
                "Tool '{}' has empty properties",
                tool.name
            );
        }
    }
}

/// Parameter structs must deserialize correctly from JSON.
#[test]
fn param_deserialization() {
    use speccade_mcp::tools::analysis::*;
    use speccade_mcp::tools::authoring::*;
    use speccade_mcp::tools::discovery::*;
    use speccade_mcp::tools::generation::*;

    // ListTemplatesParams with asset_type
    let p: ListTemplatesParams = serde_json::from_str(r#"{"asset_type": "audio"}"#).unwrap();
    assert_eq!(p.asset_type.as_deref(), Some("audio"));

    // ListTemplatesParams without asset_type
    let p: ListTemplatesParams = serde_json::from_str(r#"{}"#).unwrap();
    assert!(p.asset_type.is_none());

    // GetTemplateParams
    let p: GetTemplateParams = serde_json::from_str(r#"{"template_id": "laser-01"}"#).unwrap();
    assert_eq!(p.template_id, "laser-01");
    assert!(p.asset_type.is_none());

    // ListSpecsParams default
    let p: ListSpecsParams = serde_json::from_str(r#"{}"#).unwrap();
    assert!(p.directory.is_none());

    // ReadSpecParams
    let p: ReadSpecParams = serde_json::from_str(r#"{"path": "specs/test.star"}"#).unwrap();
    assert_eq!(p.path, "specs/test.star");

    // WriteSpecParams
    let p: WriteSpecParams =
        serde_json::from_str(r#"{"path": "out.star", "content": "spec(...)"}"#).unwrap();
    assert_eq!(p.path, "out.star");
    assert_eq!(p.content, "spec(...)");

    // EvalSpecParams
    let p: EvalSpecParams = serde_json::from_str(r#"{"path": "test.star"}"#).unwrap();
    assert_eq!(p.path, "test.star");

    // ValidateSpecParams with budget
    let p: ValidateSpecParams =
        serde_json::from_str(r#"{"path": "test.star", "budget": "strict"}"#).unwrap();
    assert_eq!(p.budget.as_deref(), Some("strict"));

    // GeneratePreviewParams
    let p: GeneratePreviewParams = serde_json::from_str(r#"{"path": "test.star"}"#).unwrap();
    assert_eq!(p.path, "test.star");

    // GenerateFullParams with out_dir
    let p: GenerateFullParams =
        serde_json::from_str(r#"{"path": "test.star", "out_dir": "/tmp/out"}"#).unwrap();
    assert_eq!(p.out_dir.as_deref(), Some("/tmp/out"));

    // GeneratePngOutputsParams
    let p: GeneratePngOutputsParams =
        serde_json::from_str(r#"{"path": "test.star", "budget": "strict"}"#).unwrap();
    assert_eq!(p.path, "test.star");
    assert_eq!(p.budget.as_deref(), Some("strict"));

    // AnalyzeAssetParams
    let p: AnalyzeAssetParams = serde_json::from_str(r#"{"path": "output.wav"}"#).unwrap();
    assert_eq!(p.path, "output.wav");

    // CompareAssetsParams
    let p: CompareAssetsParams =
        serde_json::from_str(r#"{"path_a": "a.wav", "path_b": "b.wav"}"#).unwrap();
    assert_eq!(p.path_a, "a.wav");
    assert_eq!(p.path_b, "b.wav");
}

/// write_spec and read_spec round-trip through the filesystem.
#[tokio::test]
async fn write_and_read_spec_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let spec_path = dir.path().join("test.star");
    let content = "spec(asset_id = \"test\", asset_type = \"audio\")";

    // Write
    tokio::fs::write(&spec_path, content).await.unwrap();

    // Read back
    let read_back = tokio::fs::read_to_string(&spec_path).await.unwrap();
    assert_eq!(read_back, content);
}

/// list_specs finds .star files in a directory.
#[tokio::test]
async fn list_specs_finds_star_files() {
    let dir = tempfile::tempdir().unwrap();

    // Create some .star files and a non-.star file
    tokio::fs::write(dir.path().join("a.star"), "")
        .await
        .unwrap();
    tokio::fs::write(dir.path().join("b.star"), "")
        .await
        .unwrap();
    tokio::fs::write(dir.path().join("c.txt"), "")
        .await
        .unwrap();
    tokio::fs::create_dir_all(dir.path().join("sub"))
        .await
        .unwrap();
    tokio::fs::write(dir.path().join("sub/d.star"), "")
        .await
        .unwrap();

    let pattern = format!("{}/**/*.star", dir.path().display());
    let paths: Vec<String> = glob::glob(&pattern)
        .unwrap()
        .filter_map(|p| p.ok())
        .map(|p| p.display().to_string())
        .collect();

    assert_eq!(paths.len(), 3, "Expected 3 .star files, got: {:?}", paths);
}
