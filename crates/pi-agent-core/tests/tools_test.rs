use pi_agent_core::tools::ToolRegistry;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_pi_fs_tools_roundtrip() {
    let dir = tempdir().unwrap();
    let registry = ToolRegistry::with_standard_tools(dir.path());

    // Test write_file
    let write_tool = registry.get("write_file").expect("write_file tool");
    let write_res = write_tool
        .execute(json!({
            "path": "src/hello.rs",
            "content": "fn main() {\n    println!(\"hello pi\");\n}\n"
        }))
        .await
        .unwrap();
    assert!(write_res.contains("Successfully wrote"));

    // Test read_file
    let read_tool = registry.get("read_file").expect("read_file tool");
    let read_res = read_tool
        .execute(json!({
            "path": "src/hello.rs"
        }))
        .await
        .unwrap();
    assert!(read_res.contains("println!(\"hello pi\");"));

    // Test edit_file
    let edit_tool = registry.get("edit_file").expect("edit_file tool");
    let edit_res = edit_tool
        .execute(json!({
            "path": "src/hello.rs",
            "target_content": "\"hello pi\"",
            "replacement_content": "\"hello rust pi agent\""
        }))
        .await
        .unwrap();
    assert!(edit_res.contains("Successfully replaced 1 occurrence"));

    // Verify after edit
    let read_res2 = read_tool
        .execute(json!({
            "path": "src/hello.rs"
        }))
        .await
        .unwrap();
    assert!(read_res2.contains("\"hello rust pi agent\""));

    // Test find tool
    let find_tool = registry.get("find").expect("find tool");
    let find_res = find_tool
        .execute(json!({
            "pattern": "hello.rs"
        }))
        .await
        .unwrap();
    assert!(find_res.contains("src/hello.rs"));

    // Test grep tool
    let grep_tool = registry.get("grep").expect("grep tool");
    let grep_res = grep_tool
        .execute(json!({
            "pattern": "rust pi agent"
        }))
        .await
        .unwrap();
    assert!(grep_res.contains("src/hello.rs:2: println!(\"hello rust pi agent\");"));

    // Test bash tool
    let bash_tool = registry.get("bash").expect("bash tool");
    let bash_res = bash_tool
        .execute(json!({
            "command": "cat src/hello.rs"
        }))
        .await
        .unwrap();
    assert!(bash_res.contains("hello rust pi agent"));
}
