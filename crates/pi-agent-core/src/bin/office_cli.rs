use anyhow::{bail, Result};
use pi_agent_core::tools::calc::parse_numbers;
use pi_agent_core::tools::office::{
    AsciiChart, DocxReader, ReportInput, SlidesGenerator, TableData, TableQuery,
};
use std::fs;
use std::path::PathBuf;

fn print_help() {
    println!(
        r#"⚡ office-cli: Native Rust High-Performance Office Productivity Tool
======================================================================
Usage: office-cli <COMMAND> [OPTIONS]

COMMANDS:
  table query   --file <PATH> [--filter <COL=VAL>] [--select <COLS>] [--sort <COL>] [--limit <N>]
                Query, filter, slice, and sort CSV/TSV table datasets
  table to-md   --file <PATH>
                Convert CSV/TSV file to GitHub-formatted Markdown table
  table to-csv  --file <PATH>
                Convert Markdown table file to clean CSV
  table stats   --file <PATH> --col <COL_NAME>
                Calculate summary statistics (sum, mean, median, min, max) for a column
  docx read     --file <PATH>
                Extract all plain text, paragraphs, and tables from Word .docx file
  docx stats    --file <PATH>
                Calculate word count, character count, and estimated reading time
  slides gen    --title <TITLE> [--topic <TOPIC>] [--slides <N>] [--out <FILE>]
                Generate Marp/Slidev presentation deck in Markdown
  chart bar     --labels <L1,L2,...> --values <V1,V2,...>
                Render Unicode/ASCII horizontal bar chart and sparkline
  report gen    --file <REPORT_JSON> [--out <FILE>]
                Generate formal Markdown business/technical analysis report

FLAGS:
  --help, -h    Show this help message
"#
    );
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    let cmd1 = args.first().map(|s| s.as_str()).unwrap_or("");
    let cmd2 = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match (cmd1, cmd2) {
        ("table", "query") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            if file.is_empty() {
                bail!("Missing --file argument");
            }
            let content = fs::read_to_string(&file)?;
            let delim = if file.ends_with(".tsv") { '\t' } else { ',' };
            let table = TableData::parse_csv(&content, delim)?;

            let filter_str = get_arg(&args, "--filter");
            let (filter_col, filter_op, filter_val) = if let Some(ref f) = filter_str {
                if let Some(pos) = f.find("!=") {
                    (Some(&f[..pos]), Some("!="), Some(&f[pos + 2..]))
                } else if let Some(pos) = f.find(">=") {
                    (Some(&f[..pos]), Some(">="), Some(&f[pos + 2..]))
                } else if let Some(pos) = f.find("<=") {
                    (Some(&f[..pos]), Some("<="), Some(&f[pos + 2..]))
                } else if let Some(pos) = f.find('=') {
                    (Some(&f[..pos]), Some("="), Some(&f[pos + 1..]))
                } else if let Some(pos) = f.find('>') {
                    (Some(&f[..pos]), Some(">"), Some(&f[pos + 1..]))
                } else if let Some(pos) = f.find('<') {
                    (Some(&f[..pos]), Some("<"), Some(&f[pos + 1..]))
                } else {
                    bail!("Invalid filter expression: {}", f)
                }
            } else {
                (None, None, None)
            };

            let select_cols: Option<Vec<String>> = get_arg(&args, "--select")
                .map(|s| s.split(',').map(|c| c.trim().to_string()).collect());
            let sort_by = get_arg(&args, "--sort");
            let limit = if args.iter().any(|arg| arg == "--limit") {
                Some(
                    get_arg(&args, "--limit")
                        .ok_or_else(|| anyhow::anyhow!("Missing --limit value"))?
                        .parse::<usize>()
                        .map_err(|_| anyhow::anyhow!("Invalid --limit value"))?,
                )
            } else {
                None
            };

            let res = table.query(TableQuery {
                filter_col: filter_col.map(str::trim),
                filter_op,
                filter_val: filter_val.map(str::trim),
                select_cols: select_cols.as_deref(),
                sort_by: sort_by.as_deref(),
                ascending: true,
                limit,
            })?;

            println!("{}", res.to_markdown());
        }
        ("table", "to-md") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            if file.is_empty() {
                bail!("Missing --file argument");
            }
            let content = fs::read_to_string(&file)?;
            let delim = if file.ends_with(".tsv") { '\t' } else { ',' };
            let table = TableData::parse_csv(&content, delim)?;
            println!("{}", table.to_markdown());
        }
        ("table", "to-csv") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            if file.is_empty() {
                bail!("Missing --file argument");
            }
            let content = fs::read_to_string(&file)?;
            let table = TableData::parse_markdown(&content)?;
            println!("{}", table.to_csv(','));
        }
        ("table", "stats") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            let col = get_arg(&args, "--col").unwrap_or_default();
            if file.is_empty() || col.is_empty() {
                bail!("Both --file and --col arguments are required");
            }
            let content = fs::read_to_string(&file)?;
            let delim = if file.ends_with(".tsv") { '\t' } else { ',' };
            let table = TableData::parse_csv(&content, delim)?;
            let stats = table.column_stats(&col)?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        ("docx", "read") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            if file.is_empty() {
                bail!("Missing --file argument");
            }
            let text = DocxReader::extract_text_from_file(&PathBuf::from(file))?;
            println!("{}", text);
        }
        ("docx", "stats") => {
            let file = get_arg(&args, "--file")
                .unwrap_or_else(|| get_arg(&args, "-f").unwrap_or_default());
            if file.is_empty() {
                bail!("Missing --file argument");
            }
            let p = PathBuf::from(&file);
            let text = if p.extension().and_then(|s| s.to_str()) == Some("docx") {
                DocxReader::extract_text_from_file(&p)?
            } else {
                fs::read_to_string(&p)?
            };
            let stats = DocxReader::compute_doc_stats(&text);
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        ("slides", "gen") | ("slides", "generate") => {
            let title =
                get_arg(&args, "--title").unwrap_or_else(|| "年度战略与业务规划汇报".to_string());
            let topic =
                get_arg(&args, "--topic").unwrap_or_else(|| "面向高绩效交付的工程赋能".to_string());
            let slides_count: usize = get_arg(&args, "--slides")
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);
            let md = SlidesGenerator::generate_markdown_slides(
                &title,
                &topic,
                slides_count,
                Some("Volt Office"),
            );

            if let Some(out_path) = get_arg(&args, "--out") {
                fs::write(&out_path, &md)?;
                println!("✅ Presentation deck saved to: {}", out_path);
            } else {
                println!("{}", md);
            }
        }
        ("chart", "bar") => {
            let labels_str =
                get_arg(&args, "--labels").ok_or_else(|| anyhow::anyhow!("Missing --labels"))?;
            let values_str =
                get_arg(&args, "--values").ok_or_else(|| anyhow::anyhow!("Missing --values"))?;
            let labels: Vec<String> = labels_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let values: Vec<f64> = parse_numbers(&values_str)?;
            let chart = AsciiChart::bar_chart(&labels, &values, 30)?;
            let spark = AsciiChart::sparkline(&values);
            println!(
                "📊 ASCII Bar Chart:\n\n{}\nTrend Sparkline: {}\n",
                chart, spark
            );
        }
        ("report", "gen") | ("report", "generate") => {
            let file = get_arg(&args, "--file")
                .ok_or_else(|| anyhow::anyhow!("Missing --file report JSON"))?;
            let input: ReportInput = serde_json::from_str(&fs::read_to_string(file)?)?;
            let report = input.render()?;
            if let Some(out_path) = get_arg(&args, "--out") {
                fs::write(&out_path, &report)?;
                println!("✅ Office report saved to: {}", out_path);
            } else {
                println!("{}", report);
            }
        }
        _ => {
            bail!(
                "Unknown command: '{} {}'. Run with --help for usage.",
                cmd1,
                cmd2
            );
        }
    }

    Ok(())
}
