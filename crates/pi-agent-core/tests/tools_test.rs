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

#[tokio::test]
async fn test_calc_tool_scientific_suite() {
    let dir = tempdir().unwrap();
    let registry = ToolRegistry::with_standard_tools(dir.path());
    let calc = registry.get("calc").expect("calc tool");

    // 1. Math Evaluator
    let eval_res = calc
        .execute(json!({
            "action": "eval",
            "expr": "sin(pi/2) + sqrt(16) * 2^3"
        }))
        .await
        .unwrap();
    assert!(eval_res.contains("Result: 33"));

    // 1b. Unary minus and negative operator precedence tests
    let neg_res1 = calc
        .execute(json!({
            "action": "eval",
            "expr": "2 * -5"
        }))
        .await
        .unwrap();
    assert!(neg_res1.contains("Result: -10"));

    let neg_res2 = calc
        .execute(json!({
            "action": "eval",
            "expr": "10 / -2"
        }))
        .await
        .unwrap();
    assert!(neg_res2.contains("Result: -5"));

    let neg_res3 = calc
        .execute(json!({
            "action": "eval",
            "expr": "10 - -5"
        }))
        .await
        .unwrap();
    assert!(neg_res3.contains("Result: 15"));

    let neg_res4 = calc
        .execute(json!({
            "action": "eval",
            "expr": "-(3 + 4)"
        }))
        .await
        .unwrap();
    assert!(neg_res4.contains("Result: -7"));

    // 2. Descriptive Statistics
    let stat_res = calc
        .execute(json!({
            "action": "stat",
            "data": [10.0, 20.0, 30.0, 40.0, 50.0]
        }))
        .await
        .unwrap();
    assert!(stat_res.contains("Mean (平均值): 30.0000"));
    assert!(stat_res.contains("Median (中位数): 30.0000"));
    assert!(stat_res.contains("Min: 10.0000 | Max: 50.0000"));

    // 3. Linear Regression
    let reg_res = calc
        .execute(json!({
            "action": "regression",
            "x": [1.0, 2.0, 3.0, 4.0, 5.0],
            "y": [2.0, 4.0, 6.0, 8.0, 10.0]
        }))
        .await
        .unwrap();
    assert!(reg_res.contains("Slope (斜率 m): 2.000000"));
    assert!(reg_res.contains("Pearson Correlation (相关系数 r): 1.000000"));

    // 4. Matrix Multiplication
    let mat_res = calc
        .execute(json!({
            "action": "matrix",
            "operation": "mul",
            "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
            "matrix_b": [[2.0, 0.0], [1.0, 2.0]]
        }))
        .await
        .unwrap();
    assert!(mat_res.contains("Result (2x2)"));

    // 4b. Matrix Transpose & Empty Matrix Transpose
    let trans_res = calc
        .execute(json!({
            "action": "matrix",
            "operation": "transpose",
            "matrix_a": [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
        }))
        .await
        .unwrap();
    assert!(trans_res.contains("Transposed Matrix (3x2)"));

    let trans_empty = calc
        .execute(json!({
            "action": "matrix",
            "operation": "transpose",
            "matrix_a": []
        }))
        .await
        .unwrap();
    assert!(trans_empty.contains("Transposed Matrix (0x0)"));

    // 4c. Ragged Matrix validation error handling
    let ragged_err = calc
        .execute(json!({
            "action": "matrix",
            "operation": "transpose",
            "matrix_a": [[1.0, 2.0], [3.0]]
        }))
        .await;
    assert!(ragged_err.is_err());
    assert!(ragged_err
        .unwrap_err()
        .to_string()
        .contains("Ragged matrix"));

    // 5. Matrix Determinant
    let det_res = calc
        .execute(json!({
            "action": "matrix",
            "operation": "det",
            "matrix_a": [[1.0, 2.0], [3.0, 4.0]]
        }))
        .await
        .unwrap();
    assert!(det_res.contains("-2.00000000"));

    // 5b. Ragged Determinant error handling
    let det_ragged = calc
        .execute(json!({
            "action": "matrix",
            "operation": "det",
            "matrix_a": [[1.0, 2.0, 3.0], [4.0, 5.0]]
        }))
        .await;
    assert!(det_ragged.is_err());

    // 6. Vector Dot Product
    let vec_res = calc
        .execute(json!({
            "action": "vector",
            "operation": "dot",
            "vector_a": [1.0, 2.0, 3.0],
            "vector_b": [4.0, 5.0, 6.0]
        }))
        .await
        .unwrap();
    assert!(vec_res.contains("32.00000000"));

    // 7. Finance PMT
    let fin_res = calc
        .execute(json!({
            "action": "finance",
            "operation": "pmt",
            "principal": 1000000.0,
            "rate": 0.048,
            "periods": 360.0
        }))
        .await
        .unwrap();
    assert!(fin_res.contains("Monthly Payment (月供)"));

    // 8. Unit Conversion
    let unit_res = calc
        .execute(json!({
            "action": "unit",
            "value": 100.0,
            "from_unit": "km",
            "to_unit": "m"
        }))
        .await
        .unwrap();
    assert!(unit_res.contains("100000"));

    // 9. Date diff
    let date_res = calc
        .execute(json!({
            "action": "date",
            "start_date": "2026-10-01",
            "end_date": "2026-10-15"
        }))
        .await
        .unwrap();
    assert!(date_res.contains("Calendar Days: 14 days"));
}

#[tokio::test]
async fn test_office_tool_suite() {
    let dir = tempdir().unwrap();
    let registry = ToolRegistry::with_standard_tools(dir.path());
    let office = registry.get("office").expect("office tool");

    let csv_sample = "name,role,salary,department\nAlice,Engineer,12000,Dev\nBob,Designer,9500,Design\nCharlie,PM,14000,Product\nDavid,Architect,18000,Dev\n";

    // 1. CSV to Markdown Table
    let md_res = office
        .execute(json!({
            "action": "csv_to_md",
            "content": csv_sample
        }))
        .await
        .unwrap();
    assert!(md_res.contains("name"));
    assert!(md_res.contains("salary"));
    assert!(md_res.contains("department"));
    assert!(md_res.contains("| Alice"));

    // 2. Table Query
    let query_res = office
        .execute(json!({
            "action": "csv_query",
            "content": csv_sample,
            "filter_col": "department",
            "filter_op": "=",
            "filter_val": "Dev",
            "select_cols": ["name", "salary"]
        }))
        .await
        .unwrap();
    assert!(query_res.contains("Alice"));
    assert!(query_res.contains("David"));
    assert!(!query_res.contains("Bob"));

    // 3. Table Column Statistics
    let stats_res = office
        .execute(json!({
            "action": "table_stats",
            "content": csv_sample,
            "target_col": "salary"
        }))
        .await
        .unwrap();
    assert!(stats_res.contains("\"sum\": 53500.0"));
    assert!(stats_res.contains("\"mean\": 13375.0"));

    // 4. ASCII Bar Chart
    let chart_res = office
        .execute(json!({
            "action": "chart_bar",
            "labels": ["Q1", "Q2", "Q3", "Q4"],
            "values": [120.0, 150.0, 90.0, 180.0]
        }))
        .await
        .unwrap();
    assert!(chart_res.contains("ASCII Visual Chart"));
    assert!(chart_res.contains("Trend Sparkline"));

    // 5. Slides Generation
    let slides_res = office
        .execute(json!({
            "action": "slides_gen",
            "title": "年度战略业务汇报",
            "num_slides": 4
        }))
        .await
        .unwrap();
    assert!(slides_res.contains("marp: true"));
    assert!(slides_res.contains("# 年度战略业务汇报"));

    // 6. Report Generation
    let report_res = office
        .execute(json!({
            "action": "report_gen",
            "report": {
                "title": "研发能效分析月报",
                "author": "测试",
                "date": "2026-10-24",
                "summary": "本月研发效率指标已根据输入数据生成。",
                "kpis": [["交付数量", "12"], ["缺陷率", "1.5%"]],
                "table": {
                    "headers": ["模块", "数量"],
                    "rows": [["核心服务", "8"], ["客户端", "4"]]
                }
            }
        }))
        .await
        .unwrap();
    assert!(report_res.contains("# 研发能效分析月报"));
    assert!(report_res.contains("执行摘要"));

    let invalid_filter = office
        .execute(json!({
            "action": "csv_query",
            "content": "name,value\na,1\n",
            "filter_col": "name",
            "filter_op": "unknown",
            "filter_val": "a"
        }))
        .await;
    assert!(invalid_filter.is_err());

    let invalid_report = office.execute(json!({ "action": "report_gen" })).await;
    assert!(invalid_report.is_err());

    let calc = registry.get("calc").expect("calc tool");
    let invalid_numbers = calc
        .execute(json!({ "action": "stat", "data": [1.0, "bad", 3.0] }))
        .await;
    assert!(invalid_numbers.is_err());
}

#[test]
fn test_skills_registry_coverage() {
    let registry = pi_agent_core::SkillRegistry::with_builtin_skills();
    let skills = registry.all();
    assert!(skills.len() >= 10);

    // Verify key categories exist
    assert!(skills
        .iter()
        .any(|s| s.category == pi_agent_core::SkillCategory::Office));
    assert!(skills
        .iter()
        .any(|s| s.category == pi_agent_core::SkillCategory::Scientific));
    assert!(skills
        .iter()
        .any(|s| s.category == pi_agent_core::SkillCategory::Financial));

    let appendix = registry.build_system_prompt_appendix();
    assert!(appendix.contains("Office Table & Spreadsheet Analytics"));
    assert!(appendix.contains("Scientific Math"));
}
