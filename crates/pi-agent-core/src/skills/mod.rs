use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Office,
    Scientific,
    Financial,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliExample {
    pub title: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub name_en: String,
    pub category: SkillCategory,
    pub icon: String,
    pub description: String,
    pub description_en: String,
    pub system_prompt_snippet: String,
    pub cli_examples: Vec<CliExample>,
}

#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn with_builtin_skills() -> Self {
        let mut reg = Self::new();

        // 1. Office Table & Spreadsheet Analytics
        reg.register(Skill {
            id: "office-table-analytics".to_string(),
            name: "办公表格与数据分析 (Table & Spreadsheet)".to_string(),
            name_en: "Office Table & Spreadsheet Analytics".to_string(),
            category: SkillCategory::Office,
            icon: "📊".to_string(),
            description: "纯 Rust 高性能 CSV/TSV 表格处理、条件过滤、多列统计、Markdown 表格互转及数据提取。".to_string(),
            description_en: "Pure Rust high-performance CSV/TSV processing, multi-column filtering, statistics, and bidirectional Markdown table conversion.".to_string(),
            system_prompt_snippet: "\
### Skill: 办公表格与数据分析 (Office Table & Spreadsheet Analytics)\n\
- 当用户需要处理数据表、CSV/TSV 文件或计算指标时，优先调用 `office` 工具。\n\
- 支持操作：`csv_query` (过滤/排序/投影)、`table_stats` (计算列总和/均值/中位数)、`csv_to_md` (转格式化Markdown)、`md_to_csv` (转CSV)。\n\
- 遇到复杂报表时，先进行统计聚合，再通过格式化表格清晰呈现。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "CSV 聚合与列统计".to_string(),
                    command: "office-cli table stats --file sales.csv --col revenue".to_string(),
                    description: "高速计算指定列的总和、平均值、中位数和数值极差。".to_string(),
                },
                CliExample {
                    title: "CSV 转美化 Markdown 表格".to_string(),
                    command: "office-cli table to-md --file data.csv".to_string(),
                    description: "自动计算中英文字符宽度对齐并输出 GitHub 兼容的 Markdown 表格。".to_string(),
                },
                CliExample {
                    title: "条件过滤与字段截取".to_string(),
                    command: "office-cli table query --file users.csv --filter \"status=active\" --select \"name,email\"".to_string(),
                    description: "毫秒级筛选并输出符合条件的子表格。".to_string(),
                },
            ],
        });

        // 2. Office Document & Report Generator
        reg.register(Skill {
            id: "office-doc-report".to_string(),
            name: "公文报告与文档提取 (Document & Report)".to_string(),
            name_en: "Office Document & Report Generator".to_string(),
            category: SkillCategory::Office,
            icon: "📑".to_string(),
            description: "Word (.docx) 原生解包文本提取、文档字数统计、正式 Markdown 商务与技术报告自动生成。".to_string(),
            description_en: "Native Word (.docx) decompression & text extraction, reading-time statistics, and executive Markdown report generation.".to_string(),
            system_prompt_snippet: "\
### Skill: 公文报告与文档提取 (Office Document & Report)\n\
- 可直接调用 `office` 工具的 `docx_read` 和 `docx_stats` 读取工作区中的 Word (.docx) 文档内容并分析结构。\n\
- 调用 `report_gen` 时必须提供真实的 `report` 对象；工具只呈现输入数据，不生成未经支持的指标或结论。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "提取 Word (.docx) 纯文本".to_string(),
                    command: "office-cli docx read --file meeting_notes.docx".to_string(),
                    description: "零第三方重型依赖，纯 Rust 流式解压提取 docx 内所有段落和表格。".to_string(),
                },
                CliExample {
                    title: "文档篇幅与阅读时长评估".to_string(),
                    command: "office-cli docx stats --file annual_report.docx".to_string(),
                    description: "精准计算字数、字符数、段落数并估算标准阅读时长。".to_string(),
                },
                CliExample {
                    title: "生成标准化商务分析报告".to_string(),
                    command: "office-cli report generate --file report.json --out report.md".to_string(),
                    description: "根据 report.json 中提供的摘要、KPI 与明细数据生成 Markdown 报告。".to_string(),
                },
            ],
        });

        // 3. Slides & Presentation Generator
        reg.register(Skill {
            id: "office-slides".to_string(),
            name: "演示文稿与幻灯片大纲 (Slides & Presentation)".to_string(),
            name_en: "Slides & Presentation Generator".to_string(),
            category: SkillCategory::Office,
            icon: "📽️".to_string(),
            description: "基于 Marp / Slidev 工业级标准，自动规划汇报框架并生成可直接放映的 Markdown 演示幻灯片。".to_string(),
            description_en: "Generates production-ready Marp/Slidev presentation decks with agendas, speaker notes, and layouts.".to_string(),
            system_prompt_snippet: "\
### Skill: 演示文稿生成 (Slides & Presentation Generator)\n\
- 调用 `office` 工具的 `slides_gen` 可依据汇报主题与页数生成结构严谨的 Marp 格式幻灯片源码。\n\
- 每页幻灯片自带演讲者提示词 (`<!-- note: ... -->`) 与分栏布局。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "生成技术规划演示文稿".to_string(),
                    command: "office-cli slides generate --title \"AI 办公智能体架构落地\" --slides 6".to_string(),
                    description: "生成包含封面、议程、方案剖析、量化成果及 Q&A 的完整 PPT 大纲。".to_string(),
                },
            ],
        });

        // 4. ASCII & Unicode Visual Charting
        reg.register(Skill {
            id: "office-chart".to_string(),
            name: "轻量可视化与文本图表 (Text Charting)".to_string(),
            name_en: "Text & ASCII Charting".to_string(),
            category: SkillCategory::Office,
            icon: "📈".to_string(),
            description: "无需浏览器或图形后端，直接在终端和对话中生成 Unicode 高清水平柱状图与趋势火花线 (Sparkline)。".to_string(),
            description_en: "Renders Unicode horizontal bar charts and sparklines directly in terminal and chat streams without graphical dependencies.".to_string(),
            system_prompt_snippet: "\
### Skill: 文本图表可视化 (Text Charting)\n\
- 调用 `office` 工具的 `chart_bar` 可根据标签和数值渲染整齐的 Unicode 柱状图，直观展示业务数据对比。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "绘制业务数据柱状图".to_string(),
                    command: "office-cli chart bar --labels \"Q1,Q2,Q3,Q4\" --values \"120,145,98,170\"".to_string(),
                    description: "输出等宽美观的文本柱状图与走势火花线，可直接粘贴进邮件或工单。".to_string(),
                },
            ],
        });

        // 5. Scientific Math & Formula Evaluator
        reg.register(Skill {
            id: "scientific-math".to_string(),
            name: "科学计算与公式求解 (Scientific Math)".to_string(),
            name_en: "Scientific Math & Expression Evaluator".to_string(),
            category: SkillCategory::Scientific,
            icon: "🔬".to_string(),
            description: "纯 Rust Shunting-yard 极速解析引擎，支持三角函数、双曲函数、对数指数、阶乘与数学常数求解。".to_string(),
            description_en: "Blazing fast pure Rust Shunting-yard engine supporting trigonometry, logarithms, factorials, and complex formulas.".to_string(),
            system_prompt_snippet: "\
### Skill: 科学计算 (Scientific Math)\n\
- 当需要进行高精度数学运算、复合函数求值或公式验证时，调用 `calc` 工具的 `eval` 动作。\n\
- 支持操作符：`+`, `-`, `*`, `/`, `%`, `^`, `!`，常数 `pi`, `e`, `tau`, `phi`，以及 `sin`, `cos`, `tan`, `sqrt`, `ln`, `log10`, `fact` 等函数。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "科学表达式求解".to_string(),
                    command: "calc-cli eval \"sin(pi/4) + sqrt(16) * 2^3 + fact(5)\"".to_string(),
                    description: "微秒级求得复杂复合函数的精确浮点值。".to_string(),
                },
                CliExample {
                    title: "对数与幂次综合运算".to_string(),
                    command: "calc-cli eval \"ln(100) / log10(100) + exp(2)\"".to_string(),
                    description: "支持自然对数、以10为底对数和欧拉数指数幂。".to_string(),
                },
            ],
        });

        // 6. Statistical Analysis & Modeling
        reg.register(Skill {
            id: "scientific-stats".to_string(),
            name: "统计分析与线性回归 (Statistics & Regression)".to_string(),
            name_en: "Statistical Analysis & Linear Regression".to_string(),
            category: SkillCategory::Scientific,
            icon: "📉".to_string(),
            description: "全功能描述性统计 (均值/中位数/众数/方差/标准差/分位数/IQR) 与一元线性回归、皮尔逊相关系数求解。".to_string(),
            description_en: "Descriptive statistics (mean, median, variance, std dev, percentiles) and linear regression with Pearson correlation.".to_string(),
            system_prompt_snippet: "\
### Skill: 统计建模与回归分析 (Statistics & Modeling)\n\
- 调用 `calc` 的 `stat` 动作可对一维样本进行全面的描述性统计。\n\
- 调用 `calc` 的 `regression` 动作可输入 X、Y 数组拟合一元线性方程 `y = mx + b`，并输出相关系数 r 及解释方差 R²。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "样本描述性统计分析".to_string(),
                    command: "calc-cli stat \"12.5, 14.8, 19.2, 23.1, 28.4, 31.0, 35.6\"".to_string(),
                    description: "输出总和、均值、中位数、四分位距 IQR 以及 P90/P95/P99 高位分位数。".to_string(),
                },
                CliExample {
                    title: "双变量线性回归与相关性判定".to_string(),
                    command: "calc-cli regress \"1,2,3,4,5\" \"2.1,3.9,6.2,7.8,10.1\"".to_string(),
                    description: "计算拟合斜率、截距、皮尔逊相关系数 r 和判定系数 R²。".to_string(),
                },
            ],
        });

        // 7. Linear Algebra & Matrix Math
        reg.register(Skill {
            id: "scientific-linear-algebra".to_string(),
            name: "线性代数与矩阵运算 (Linear Algebra)".to_string(),
            name_en: "Linear Algebra & Matrix Operations".to_string(),
            category: SkillCategory::Scientific,
            icon: "📐".to_string(),
            description: "向量点乘、叉乘、模长、余弦相似度计算，以及矩阵乘法、转置与高斯消元行列式 (Determinant) 求解。".to_string(),
            description_en: "Vector dot product, cross product, cosine similarity, and matrix multiplication, transpose, and Gaussian determinant.".to_string(),
            system_prompt_snippet: "\
### Skill: 线性代数与向量计算 (Linear Algebra)\n\
- 调用 `calc` 工具的 `matrix` 和 `vector` 动作进行空间几何、向量相似度度量与矩阵变换。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "矩阵乘法计算".to_string(),
                    command: "calc-cli matrix mul \"[[1,2],[3,4]]\" \"[[2,0],[1,2]]\"".to_string(),
                    description: "执行标准行向量与列向量相乘，输出结果矩阵。".to_string(),
                },
                CliExample {
                    title: "向量余弦相似度".to_string(),
                    command: "calc-cli vector cosine \"[1,2,3]\" \"[2,4,6]\"".to_string(),
                    description: "常用于高维特征比对和语义嵌入向量夹角计算。".to_string(),
                },
            ],
        });

        // 8. Financial & Office Mathematics
        reg.register(Skill {
            id: "financial-math".to_string(),
            name: "办公财务与信贷投资计算 (Financial Math)".to_string(),
            name_en: "Financial & Investment Math".to_string(),
            category: SkillCategory::Financial,
            icon: "💰".to_string(),
            description: "等额本息房贷与车贷月供 (PMT)、复利终值、复合年均增长率 (CAGR)、投资回报率 (ROI) 与净现值 (NPV)。".to_string(),
            description_en: "Mortgage/loan monthly payment (PMT), compound interest, CAGR, ROI, and Net Present Value (NPV).".to_string(),
            system_prompt_snippet: "\
### Skill: 办公财务与投资计算 (Financial Math)\n\
- 处理贷款测算、利息评估或商业计划书回报率时，调用 `calc` 的 `finance` 动作。\n\
- 涵盖：`pmt` (计算月供与总利息支出)、`compound` (复利收益测算)、`cagr` (增长率)、`roi` (投资收益比)。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "房贷/商业贷款月供测算".to_string(),
                    command: "calc-cli finance pmt --principal 1000000 --rate 0.0385 --periods 360".to_string(),
                    description: "计算百万贷款在 30 年期等额本息下的每月还款额与总利息。".to_string(),
                },
                CliExample {
                    title: "复利终值增长测算".to_string(),
                    command: "calc-cli finance compound --principal 50000 --rate 0.06 --periods 10".to_string(),
                    description: "按月复利计算 10 年后累计资产总值与利息收益。".to_string(),
                },
            ],
        });

        // 9. Metric, Imperial & Digital Unit Conversion
        reg.register(Skill {
            id: "unit-conversion".to_string(),
            name: "度量衡与数据存储换算 (Unit Conversion)".to_string(),
            name_en: "Physical & Digital Unit Conversion".to_string(),
            category: SkillCategory::General,
            icon: "🔄".to_string(),
            description: "长度、质量、温度、时间以及计算机存储字节 (B, KB, MB, GB, TB / KiB, MiB, GiB) 双向精确换算。".to_string(),
            description_en: "Precision metric, imperial, temperature, time, and digital storage (bytes, GiB, TiB) unit conversions.".to_string(),
            system_prompt_snippet: "\
### Skill: 度量衡换算 (Unit Conversion)\n\
- 调用 `calc` 的 `unit` 动作可实现国际公制、英制、温度与计算机二进制存储单位之间的快速换算。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "数字存储单位换算".to_string(),
                    command: "calc-cli convert 100 \"gib\" \"mb\"".to_string(),
                    description: "准确处理 1024 与 1000 进制的存储容量换算。".to_string(),
                },
                CliExample {
                    title: "温度摄氏与华氏互转".to_string(),
                    command: "calc-cli convert 37.5 \"celsius\" \"fahrenheit\"".to_string(),
                    description: "工程与环境温度标准换算。".to_string(),
                },
            ],
        });

        // 10. Date & Business Days Calculation
        reg.register(Skill {
            id: "date-calc".to_string(),
            name: "公历日历与工作日计算 (Date & Business Days)".to_string(),
            name_en: "Calendar & Business Days Calculation".to_string(),
            category: SkillCategory::General,
            icon: "📅".to_string(),
            description: "精确公历两日期间隔计算、闰年识别、自动剔除周末的工作日 (Business Days) 统计。".to_string(),
            description_en: "Exact calendar days delta, leap year verification, and business days (excluding weekends) calculation.".to_string(),
            system_prompt_snippet: "\
### Skill: 日历与工作日计算 (Date & Business Days)\n\
- 调用 `calc` 的 `date` 动作计算两日期间的自然天数与净工作日数量，便于项目排期和工期测算。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "项目排期净工作日计算".to_string(),
                    command: "calc-cli date diff \"2026-10-20\" \"2026-12-31\"".to_string(),
                    description: "自动扣除周六日，精确计算剩余交付工作日。".to_string(),
                },
            ],
        });

        // 11. Generative UI & Visual Artifacts
        reg.register(Skill {
            id: "generative-ui".to_string(),
            name: "对话生成界面与交互产物 (Generative UI)".to_string(),
            name_en: "Generative UI & Visual Artifacts".to_string(),
            category: SkillCategory::General,
            icon: "🎨".to_string(),
            description: "原生 GPU 硬件加速交互界面与图表引擎，支持在会话中渲染交互卡片、表单收集、统计图表与独立 Web 视窗产物。".to_string(),
            description_en: "GPU-accelerated native Generative UI and charting engine for conversational cards, form controls, charts, and standalone web artifacts.".to_string(),
            system_prompt_snippet: "\
### Skill: 对话生成 UI 与交互产物 (Generative UI & Visual Artifacts)\n\
- 当需要向用户展示决策审批、选项单选/多选、表单输入、关键 KPI 指标、数据走势或数据分布时，可输出 ```json-ui 代码块。\n\
- 支持的核心 UI 节点包括：\n\
  1. 卡片与布局: `card` (带标题/副标题/徽章), `container` (横向/纵向弹性布局), `callout` (信息/成功/警告提示), `accordion` (折叠区)\n\
  2. 表单与控制: `choice_group` (药丸选项组), `select` (单选下拉框), `input_field` (输入框提交), `toggle` (状态开关), `checkbox_group` (多选项), `form` (多字段聚合表单), `rating` (星级评分)\n\
  3. 数据可视化: `metric` (大字指标), `line_chart` (平滑面积趋势图), `bar_chart` (柱状图), `pie_chart` (扇形/环形图), `sparkline` (迷你走势), `table` (表格)\n\
  4. 产物与前端: `artifact` (代码/HTML/SVG) 以及 `<agent-embed src=\"...\">` (内嵌前端小组件，支持侧栏与独立 Web 视窗预览)。\n\
- 结构化 JSON 会被客户端直接实时渲染为原生 GPU 硬件加速组件，用户操作按钮会双向回传给智能体推进任务。".to_string(),
            cli_examples: vec![
                CliExample {
                    title: "生成审批确认卡片".to_string(),
                    command: "volt-cli ui card --title \"生产部署审批\" --badge \"待确认\"".to_string(),
                    description: "生成包含目标集群、待发版本、确认/取消操作按钮的原生交互卡片。".to_string(),
                },
                CliExample {
                    title: "渲染 QPS 趋势图".to_string(),
                    command: "volt-cli ui chart --type line --points \"120,150,220,310,290,450\"".to_string(),
                    description: "基于 Metal GPU 硬件加速绘制平滑面积趋势曲线与关键极值统计。".to_string(),
                },
                CliExample {
                    title: "渲染业务份额环形图".to_string(),
                    command: "volt-cli ui chart --type pie --donut --slices \"API:45,Web:30,Job:25\"".to_string(),
                    description: "原生绘制带图例与百分比标签的高清环形分布图。".to_string(),
                },
            ],
        });

        reg
    }

    pub fn register(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    pub fn all(&self) -> &[Skill] {
        &self.skills
    }

    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    pub fn build_system_prompt_appendix(&self) -> String {
        let mut out = String::from("\n## Built-in Native Skills & Capabilities:\n");
        for s in &self.skills {
            out.push_str(&format!("\n{}\n", s.system_prompt_snippet));
        }
        out
    }
}
