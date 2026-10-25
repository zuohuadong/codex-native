use anyhow::{bail, Result};
use pi_agent_core::tools::calc::parse_numbers;
use pi_agent_core::tools::calc::{
    DateCalculator, FinanceEngine, LinearAlgebra, MathEvaluator, StatEngine, UnitConverter,
};

fn print_help() {
    println!(
        r#"🔬 calc-cli: Native Rust High-Performance Scientific & Math Calculator
======================================================================
Usage: calc-cli <COMMAND> [OPTIONS]

COMMANDS:
  eval     "<EXPRESSION>"
           Evaluate mathematical and scientific expressions
           Examples:
             calc-cli eval "sin(pi/4) + sqrt(16) * 2^3"
             calc-cli eval "ln(100) + fact(5) - 3.5e-2"

  stat     "<N1,N2,N3,...>"
           Compute comprehensive descriptive statistics (mean, median, std_dev, IQR, percentiles)
           Example:
             calc-cli stat "12.5, 14.8, 19.2, 23.1, 28.4, 31.0, 35.6"

  regress  "<X1,X2,...>" "<Y1,Y2,...>"
           Linear regression line (y = mx + b), Pearson correlation r, and R²
           Example:
             calc-cli regress "1,2,3,4,5" "2.1,3.9,6.2,7.8,10.1"

  matrix   mul "<MAT_A>" "<MAT_B>" | det "<MAT>" | transpose "<MAT>"
           Matrix multiplication, determinant, or transposition
           Example:
             calc-cli matrix mul "[[1,2],[3,4]]" "[[2,0],[1,2]]"

  vector   dot "<V1>" "<V2>" | cosine "<V1>" "<V2>" | norm "<V>"
           Vector dot product, cosine similarity, or Euclidean norm
           Example:
             calc-cli vector cosine "[1,2,3]" "[2,4,6]"

  finance  pmt --principal <P> --rate <R> --periods <N>
           finance compound --principal <P> --rate <R> --periods <N>
           finance cagr --principal <P> --value <V> --periods <N>
           Loan monthly payment (PMT), compound interest, or CAGR

  convert  <VAL> <FROM_UNIT> <TO_UNIT>
           Physical, temperature, and digital storage unit conversions
           Example:
             calc-cli convert 100 "gib" "mb"
             calc-cli convert 37.5 "celsius" "fahrenheit"

  date     diff <YYYY-MM-DD> <YYYY-MM-DD>
           Calculate calendar days and business days (excluding weekends)
           Example:
             calc-cli date diff "2026-10-20" "2026-12-31"

FLAGS:
  --help, -h    Show this help message
"#
    );
}

