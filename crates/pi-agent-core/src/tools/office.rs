use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use super::Tool;

// ============================================================================
// 1. HIGH-PERFORMANCE TABLE & CSV ENGINE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct TableQuery<'a> {
    pub filter_col: Option<&'a str>,
    pub filter_op: Option<&'a str>,
    pub filter_val: Option<&'a str>,
    pub select_cols: Option<&'a [String]>,
    pub sort_by: Option<&'a str>,
    pub ascending: bool,
    pub limit: Option<usize>,
}

impl TableData {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self { headers, rows }
    }

    pub fn parse_csv(content: &str, delimiter: char) -> Result<Self> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_quotes {
                if ch == '"' {
                    if chars.peek() == Some(&'"') {
                        // Escaped quote: ""
                        chars.next();
                        current_field.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    current_field.push(ch);
                }
            } else {
                if ch == '"' {
                    in_quotes = true;
                } else if ch == delimiter {
                    current_row.push(current_field.trim().to_string());
                    current_field = String::new();
                } else if ch == '\n' || ch == '\r' {
                    if ch == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_row.push(current_field.trim().to_string());
                    current_field = String::new();
                    if !current_row.iter().all(|s| s.is_empty()) {
                        rows.push(current_row);
                    }
                    current_row = Vec::new();
                } else {
                    current_field.push(ch);
                }
            }
        }

        if in_quotes {
            bail!("Unterminated quoted CSV field");
        }
        if !current_field.is_empty() || !current_row.is_empty() {
            current_row.push(current_field.trim().to_string());
            if !current_row.iter().all(|s| s.is_empty()) {
                rows.push(current_row);
            }
        }

        if rows.is_empty() {
            bail!("CSV / TSV content is empty");
        }

        let headers = rows.remove(0);
        for (index, row) in rows.iter().enumerate() {
            if row.len() != headers.len() {
                bail!(
                    "CSV row {} has {} fields; expected {}",
                    index + 2,
                    row.len(),
                    headers.len()
                );
            }
        }
        Ok(Self { headers, rows })
    }

    pub fn to_csv(&self, delimiter: char) -> String {
        let mut out = String::new();
        let quote_field = |s: &str| -> String {
            if s.contains(delimiter) || s.contains('"') || s.contains('\n') || s.contains('\r') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };

        let header_line = self
            .headers
            .iter()
            .map(|h| quote_field(h))
            .collect::<Vec<_>>()
            .join(&delimiter.to_string());
        out.push_str(&header_line);
        out.push('\n');

        for row in &self.rows {
            let row_line = row
                .iter()
                .map(|f| quote_field(f))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string());
            out.push_str(&row_line);
            out.push('\n');
        }

        out
    }

    pub fn to_markdown(&self) -> String {
        if self.headers.is_empty() {
            return String::new();
        }

        let col_count = self.headers.len();
        let mut col_widths = vec![3; col_count];

        for (i, h) in self.headers.iter().enumerate() {
            col_widths[i] = col_widths[i].max(unicode_width(h));
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_count {
                    col_widths[i] = col_widths[i].max(unicode_width(cell));
                }
            }
        }

        let mut out = String::new();

        // Header row
        out.push_str("| ");
        for (i, h) in self.headers.iter().enumerate() {
            out.push_str(h);
            out.push_str(&" ".repeat(col_widths[i].saturating_sub(unicode_width(h))));
            out.push_str(" | ");
        }
        out.pop();
        out.push('\n');

        // Separator row
        out.push_str("| ");
        for width in &col_widths {
            out.push_str(&"-".repeat(*width));
            out.push_str(" | ");
        }
        out.pop();
        out.push('\n');

        // Data rows
        for row in &self.rows {
            out.push_str("| ");
            for (i, width) in col_widths.iter().enumerate().take(col_count) {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                out.push_str(cell);
                out.push_str(&" ".repeat(width.saturating_sub(unicode_width(cell))));
                out.push_str(" | ");
            }
            out.pop();
            out.push('\n');
        }

        out
    }

    pub fn parse_markdown(md: &str) -> Result<Self> {
        let lines: Vec<&str> = md
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.starts_with('|') && l.ends_with('|'))
            .collect();
        if lines.len() < 2 {
            bail!("Markdown table must contain at least a header row and a separator line");
        }

        let split_cells = |line: &str| -> Vec<String> {
            let inner = &line[1..line.len() - 1];
            inner.split('|').map(|s| s.trim().to_string()).collect()
        };

        let headers = split_cells(lines[0]);
        let mut rows = Vec::new();

        // Skip separator line at index 1
        for line in &lines[2..] {
            rows.push(split_cells(line));
        }

        Ok(Self { headers, rows })
    }

    pub fn query(&self, query: TableQuery<'_>) -> Result<Self> {
        let TableQuery {
            filter_col,
            filter_op,
            filter_val,
            select_cols,
            sort_by,
            ascending,
            limit,
        } = query;
        match (filter_col, filter_op, filter_val) {
            (None, None, None) => {}
            (
                Some(_),
                Some(
                    "=" | "==" | "eq" | "!=" | "neq" | "contains" | ">" | "gt" | "<" | "lt" | ">="
                    | "gte" | "<=" | "lte",
                ),
                Some(_),
            ) => {}
            (Some(_), Some(op), Some(_)) => bail!("Unknown filter operator: '{}'", op),
            _ => bail!("Filter requires column, operator and value together"),
        }
        for col in sort_by
            .into_iter()
            .chain(select_cols.into_iter().flatten().map(String::as_str))
        {
            if !self.headers.iter().any(|header| header == col) {
                bail!("Unknown column: '{}'", col);
            }
        }
        let col_idx_map: HashMap<&str, usize> = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.as_str(), i))
            .collect();

        // 1. Filter rows
        let filtered_rows: Vec<Vec<String>> =
            if let (Some(col), Some(op), Some(val)) = (filter_col, filter_op, filter_val) {
                let col_idx = *col_idx_map
                    .get(col)
                    .ok_or_else(|| anyhow!("Filter column '{}' not found in headers", col))?;
                self.rows
                    .iter()
                    .filter(|row| {
                        let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                        match op {
                            "=" | "==" | "eq" => cell == val,
                            "!=" | "neq" => cell != val,
                            "contains" => cell.to_lowercase().contains(&val.to_lowercase()),
                            ">" | "gt" => {
                                if let (Ok(c_num), Ok(v_num)) =
                                    (cell.parse::<f64>(), val.parse::<f64>())
                                {
                                    c_num > v_num
                                } else {
                                    cell > val
                                }
                            }
                            "<" | "lt" => {
                                if let (Ok(c_num), Ok(v_num)) =
                                    (cell.parse::<f64>(), val.parse::<f64>())
                                {
                                    c_num < v_num
                                } else {
                                    cell < val
                                }
                            }
                            ">=" | "gte" => {
                                if let (Ok(c_num), Ok(v_num)) =
                                    (cell.parse::<f64>(), val.parse::<f64>())
                                {
                                    c_num >= v_num
                                } else {
                                    cell >= val
                                }
                            }
                            "<=" | "lte" => {
                                if let (Ok(c_num), Ok(v_num)) =
                                    (cell.parse::<f64>(), val.parse::<f64>())
                                {
                                    c_num <= v_num
                                } else {
                                    cell <= val
                                }
                            }
                            _ => unreachable!("filter operator validated above"),
                        }
                    })
                    .cloned()
                    .collect()
            } else {
                self.rows.clone()
            };

        // 2. Sort rows
        let mut sorted_rows = filtered_rows;
        if let Some(sort_col) = sort_by {
            if let Some(&col_idx) = col_idx_map.get(sort_col) {
                sorted_rows.sort_by(|a, b| {
                    let val_a = a.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    let val_b = b.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    let ord = match (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                        (Ok(num_a), Ok(num_b)) => num_a
                            .partial_cmp(&num_b)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        _ => val_a.cmp(val_b),
                    };
                    if ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });
            }
        }

        // 3. Limit
        let limited_rows: Vec<Vec<String>> = if let Some(lim) = limit {
            sorted_rows.into_iter().take(lim).collect()
        } else {
            sorted_rows
        };

        // 4. Select columns
        if let Some(sel) = select_cols {
            let mut new_headers = Vec::new();
            let mut new_indices = Vec::new();
            for s in sel {
                if let Some(&idx) = col_idx_map.get(s.as_str()) {
                    new_headers.push(s.clone());
                    new_indices.push(idx);
                }
            }
            let projected_rows = limited_rows
                .into_iter()
                .map(|r| {
                    new_indices
                        .iter()
                        .map(|&i| r.get(i).cloned().unwrap_or_default())
                        .collect()
                })
                .collect();
            Ok(Self {
                headers: new_headers,
                rows: projected_rows,
            })
        } else {
            Ok(Self {
                headers: self.headers.clone(),
                rows: limited_rows,
            })
        }
    }

    pub fn column_stats(&self, col_name: &str) -> Result<Value> {
        let col_idx = self
            .headers
            .iter()
            .position(|h| h == col_name)
            .ok_or_else(|| anyhow!("Column '{}' not found in table", col_name))?;

        let mut nums = Vec::new();
        let mut non_empty_count = 0;

        for row in &self.rows {
            if let Some(val) = row.get(col_idx) {
                if !val.trim().is_empty() {
                    non_empty_count += 1;
                    if let Ok(n) = val.trim().replace(',', "").parse::<f64>() {
                        nums.push(n);
                    }
                }
            }
        }

        if nums.is_empty() {
            return Ok(json!({
                "column": col_name,
                "total_rows": self.rows.len(),
                "non_empty_rows": non_empty_count,
                "is_numeric": false,
            }));
        }

        nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let count = nums.len();
        let sum: f64 = nums.iter().sum();
        let mean = sum / (count as f64);
        let min = nums[0];
        let max = nums[count - 1];
        let median = if count % 2 == 1 {
            nums[count / 2]
        } else {
            (nums[count / 2 - 1] + nums[count / 2]) / 2.0
        };

        Ok(json!({
            "column": col_name,
            "total_rows": self.rows.len(),
            "numeric_count": count,
            "sum": sum,
            "mean": mean,
            "median": median,
            "min": min,
            "max": max,
            "range": max - min
        }))
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().map(|c| if c as u32 > 0x7f { 2 } else { 1 }).sum()
}

