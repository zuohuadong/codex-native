use pi_agent_core::tools::ToolRegistry;
use serde_json::json;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn numeric_cli_rejects_invalid_data_without_partial_results() {
    for input in ["1,bad,3", "1,,3", "", "NaN,1", "inf,2"] {
        let output = Command::new(env!("CARGO_BIN_EXE_calc-cli"))
            .args(["stat", input])
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {input}");
        assert!(output.stdout.is_empty(), "printed partial statistics");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_calc-cli"))
        .args(["stat", "1,2,3"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Mean:      2.0000"));
    let output = Command::new(env!("CARGO_BIN_EXE_calc-cli"))
        .args(["finance", "pmt", "--rate", "oops"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[tokio::test]
async fn report_preserves_input_without_fabricated_metrics() {
    let dir = tempdir().unwrap();
    let office = ToolRegistry::with_standard_tools(dir.path())
        .get("office")
        .unwrap();
    let report = json!({"title":"Supplied title", "author":"Author", "date":"2030-01-02", "summary":"Recorded observation", "kpis":[["Count", "7"]], "table":{"headers":["Item", "Count"], "rows":[["Actual", "7"]]}});
    let text = office
        .execute(json!({"action":"report_gen", "report":report}))
        .await
        .unwrap();
    for expected in [
        "Supplied title",
        "2030-01-02",
        "Recorded observation",
        "Actual",
        "7",
    ] {
        assert!(text.contains(expected));
    }
    for fabricated in ["18.2 MB", "128,000", "正常达标", "2026-10-20"] {
        assert!(!text.contains(fabricated));
    }
    for missing in ["title", "author", "date", "summary", "kpis", "table"] {
        let mut invalid = report.clone();
        invalid.as_object_mut().unwrap().remove(missing);
        assert!(office
            .execute(json!({"action":"report_gen", "report":invalid}))
            .await
            .is_err());
    }
    let file = dir.path().join("report.json");
    std::fs::write(&file, report.to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_office-cli"))
        .args(["report", "gen", "--file"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim_end(),
        text.trim_end()
    );
}

#[tokio::test]
async fn queries_reject_invalid_and_incomplete_filters_even_on_empty_tables() {
    let dir = tempdir().unwrap();
    let office = ToolRegistry::with_standard_tools(dir.path())
        .get("office")
        .unwrap();
    for content in ["name,value\na,1\n", "name,value\n"] {
        for fields in [
            json!({"filter_col":"name", "filter_op":"typo", "filter_val":"a"}),
            json!({"filter_col":"name"}),
            json!({"limit":-1}),
            json!({"limit":1.5}),
            json!({"ascending":"false"}),
            json!({"filter_col":42}),
            json!({"select_cols":"name"}),
            json!({"sort_by":"missing"}),
            json!({"select_cols":["missing"]}),
            json!({"select_cols":["name", 42]}),
        ] {
            let mut request = json!({"action":"csv_query", "content":content});
            request
                .as_object_mut()
                .unwrap()
                .extend(fields.as_object().unwrap().clone());
            assert!(office.execute(request).await.is_err());
        }
    }
}

#[tokio::test]
async fn regression_and_chart_do_not_realign_invalid_samples() {
    let dir = tempdir().unwrap();
    let registry = ToolRegistry::with_standard_tools(dir.path());
    assert!(registry
        .get("calc")
        .unwrap()
        .execute(json!({"action":"regression", "x":[1,"bad",3], "y":["bad",2,6]}))
        .await
        .is_err());
    assert!(registry
        .get("office")
        .unwrap()
        .execute(json!({"action":"chart_bar", "labels":["a",1,"b"], "values":[1,2,"bad"]}))
        .await
        .is_err());
}

#[tokio::test]
async fn malformed_csv_is_rejected_without_dropping_cells() {
    let dir = tempdir().unwrap();
    let office = ToolRegistry::with_standard_tools(dir.path())
        .get("office")
        .unwrap();
    for content in ["a,b\n1,2,3\n", "a,b\n1\n", "a,b\n\"unfinished,2"] {
        assert!(office
            .execute(json!({"action":"csv_query", "content":content}))
            .await
            .is_err());
    }
    let file = dir.path().join("input.csv");
    std::fs::write(&file, "a,b\n1,2\n").unwrap();
    for limit in ["bad", "-1", "1.5"] {
        let output = Command::new(env!("CARGO_BIN_EXE_office-cli"))
            .args(["table", "query", "--file"])
            .arg(&file)
            .args(["--limit", limit])
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
}