fn number_option(args: &[String], flag: &str, default: f64) -> Result<f64> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(default);
    };
    let raw = args
        .get(index + 1)
        .ok_or_else(|| anyhow::anyhow!("Missing value for {flag}"))?;
    let number: f64 = raw
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number for {flag}: {raw}"))?;
    if !number.is_finite() {
        bail!("{flag} must be finite");
    }
    Ok(number)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return Ok(());
    }

    let cmd1 = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd1 {
        "eval" => {
            let expr = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Missing expression to evaluate"))?;
            let val = MathEvaluator::eval(expr)?;
            println!("Expression: {}", expr);
            println!("Result:     {}", val);
            println!("Precise:    {:.10}", val);
        }
        "stat" => {
            let nums_str = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Missing numbers string"))?;
            let data: Vec<f64> = parse_numbers(nums_str)?;
            let summary = StatEngine::compute(&data)?;
            println!("📊 Descriptive Statistics (N = {}):", summary.count);
            println!("  • Sum:       {:.4}", summary.sum);
            println!("  • Mean:      {:.4}", summary.mean);
            println!("  • Median:    {:.4}", summary.median);
            if let Some(m) = summary.mode {
                println!("  • Mode:      {:.4}", m);
            }
            println!(
                "  • Std Dev:   {:.4} (Variance: {:.4})",
                summary.std_dev, summary.variance
            );
            println!(
                "  • Min / Max: {:.4} / {:.4} (Range: {:.4})",
                summary.min, summary.max, summary.range
            );
            println!(
                "  • IQR:       {:.4} [Q1: {:.4}, Q3: {:.4}]",
                summary.iqr, summary.q1, summary.q3
            );
            println!(
                "  • P90 / P95: {:.4} / {:.4} (P99: {:.4})",
                summary.p90, summary.p95, summary.p99
            );
        }
        "regress" => {
            let x_str = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Missing X vector"))?;
            let y_str = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing Y vector"))?;
            let x: Vec<f64> = parse_numbers(x_str)?;
            let y: Vec<f64> = parse_numbers(y_str)?;
            let (slope, intercept, r, r_sq) = StatEngine::linear_regression(&x, &y)?;
            let sign = if intercept >= 0.0 { "+" } else { "-" };
            println!("📈 Linear Regression Analysis (N = {}):", x.len());
            println!(
                "  • Model Equation:      y = {:.6} * x {} {:.6}",
                slope,
                sign,
                intercept.abs()
            );
            println!("  • Slope (m):           {:.6}", slope);
            println!("  • Intercept (b):       {:.6}", intercept);
            println!("  • Pearson r:           {:.6}", r);
            println!(
                "  • R² Determination:    {:.6} ({:.2}% variance explained)",
                r_sq,
                r_sq * 100.0
            );
        }
        "matrix" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("mul");
            match sub {
                "mul" => {
                    let a_str = args
                        .get(2)
                        .ok_or_else(|| anyhow::anyhow!("Missing Matrix A"))?;
                    let b_str = args
                        .get(3)
                        .ok_or_else(|| anyhow::anyhow!("Missing Matrix B"))?;
                    let a: Vec<Vec<f64>> = serde_json::from_str(a_str)?;
                    let b: Vec<Vec<f64>> = serde_json::from_str(b_str)?;
                    let res = LinearAlgebra::matrix_multiply(&a, &b)?;
                    println!("Result Matrix ({}x{}):", res.len(), res[0].len());
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
                "det" => {
                    let a_str = args
                        .get(2)
                        .ok_or_else(|| anyhow::anyhow!("Missing Matrix"))?;
                    let a: Vec<Vec<f64>> = serde_json::from_str(a_str)?;
                    let det = LinearAlgebra::matrix_determinant(&a)?;
                    println!("Matrix Determinant: {:.8}", det);
                }
                "transpose" => {
                    let a_str = args
                        .get(2)
                        .ok_or_else(|| anyhow::anyhow!("Missing Matrix"))?;
                    let a: Vec<Vec<f64>> = serde_json::from_str(a_str)?;
                    let res = LinearAlgebra::matrix_transpose(&a)?;
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
                _ => bail!("Unknown matrix command: {}", sub),
            }
        }
        "vector" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("dot");
            let v1_str = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing Vector 1"))?;
            let v1: Vec<f64> = serde_json::from_str(v1_str)?;
            match sub {
                "dot" => {
                    let v2_str = args
                        .get(3)
                        .ok_or_else(|| anyhow::anyhow!("Missing Vector 2"))?;
                    let v2: Vec<f64> = serde_json::from_str(v2_str)?;
                    let dot = LinearAlgebra::vector_dot(&v1, &v2)?;
                    println!("Vector Dot Product: {:.8}", dot);
                }
                "cosine" => {
                    let v2_str = args
                        .get(3)
                        .ok_or_else(|| anyhow::anyhow!("Missing Vector 2"))?;
                    let v2: Vec<f64> = serde_json::from_str(v2_str)?;
                    let sim = LinearAlgebra::vector_cosine_similarity(&v1, &v2)?;
                    println!(
                        "Cosine Similarity: {:.8} (Angle: {:.2}°)",
                        sim,
                        sim.acos().to_degrees()
                    );
                }
                "norm" => {
                    let norm = LinearAlgebra::vector_norm(&v1);
                    println!("Euclidean Norm (L2): {:.8}", norm);
                }
                _ => bail!("Unknown vector command: {}", sub),
            }
        }
        "finance" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("pmt");
            match sub {
                "pmt" => {
                    let pv = number_option(&args, "--principal", 1_000_000.0)?;
                    let rate = number_option(&args, "--rate", 0.0385)?;
                    let periods = number_option(&args, "--periods", 360.0)?;
                    let (monthly, total, interest) = FinanceEngine::pmt(rate / 12.0, periods, pv)?;
                    println!("💰 Loan PMT Analysis:");
                    println!("  • Principal:       {:.2}", pv);
                    println!("  • Annual Rate:     {:.2}%", rate * 100.0);
                    println!(
                        "  • Periods:         {} months ({:.1} years)",
                        periods,
                        periods / 12.0
                    );
                    println!("  • Monthly Payment: {:.2}", monthly);
                    println!("  • Total Payment:   {:.2}", total);
                    println!("  • Total Interest:  {:.2}", interest);
                }
                "compound" => {
                    let p = number_option(&args, "--principal", 10_000.0)?;
                    let r = number_option(&args, "--rate", 0.06)?;
                    let periods = number_option(&args, "--periods", 10.0)?;
                    let (amount, interest) = FinanceEngine::compound_interest(p, r, 12.0, periods)?;
                    println!("📈 Compound Interest:");
                    println!("  • Future Value:    {:.2}", amount);
                    println!("  • Total Interest:  {:.2}", interest);
                    println!("  • Growth Factor:   {:.2}x", amount / p);
                }
                "cagr" => {
                    let begin = number_option(&args, "--principal", 100.0)?;
                    let end = number_option(&args, "--value", 250.0)?;
                    let years = number_option(&args, "--periods", 5.0)?;
                    let cagr = FinanceEngine::cagr(begin, end, years)?;
                    println!("📊 CAGR: {:.4} ({:.2}%)", cagr, cagr * 100.0);
                }
                _ => bail!("Unknown finance command: {}", sub),
            }
        }
        "convert" => {
            let val: f64 = args
                .get(1)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow::anyhow!("Missing numeric value"))?;
            let from = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing from unit"))?;
            let to = args
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Missing to unit"))?;
            let converted = UnitConverter::convert(val, from, to)?;
            println!("🔄 Conversion: {} {} = {} {}", val, from, converted, to);
        }
        "date" => {
            let d1 = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing start date (YYYY-MM-DD)"))?;
            let d2 = args
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Missing end date (YYYY-MM-DD)"))?;
            let days = DateCalculator::days_between(d1, d2)?;
            let b_days = DateCalculator::business_days_between(d1, d2)?;
            println!("📅 Date Calculation ({} -> {}):", d1, d2);
            println!("  • Calendar Days:  {} days", days);
            println!("  • Business Days:  {} days (excluding weekends)", b_days);
            println!("  • Full Weeks:     {:.1} weeks", (days as f64) / 7.0);
        }
        _ => {
            bail!("Unknown command: '{}'. Run with --help for usage.", cmd1);
        }
    }

    Ok(())
}