// ============================================================================
// 2. PURE RUST DOCX & ZIP EXTRACTOR
// ============================================================================

pub struct DocxReader;

impl DocxReader {
    pub fn extract_text_from_file(path: &Path) -> Result<String> {
        let mut file = File::open(path)
            .with_context(|| format!("Cannot open DOCX file: {}", path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Self::extract_text_from_bytes(&buffer)
    }

    pub fn extract_text_from_bytes(bytes: &[u8]) -> Result<String> {
        let xml_content = Self::extract_zip_entry(bytes, "word/document.xml").ok_or_else(|| {
            anyhow!("Invalid DOCX archive: 'word/document.xml' not found in package")
        })?;
        let xml_str = String::from_utf8_lossy(&xml_content);
        Ok(Self::parse_word_document_xml(&xml_str))
    }

    /// Extract a file entry from ZIP byte stream in pure Rust
    fn extract_zip_entry(bytes: &[u8], target_name: &str) -> Option<Vec<u8>> {
        let mut cursor = 0;
        let len = bytes.len();

        while cursor + 30 <= len {
            // Check for Local File Header Signature: PK\x03\x04
            if bytes[cursor..cursor + 4] != [0x50, 0x4b, 0x03, 0x04] {
                // Not a local file header, advance by 1
                cursor += 1;
                continue;
            }

            let comp_method = u16::from_le_bytes([bytes[cursor + 8], bytes[cursor + 9]]);
            let comp_size = u32::from_le_bytes([
                bytes[cursor + 18],
                bytes[cursor + 19],
                bytes[cursor + 20],
                bytes[cursor + 21],
            ]) as usize;
            let file_name_len =
                u16::from_le_bytes([bytes[cursor + 26], bytes[cursor + 27]]) as usize;
            let extra_field_len =
                u16::from_le_bytes([bytes[cursor + 28], bytes[cursor + 29]]) as usize;

            let name_start = cursor + 30;
            let name_end = name_start + file_name_len;
            if name_end > len {
                break;
            }

            let file_name = String::from_utf8_lossy(&bytes[name_start..name_end]);
            let data_start = name_end + extra_field_len;
            let data_end = data_start + comp_size;

            if file_name == target_name && data_end <= len {
                let comp_data = &bytes[data_start..data_end];
                if comp_method == 0 {
                    // Stored (no compression)
                    return Some(comp_data.to_vec());
                } else if comp_method == 8 {
                    // Deflate compression
                    let mut decoder = DeflateDecoder::new(Cursor::new(comp_data));
                    let mut decompressed = Vec::new();
                    if decoder.read_to_end(&mut decompressed).is_ok() {
                        return Some(decompressed);
                    }
                }
            }

            cursor = data_end;
        }

        None
    }

    /// Fast XML tag parsing to extract structured paragraphs and tables
    fn parse_word_document_xml(xml: &str) -> String {
        let mut out = String::new();
        let mut current_tag = String::new();
        let mut current_text = String::new();

        let chars: Vec<char> = xml.chars().collect();
        let mut i = 0;
        let n = chars.len();

        while i < n {
            let ch = chars[i];
            if ch == '<' {
                current_tag.clear();
                i += 1;
                while i < n && chars[i] != '>' {
                    current_tag.push(chars[i]);
                    i += 1;
                }

                // Handle tag events
                let tag_name = current_tag.split_whitespace().next().unwrap_or("");
                if tag_name == "/w:p" || tag_name == "w:p/" {
                    if !current_text.trim().is_empty() {
                        out.push_str(current_text.trim());
                        out.push_str("\n\n");
                        current_text.clear();
                    }
                } else if tag_name == "w:tab" || tag_name == "w:tab/" {
                    current_text.push('\t');
                } else if tag_name == "w:br" || tag_name == "w:br/" {
                    current_text.push('\n');
                } else if tag_name == "/w:tc" {
                    current_text.push_str(" | ");
                } else if tag_name == "/w:tr" {
                    out.push_str(&format!("| {}\n", current_text.trim()));
                    current_text.clear();
                }
            } else {
                current_text.push(ch);
            }
            i += 1;
        }

        if !current_text.trim().is_empty() {
            out.push_str(current_text.trim());
            out.push('\n');
        }

        out
    }

    pub fn compute_doc_stats(text: &str) -> Value {
        let chars_count = text.chars().count();
        let chars_no_space = text.chars().filter(|c| !c.is_whitespace()).count();
        let words_count = text.split_whitespace().count();
        let lines_count = text.lines().count();
        let paragraphs_count = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
        let reading_time_minutes = (words_count as f64) / 250.0; // Standard 250 wpm

        json!({
            "characters_total": chars_count,
            "characters_no_whitespace": chars_no_space,
            "words": words_count,
            "lines": lines_count,
            "paragraphs": paragraphs_count,
            "estimated_reading_time_min": format!("{:.1}", reading_time_minutes.max(0.5))
        })
    }
}

// ============================================================================
// 3. SLIDES & PRESENTATION GENERATOR (Marp / Slidev format)
// ============================================================================

pub struct SlidesGenerator;

impl SlidesGenerator {
    pub fn generate_markdown_slides(
        title: &str,
        topic: &str,
        num_slides: usize,
        author: Option<&str>,
    ) -> String {
        let mut out = String::new();

        // Frontmatter
        out.push_str("---\n");
        out.push_str("marp: true\n");
        out.push_str("theme: default\n");
        out.push_str("paginate: true\n");
        out.push_str("backgroundColor: #121217\n");
        out.push_str("color: #e4e4e7\n");
        out.push_str("---\n\n");

        // Slide 1: Title Slide
        out.push_str(&format!("# {}\n\n", title));
        out.push_str(&format!("### {}\n\n", topic));
        if let Some(auth) = author {
            out.push_str(&format!("*汇报人: {}*\n", auth));
        }
        out.push_str(
            "\n<!-- note: 演讲开场：欢迎各位并介绍本次分享的主题背景与核心目标。 -->\n\n---\n\n",
        );

        // Slide 2: Agenda
        out.push_str("## 目录与议程\n\n");
        out.push_str("1. **背景与战略定位**：行业痛点与核心机遇\n");
        out.push_str("2. **核心方案架构**：极速高性能原生能力设计\n");
        out.push_str("3. **实测数据与成果**：关键指标与基准对比\n");
        out.push_str("4. **未来演进规划**：阶段目标与落地路线\n\n");
        out.push_str(
            "<!-- note: 简明扼要概述本次分享的四个主要篇章，让听众建立全局视野。 -->\n\n---\n\n",
        );

        // Remaining slides
        let topics: &[(&str, &str, &[&str])] = &[
            (
                "背景与目标",
                "请填写业务背景",
                &["填写目标与范围", "说明数据来源与统计周期"],
            ),
            (
                "方案与执行",
                "请填写实际方案",
                &["记录已实施工作", "说明未完成事项"],
            ),
            (
                "数据与结论",
                "请填写实测数据",
                &["附上测量环境、方法与样本", "仅给出数据支持的结论"],
            ),
            (
                "后续行动",
                "请填写负责人及时间",
                &["列出待解决问题", "明确验收标准"],
            ),
        ];

        let target_slides = num_slides.max(3);
        for i in 0..(target_slides - 2) {
            let (sec_title, subtitle, bullets) = topics[i % topics.len()];
            out.push_str(&format!("## {}\n\n", sec_title));
            out.push_str(&format!("*{}: 关键要点解析*\n\n", subtitle));
            for b in bullets {
                out.push_str(&format!("* {}\n", b));
            }
            out.push_str("\n> 请结合输入数据补充有依据的结论。\n\n");
            out.push_str(&format!(
                "<!-- note: 深入阐述第 {} 部分的落地细节与实施保障。 -->\n\n---\n\n",
                i + 1
            ));
        }

        // Final Slide: Q&A
        out.push_str("# 谢谢倾听 / Q&A\n\n");
        out.push_str("欢迎提问与深入探讨！\n\n");
        out.push_str("*Powered by Volt Native AI Desktop*\n");

        out
    }
}

// ============================================================================
// 4. ASCII / UNICODE VISUALIZATION CHART GENERATOR
// ============================================================================

pub struct AsciiChart;

impl AsciiChart {
    pub fn bar_chart(labels: &[String], values: &[f64], max_bar_width: usize) -> Result<String> {
        if labels.len() != values.len() || labels.is_empty() {
            bail!("Labels and values must be non-empty and have matching lengths");
        }

        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_val = values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
            .min(0.0);
        let max_label_len = labels.iter().map(|l| unicode_width(l)).max().unwrap_or(10);

        let range = (max_val - min_val).max(1e-9);
        let mut out = String::new();

        for (label, &val) in labels.iter().zip(values.iter()) {
            let bar_len = (((val - min_val) / range) * (max_bar_width as f64)).round() as usize;
            let bar = "█".repeat(bar_len);
            let padding = " ".repeat(max_label_len.saturating_sub(unicode_width(label)));
            out.push_str(&format!("{} {} | {} {:.2}\n", label, padding, bar, val));
        }

        Ok(out)
    }

    pub fn sparkline(values: &[f64]) -> String {
        if values.is_empty() {
            return String::new();
        }
        let ticks = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max - min).max(1e-9);

        values
            .iter()
            .map(|&v| {
                let idx = (((v - min) / range) * ((ticks.len() - 1) as f64)).round() as usize;
                ticks[idx.min(ticks.len() - 1)]
            })
            .collect()
    }
}

// ============================================================================
// 5. OFFICE REPORT GENERATOR
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportInput {
    pub title: String,
    pub author: String,
    pub date: String,
    pub summary: String,
    pub kpis: Vec<(String, String)>,
    pub table: TableData,
}

impl ReportInput {
    pub fn render(&self) -> Result<String> {
        if [&self.title, &self.author, &self.date, &self.summary]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            bail!("Report title, author, date and summary must not be empty");
        }
        if self.kpis.is_empty()
            || self
                .kpis
                .iter()
                .any(|(name, value)| name.trim().is_empty() || value.trim().is_empty())
        {
            bail!("Report requires nonempty KPI names and values");
        }
        if self.table.headers.is_empty()
            || self.table.rows.is_empty()
            || self
                .table
                .rows
                .iter()
                .any(|row| row.len() != self.table.headers.len())
        {
            bail!("Report requires a nonempty rectangular detail table");
        }
        let mut out = format!("# {}\n\n**编制人**: {} | **编制日期**: {}\n\n## 执行摘要 (Executive Summary)\n\n{}\n\n## 核心指标 (KPI Scorecard)\n\n", self.title, self.author, self.date, self.summary);
        let kpis = TableData::new(
            vec!["指标".into(), "数值".into()],
            self.kpis
                .iter()
                .map(|(name, value)| vec![name.clone(), value.clone()])
                .collect(),
        );
        out.push_str(&kpis.to_markdown());
        out.push_str("\n## 明细数据 (Detailed Dataset)\n\n");
        out.push_str(&self.table.to_markdown());
        Ok(out)
    }
}

// ============================================================================
// 6. OFFICE AGENT TOOL IMPLEMENTATION
// ============================================================================

fn optional_argument<T: serde::de::DeserializeOwned>(
    arguments: &Value,
    name: &str,
) -> Result<Option<T>> {
    arguments
        .get(name)
        .map(|value| {
            serde_json::from_value(value.clone())
                .with_context(|| format!("Invalid '{name}' parameter"))
        })
        .transpose()
}

pub struct OfficeTool {
    pub workspace_dir: PathBuf,
}

impl OfficeTool {
    pub fn new(workspace_dir: impl Into<PathBuf>) -> Self {
        Self {
            workspace_dir: workspace_dir.into(),
        }
    }
}

#[async_trait]
impl Tool for OfficeTool {
    fn name(&self) -> &'static str {
        "office"
    }

    fn description(&self) -> &'static str {
        "Native Rust high-performance office suite tool. Supports querying/filtering/aggregating CSV and TSV tables, two-way Markdown table conversion, DOCX text extraction, presentation slide deck generation (Marp/Slidev), ASCII/Unicode charts, and formal office reports."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "report": {
                    "type": "object",
                    "required": ["title", "author", "date", "summary", "kpis", "table"],
                    "properties": {
                        "title": {"type": "string"}, "author": {"type": "string"}, "date": {"type": "string"}, "summary": {"type": "string"},
                        "kpis": {"type": "array", "items": {"type": "array", "items": {"type": "string"}, "minItems": 2, "maxItems": 2}},
                        "table": {"type": "object", "required": ["headers", "rows"], "properties": {"headers": {"type": "array", "items": {"type": "string"}}, "rows": {"type": "array", "items": {"type": "array", "items": {"type": "string"}}}}}
                    }
                },
                "action": {
                    "type": "string",
                    "enum": ["csv_query", "csv_to_md", "md_to_csv", "table_stats", "docx_read", "docx_stats", "slides_gen", "chart_bar", "report_gen"],
                    "description": "Office action: 'csv_query' (filter/sort/query CSV data), 'csv_to_md' (convert CSV to Markdown table), 'md_to_csv' (convert Markdown table to CSV), 'table_stats' (compute column statistics), 'docx_read' (read text from .docx file), 'docx_stats' (word/char count of docx or text), 'slides_gen' (generate slide deck), 'chart_bar' (generate ASCII bar chart), 'report_gen' (generate complete office report)"
                },
                "file_path": {
                    "type": "string",
                    "description": "Path to target file (CSV, TSV, DOCX, or Markdown) relative to workspace or absolute"
                },
                "content": {
                    "type": "string",
                    "description": "Inline CSV, TSV, or Markdown content to process directly"
                },
                "delimiter": {
                    "type": "string",
                    "description": "Delimiter for CSV/TSV, default is ',' (or '\\t' for TSV)"
                },
                "filter_col": { "type": "string", "description": "Column name to filter by" },
                "filter_op": { "type": "string", "description": "Filter operator: '=', '!=', 'contains', '>', '<', '>=', '<='" },
                "filter_val": { "type": "string", "description": "Value to match for filter" },
                "select_cols": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of columns to select in output"
                },
                "sort_by": { "type": "string", "description": "Column name to sort by" },
                "ascending": { "type": "boolean", "description": "Sort direction (default true)" },
                "limit": { "type": "integer", "description": "Max rows to return" },
                "target_col": { "type": "string", "description": "Target column name for 'table_stats'" },
                "title": { "type": "string", "description": "Title for slide deck or report" },
                "topic": { "type": "string", "description": "Topic or subtitle for slides/report" },
                "num_slides": { "type": "integer", "description": "Number of slides to generate (default 5)" },
                "labels": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Labels for bar chart"
                },
                "values": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Values for bar chart"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'action' parameter"))?;

        let delimiter = arguments
            .get("delimiter")
            .and_then(|v| v.as_str())
            .and_then(|s| s.chars().next())
            .unwrap_or(',');

        let load_table = || -> Result<TableData> {
            if let Some(content) = arguments.get("content").and_then(|v| v.as_str()) {
                TableData::parse_csv(content, delimiter)
            } else if let Some(path_str) = arguments.get("file_path").and_then(|v| v.as_str()) {
                let path = self.workspace_dir.join(path_str);
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {}", path.display()))?;
                TableData::parse_csv(&text, delimiter)
            } else {
                bail!("Either 'content' or 'file_path' must be provided for table operations")
            }
        };

        match action {
            "csv_query" => {
                let table = load_table()?;
                let filter_col: Option<String> = optional_argument(&arguments, "filter_col")?;
                let filter_op: Option<String> = optional_argument(&arguments, "filter_op")?;
                let filter_val: Option<String> = optional_argument(&arguments, "filter_val")?;
                let select_cols: Option<Vec<String>> =
                    optional_argument(&arguments, "select_cols")?;
                let sort_by: Option<String> = optional_argument(&arguments, "sort_by")?;
                let ascending = optional_argument(&arguments, "ascending")?.unwrap_or(true);
                let limit = optional_argument(&arguments, "limit")?;
                let queried = table.query(TableQuery {
                    filter_col: filter_col.as_deref(),
                    filter_op: filter_op.as_deref(),
                    filter_val: filter_val.as_deref(),
                    select_cols: select_cols.as_deref(),
                    sort_by: sort_by.as_deref(),
                    ascending,
                    limit,
                })?;

                let md = queried.to_markdown();
                Ok(format!(
                    "📋 Queried Table Result ({} rows):\n\n{}",
                    queried.rows.len(),
                    md
                ))
            }
            "csv_to_md" => {
                let table = load_table()?;
                Ok(table.to_markdown())
            }
            "md_to_csv" => {
                let md_text = if let Some(c) = arguments.get("content").and_then(|v| v.as_str()) {
                    c.to_string()
                } else if let Some(path_str) = arguments.get("file_path").and_then(|v| v.as_str()) {
                    let path = self.workspace_dir.join(path_str);
                    std::fs::read_to_string(&path)?
                } else {
                    bail!("Missing 'content' or 'file_path'")
                };
                let table = TableData::parse_markdown(&md_text)?;
                Ok(table.to_csv(delimiter))
            }
            "table_stats" => {
                let table = load_table()?;
                let col = arguments
                    .get("target_col")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'target_col' for table_stats"))?;
                let stats = table.column_stats(col)?;
                Ok(serde_json::to_string_pretty(&stats)?)
            }
            "docx_read" => {
                let path_str = arguments
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'file_path' for docx_read"))?;
                let path = self.workspace_dir.join(path_str);
                let text = DocxReader::extract_text_from_file(&path)?;
                Ok(format!(
                    "📄 Extracted text from DOCX ({}):\n\n{}",
                    path_str, text
                ))
            }
            "docx_stats" => {
                let text =
                    if let Some(path_str) = arguments.get("file_path").and_then(|v| v.as_str()) {
                        let path = self.workspace_dir.join(path_str);
                        if path.extension().and_then(|s| s.to_str()) == Some("docx") {
                            DocxReader::extract_text_from_file(&path)?
                        } else {
                            std::fs::read_to_string(&path)?
                        }
                    } else if let Some(c) = arguments.get("content").and_then(|v| v.as_str()) {
                        c.to_string()
                    } else {
                        bail!("Missing 'file_path' or 'content'")
                    };
                let stats = DocxReader::compute_doc_stats(&text);
                Ok(format!(
                    "📊 Document Statistics:\n{}",
                    serde_json::to_string_pretty(&stats)?
                ))
            }
            "slides_gen" => {
                let title = arguments
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("年度战略与业务规划汇报");
                let topic = arguments
                    .get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("面向高绩效交付的工程赋能");
                let num_slides = arguments
                    .get("num_slides")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
                    .unwrap_or(5);
                let slides_md = SlidesGenerator::generate_markdown_slides(
                    title,
                    topic,
                    num_slides,
                    Some("Volt Native Agent"),
                );
                Ok(slides_md)
            }
            "chart_bar" => {
                let labels: Vec<String> = arguments
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing 'labels' array"))?
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| anyhow!("Chart labels must be strings"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let values: Vec<f64> = arguments
                    .get("values")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing 'values' array"))?
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| anyhow!("Chart values must be numbers"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let chart = AsciiChart::bar_chart(&labels, &values, 30)?;
                let spark = AsciiChart::sparkline(&values);
                Ok(format!(
                    "📊 ASCII Visual Chart:\n\n{}\nTrend Sparkline: {}\n",
                    chart, spark
                ))
            }
            "report_gen" => {
                let report: ReportInput = serde_json::from_value(
                    arguments
                        .get("report")
                        .cloned()
                        .ok_or_else(|| anyhow!("Missing 'report' object"))?,
                )?;
                report.render()
            }
            unknown => bail!("Unknown office action: '{}'", unknown),
        }
    }
}
