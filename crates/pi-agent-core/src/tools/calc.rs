use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use super::Tool;

pub fn parse_numbers(input: &str) -> Result<Vec<f64>> {
    input
        .split(',')
        .enumerate()
        .map(|(index, field)| {
            let number: f64 = field
                .trim()
                .parse()
                .map_err(|_| anyhow!("Invalid number at position {}: '{}'", index + 1, field))?;
            if !number.is_finite() {
                bail!("Number at position {} must be finite", index + 1);
            }
            Ok(number)
        })
        .collect()
}

// ============================================================================
// 1. SCIENTIFIC MATH EXPRESSION EVALUATOR
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    UnaryMinus,
    Multiply,
    Divide,
    Modulo,
    Power,
    Factorial,
    LParen,
    RParen,
    Comma,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut prev_is_op_or_lparen = true;

        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }

            if ch.is_ascii_digit() || ch == '.' {
                let num = self.read_number()?;
                tokens.push(Token::Number(num));
                prev_is_op_or_lparen = false;
                continue;
            }

            if ch.is_alphabetic() || ch == '_' {
                let ident = self.read_ident();
                tokens.push(Token::Ident(ident.to_lowercase()));
                prev_is_op_or_lparen = false;
                continue;
            }

            self.chars.next();
            match ch {
                '+' => {
                    if prev_is_op_or_lparen {
                        // Unary plus, skip
                        continue;
                    }
                    tokens.push(Token::Plus);
                    prev_is_op_or_lparen = true;
                }
                '-' => {
                    if prev_is_op_or_lparen {
                        tokens.push(Token::UnaryMinus);
                    } else {
                        tokens.push(Token::Minus);
                    }
                    prev_is_op_or_lparen = true;
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    prev_is_op_or_lparen = true;
                }
                '/' => {
                    tokens.push(Token::Divide);
                    prev_is_op_or_lparen = true;
                }
                '%' => {
                    tokens.push(Token::Modulo);
                    prev_is_op_or_lparen = true;
                }
                '^' => {
                    tokens.push(Token::Power);
                    prev_is_op_or_lparen = true;
                }
                '!' => {
                    tokens.push(Token::Factorial);
                    prev_is_op_or_lparen = false;
                }
                '(' => {
                    tokens.push(Token::LParen);
                    prev_is_op_or_lparen = true;
                }
                ')' => {
                    tokens.push(Token::RParen);
                    prev_is_op_or_lparen = false;
                }
                ',' => {
                    tokens.push(Token::Comma);
                    prev_is_op_or_lparen = true;
                }
                _ => bail!("Unexpected character in expression: '{}'", ch),
            }
        }

        Ok(tokens)
    }

    fn read_number(&mut self) -> Result<f64> {
        let mut s = String::new();
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.chars.next();
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                s.push(c);
                self.chars.next();
            } else if (c == 'e' || c == 'E') && !has_exp {
                has_exp = true;
                s.push(c);
                self.chars.next();
                if let Some(&sign) = self.chars.peek() {
                    if sign == '+' || sign == '-' {
                        s.push(sign);
                        self.chars.next();
                    }
                }
            } else {
                break;
            }
        }

        s.parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse float '{}': {}", s, e))
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        s
    }
}

pub struct MathEvaluator;

impl MathEvaluator {
    pub fn eval(expr: &str) -> Result<f64> {
        let mut lexer = Lexer::new(expr);
        let tokens = lexer.tokenize()?;
        let rpn = Self::shunting_yard(tokens)?;
        Self::eval_rpn(rpn)
    }

    fn get_precedence(op: &Token) -> (u8, bool) {
        // Returns (precedence, is_right_associative)
        match op {
            Token::Factorial => (5, false),
            Token::UnaryMinus => (4, true),
            Token::Power => (3, true),
            Token::Multiply | Token::Divide | Token::Modulo => (2, false),
            Token::Plus | Token::Minus => (1, false),
            _ => (0, false),
        }
    }

    fn shunting_yard(tokens: Vec<Token>) -> Result<Vec<Token>> {
        let mut output = Vec::new();
        let mut op_stack = Vec::new();

        for token in tokens {
            match token {
                Token::Number(_) => output.push(token),
                Token::Ident(ref name) => {
                    // Check if it's a known constant
                    match name.as_str() {
                        "pi" | "π" => output.push(Token::Number(std::f64::consts::PI)),
                        "e" => output.push(Token::Number(std::f64::consts::E)),
                        "tau" | "τ" => output.push(Token::Number(std::f64::consts::TAU)),
                        "phi" => output.push(Token::Number(1.618033988749895)),
                        _ => op_stack.push(token), // function call
                    }
                }
                Token::Comma => {
                    while let Some(top) = op_stack.last() {
                        if *top == Token::LParen {
                            break;
                        }
                        output.push(op_stack.pop().unwrap());
                    }
                    if op_stack.is_empty() {
                        bail!("Misplaced comma or mismatched parentheses");
                    }
                }
                Token::Plus
                | Token::Minus
                | Token::UnaryMinus
                | Token::Multiply
                | Token::Divide
                | Token::Modulo
                | Token::Power
                | Token::Factorial => {
                    let (prec, right_assoc) = Self::get_precedence(&token);
                    while let Some(top) = op_stack.last() {
                        if *top == Token::LParen {
                            break;
                        }
                        let (top_prec, _) = Self::get_precedence(top);
                        if (top_prec > prec) || (top_prec == prec && !right_assoc) {
                            output.push(op_stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    op_stack.push(token);
                }
                Token::LParen => op_stack.push(token),
                Token::RParen => {
                    let mut found_lparen = false;
                    while let Some(top) = op_stack.pop() {
                        if top == Token::LParen {
                            found_lparen = true;
                            break;
                        }
                        output.push(top);
                    }
                    if !found_lparen {
                        bail!("Mismatched parentheses: missing '('");
                    }
                    if let Some(Token::Ident(_)) = op_stack.last() {
                        output.push(op_stack.pop().unwrap());
                    }
                }
            }
        }

        while let Some(op) = op_stack.pop() {
            if op == Token::LParen || op == Token::RParen {
                bail!("Mismatched parentheses in expression");
            }
            output.push(op);
        }

        Ok(output)
    }

    fn eval_rpn(tokens: Vec<Token>) -> Result<f64> {
        let mut stack: Vec<f64> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(n) => stack.push(n),
                Token::Plus => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '+'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '+'"))?;
                    stack.push(a + b);
                }
                Token::Minus => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '-'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '-'"))?;
                    stack.push(a - b);
                }
                Token::UnaryMinus => {
                    let a = stack
                        .pop()
                        .ok_or_else(|| anyhow!("Syntax error in unary '-'"))?;
                    stack.push(-a);
                }
                Token::Multiply => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '*'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '*'"))?;
                    stack.push(a * b);
                }
                Token::Divide => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '/'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '/'"))?;
                    if b == 0.0 {
                        bail!("Division by zero");
                    }
                    stack.push(a / b);
                }
                Token::Modulo => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '%'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '%'"))?;
                    stack.push(a % b);
                }
                Token::Power => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Syntax error in '^'"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '^'"))?;
                    stack.push(a.powf(b));
                }
                Token::Factorial => {
                    let a = stack.pop().ok_or_else(|| anyhow!("Syntax error in '!'"))?;
                    if a < 0.0 || a.fract() != 0.0 || a > 170.0 {
                        bail!("Factorial operand must be an integer between 0 and 170");
                    }
                    let mut res = 1.0;
                    for i in 1..=(a as u64) {
                        res *= i as f64;
                    }
                    stack.push(res);
                }
                Token::Ident(name) => match name.as_str() {
                    "sin" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for sin()"))?;
                        stack.push(a.sin());
                    }
                    "cos" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for cos()"))?;
                        stack.push(a.cos());
                    }
                    "tan" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for tan()"))?;
                        stack.push(a.tan());
                    }
                    "asin" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for asin()"))?;
                        stack.push(a.asin());
                    }
                    "acos" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for acos()"))?;
                        stack.push(a.acos());
                    }
                    "atan" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for atan()"))?;
                        stack.push(a.atan());
                    }
                    "atan2" => {
                        let x = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing second argument for atan2(y, x)"))?;
                        let y = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing first argument for atan2(y, x)"))?;
                        stack.push(y.atan2(x));
                    }
                    "sinh" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for sinh()"))?;
                        stack.push(a.sinh());
                    }
                    "cosh" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for cosh()"))?;
                        stack.push(a.cosh());
                    }
                    "tanh" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for tanh()"))?;
                        stack.push(a.tanh());
                    }
                    "sqrt" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for sqrt()"))?;
                        if a < 0.0 {
                            bail!("Square root of negative number: {}", a);
                        }
                        stack.push(a.sqrt());
                    }
                    "cbrt" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for cbrt()"))?;
                        stack.push(a.cbrt());
                    }
                    "exp" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for exp()"))?;
                        stack.push(a.exp());
                    }
                    "ln" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for ln()"))?;
                        if a <= 0.0 {
                            bail!("Natural log of non-positive number: {}", a);
                        }
                        stack.push(a.ln());
                    }
                    "log10" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for log10()"))?;
                        if a <= 0.0 {
                            bail!("Log10 of non-positive number: {}", a);
                        }
                        stack.push(a.log10());
                    }
                    "log2" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for log2()"))?;
                        if a <= 0.0 {
                            bail!("Log2 of non-positive number: {}", a);
                        }
                        stack.push(a.log2());
                    }
                    "abs" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for abs()"))?;
                        stack.push(a.abs());
                    }
                    "floor" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for floor()"))?;
                        stack.push(a.floor());
                    }
                    "ceil" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for ceil()"))?;
                        stack.push(a.ceil());
                    }
                    "round" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for round()"))?;
                        stack.push(a.round());
                    }
                    "deg_to_rad" | "rad" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for deg_to_rad()"))?;
                        stack.push(a.to_radians());
                    }
                    "rad_to_deg" | "deg" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for rad_to_deg()"))?;
                        stack.push(a.to_degrees());
                    }
                    "min" => {
                        let b = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing second argument for min()"))?;
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing first argument for min()"))?;
                        stack.push(a.min(b));
                    }
                    "max" => {
                        let b = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing second argument for max()"))?;
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing first argument for max()"))?;
                        stack.push(a.max(b));
                    }
                    "pow" => {
                        let b = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing exponent for pow()"))?;
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing base for pow()"))?;
                        stack.push(a.powf(b));
                    }
                    "fact" => {
                        let a = stack
                            .pop()
                            .ok_or_else(|| anyhow!("Missing argument for fact()"))?;
                        if a < 0.0 || a.fract() != 0.0 || a > 170.0 {
                            bail!("Factorial operand must be an integer between 0 and 170");
                        }
                        let mut res = 1.0;
                        for i in 1..=(a as u64) {
                            res *= i as f64;
                        }
                        stack.push(res);
                    }
                    unknown => bail!("Unknown function in expression: '{}()'", unknown),
                },
                _ => bail!("Unexpected token in evaluation"),
            }
        }

        if stack.len() != 1 {
            bail!("Invalid expression evaluation state: leftover items on stack");
        }

        Ok(stack[0])
    }
}

// ============================================================================
// 2. STATISTICAL & DATA COMPUTING ENGINE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatSummary {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub mode: Option<f64>,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

pub struct StatEngine;

impl StatEngine {
    pub fn compute(data: &[f64]) -> Result<StatSummary> {
        if data.is_empty() {
            bail!("Data set cannot be empty for statistical analysis");
        }

        let count = data.len();
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = sorted.iter().sum();
        let mean = sum / (count as f64);

        let median = Self::percentile(&sorted, 50.0);
        let q1 = Self::percentile(&sorted, 25.0);
        let q3 = Self::percentile(&sorted, 75.0);
        let iqr = q3 - q1;
        let p90 = Self::percentile(&sorted, 90.0);
        let p95 = Self::percentile(&sorted, 95.0);
        let p99 = Self::percentile(&sorted, 99.0);

        let variance = if count > 1 {
            let sq_diff_sum: f64 = sorted.iter().map(|&x| (x - mean).powi(2)).sum();
            sq_diff_sum / ((count - 1) as f64)
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let min = sorted[0];
        let max = sorted[count - 1];
        let range = max - min;

        // Mode calculation
        let mut freq_map: HashMap<i64, usize> = HashMap::new();
        for &x in &sorted {
            let key = (x * 1000.0).round() as i64;
            *freq_map.entry(key).or_insert(0) += 1;
        }
        let mode = freq_map
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .filter(|&(_, count)| count > 1)
            .map(|(key, _)| (key as f64) / 1000.0);

        Ok(StatSummary {
            count,
            sum,
            mean,
            median,
            mode,
            variance,
            std_dev,
            min,
            max,
            range,
            q1,
            q3,
            iqr,
            p90,
            p95,
            p99,
        })
    }

    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.len() == 1 {
            return sorted[0];
        }
        let index = (p / 100.0) * ((sorted.len() - 1) as f64);
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;
        let weight = index - (lower as f64);
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }

    pub fn linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64, f64, f64)> {
        // Returns (slope m, intercept b, r correlation, r_squared)
        if x.len() != y.len() || x.len() < 2 {
            bail!("Linear regression requires at least 2 pairs of points with matching lengths");
        }
        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&a, &b)| a * b).sum();
        let sum_xx: f64 = x.iter().map(|&a| a * a).sum();
        let sum_yy: f64 = y.iter().map(|&b| b * b).sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() < 1e-12 {
            bail!("Variance of X is zero; vertical line regression undefined");
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        let num_r = n * sum_xy - sum_x * sum_y;
        let den_r = ((n * sum_xx - sum_x * sum_x) * (n * sum_yy - sum_y * sum_y)).sqrt();
        let r = if den_r > 0.0 { num_r / den_r } else { 0.0 };
        let r_squared = r * r;

        Ok((slope, intercept, r, r_squared))
    }
}

// ============================================================================
// 3. LINEAR ALGEBRA & MATRIX OPERATIONS
// ============================================================================

pub struct LinearAlgebra;

impl LinearAlgebra {
    pub fn vector_dot(a: &[f64], b: &[f64]) -> Result<f64> {
        if a.len() != b.len() {
            bail!("Vector dimensions must match ({} vs {})", a.len(), b.len());
        }
        Ok(a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum())
    }

    pub fn vector_norm(a: &[f64]) -> f64 {
        a.iter().map(|&x| x * x).sum::<f64>().sqrt()
    }

    pub fn vector_cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64> {
        let dot = Self::vector_dot(a, b)?;
        let norm_a = Self::vector_norm(a);
        let norm_b = Self::vector_norm(b);
        if norm_a == 0.0 || norm_b == 0.0 {
            bail!("Cannot compute cosine similarity with a zero vector");
        }
        Ok(dot / (norm_a * norm_b))
    }

    pub fn vector_cross_3d(a: &[f64], b: &[f64]) -> Result<Vec<f64>> {
        if a.len() != 3 || b.len() != 3 {
            bail!("3D Cross product requires exactly 3 dimensions");
        }
        Ok(vec![
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ])
    }

    fn validate_matrix(m: &[Vec<f64>], name: &str) -> Result<(usize, usize)> {
        if m.is_empty() {
            bail!("Matrix '{}' cannot be empty", name);
        }
        let cols = m[0].len();
        if cols == 0 {
            bail!("Matrix '{}' cannot have zero columns", name);
        }
        for (idx, row) in m.iter().enumerate() {
            if row.len() != cols {
                bail!(
                    "Ragged matrix in '{}': row {} has length {}, expected {}",
                    name,
                    idx,
                    row.len(),
                    cols
                );
            }
        }
        Ok((m.len(), cols))
    }

    pub fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        let (rows_a, cols_a) = Self::validate_matrix(a, "matrix_a")?;
        let (rows_b, cols_b) = Self::validate_matrix(b, "matrix_b")?;

        if cols_a != rows_b {
            bail!(
                "Matrix dimension mismatch for multiplication: ({}x{}) x ({}x{})",
                rows_a,
                cols_a,
                rows_b,
                cols_b
            );
        }

        let mut res = vec![vec![0.0; cols_b]; rows_a];
        for i in 0..rows_a {
            for k in 0..cols_a {
                for j in 0..cols_b {
                    res[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        Ok(res)
    }

    pub fn matrix_transpose(m: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        if m.is_empty() {
            return Ok(Vec::new());
        }
        let (rows, cols) = Self::validate_matrix(m, "matrix")?;
        let mut res = vec![vec![0.0; rows]; cols];
        for i in 0..rows {
            for j in 0..cols {
                res[j][i] = m[i][j];
            }
        }
        Ok(res)
    }

    pub fn matrix_determinant(m: &[Vec<f64>]) -> Result<f64> {
        let (n, cols) = Self::validate_matrix(m, "matrix")?;
        if cols != n {
            bail!(
                "Matrix must be square to compute determinant: ({}x{})",
                n,
                cols
            );
        }
        let mut mat: Vec<Vec<f64>> = m.to_vec();
        let mut det = 1.0;

        for i in 0..n {
            // Find pivot
            let mut pivot = i;
            for j in (i + 1)..n {
                if mat[j][i].abs() > mat[pivot][i].abs() {
                    pivot = j;
                }
            }
            if mat[pivot][i].abs() < 1e-12 {
                return Ok(0.0);
            }
            if pivot != i {
                mat.swap(i, pivot);
                det = -det;
            }
            det *= mat[i][i];

            let diag = mat[i][i];
            for j in (i + 1)..n {
                let factor = mat[j][i] / diag;
                let (above, below) = mat.split_at_mut(j);
                for (cell, pivot_cell) in below[0][i..].iter_mut().zip(&above[i][i..]) {
                    *cell -= factor * pivot_cell;
                }
            }
        }

        Ok(det)
    }
}

// ============================================================================
// 4. FINANCIAL & BUSINESS MATHEMATICS
// ============================================================================

pub struct FinanceEngine;

impl FinanceEngine {
    /// Loan payment (PMT) calculation
    /// rate: periodic rate (e.g. annual_rate / 12)
    /// nper: number of periods (e.g. years * 12)
    /// pv: present value (loan amount)
    pub fn pmt(rate: f64, nper: f64, pv: f64) -> Result<(f64, f64, f64)> {
        // Returns (monthly_pmt, total_payment, total_interest)
        if rate == 0.0 {
            let pmt = pv / nper;
            return Ok((pmt, pv, 0.0));
        }
        let term = (1.0 + rate).powf(nper);
        let pmt = (pv * rate * term) / (term - 1.0);
        let total = pmt * nper;
        let interest = total - pv;
        Ok((pmt, total, interest))
    }

    /// Compound interest calculation: A = P(1 + r/n)^(nt)
    pub fn compound_interest(
        principal: f64,
        rate: f64,
        times_per_year: f64,
        years: f64,
    ) -> Result<(f64, f64)> {
        // Returns (final_amount, total_interest)
        let amount = principal * (1.0 + rate / times_per_year).powf(times_per_year * years);
        let interest = amount - principal;
        Ok((amount, interest))
    }

    /// Compound Annual Growth Rate (CAGR)
    pub fn cagr(beginning_val: f64, ending_val: f64, years: f64) -> Result<f64> {
        if beginning_val <= 0.0 || years <= 0.0 {
            bail!("Beginning value and years must be positive");
        }
        let rate = (ending_val / beginning_val).powf(1.0 / years) - 1.0;
        Ok(rate)
    }

    /// Return on Investment (ROI)
    pub fn roi(gain: f64, cost: f64) -> Result<f64> {
        if cost == 0.0 {
            bail!("Cost cannot be zero for ROI calculation");
        }
        Ok((gain - cost) / cost)
    }

    /// Net Present Value (NPV)
    pub fn npv(rate: f64, cash_flows: &[f64]) -> Result<f64> {
        let mut total = 0.0;
        for (t, &cf) in cash_flows.iter().enumerate() {
            total += cf / (1.0 + rate).powi(t as i32);
        }
        Ok(total)
    }
}

// ============================================================================
// 5. HIGH-PERFORMANCE UNIT CONVERTER
// ============================================================================

pub struct UnitConverter;

impl UnitConverter {
    pub fn convert(val: f64, from_unit: &str, to_unit: &str) -> Result<f64> {
        let from = from_unit.trim().to_lowercase();
        let to = to_unit.trim().to_lowercase();

        if from == to {
            return Ok(val);
        }

        // Temperature conversions
        if ["c", "celsius", "f", "fahrenheit", "k", "kelvin"].contains(&from.as_str()) {
            return Self::convert_temperature(val, &from, &to);
        }

        // Length in meters
        let length_units = [
            ("m", 1.0),
            ("meter", 1.0),
            ("km", 1000.0),
            ("kilometer", 1000.0),
            ("cm", 0.01),
            ("centimeter", 0.01),
            ("mm", 0.001),
            ("millimeter", 0.001),
            ("inch", 0.0254),
            ("in", 0.0254),
            ("foot", 0.3048),
            ("ft", 0.3048),
            ("yard", 0.9144),
            ("yd", 0.9144),
            ("mile", 1609.344),
            ("mi", 1609.344),
        ];

        // Mass in grams
        let mass_units = [
            ("g", 1.0),
            ("gram", 1.0),
            ("kg", 1000.0),
            ("kilogram", 1000.0),
            ("mg", 0.001),
            ("milligram", 0.001),
            ("ton", 1_000_000.0),
            ("t", 1_000_000.0),
            ("oz", 28.3495),
            ("ounce", 28.3495),
            ("lb", 453.592),
            ("pound", 453.592),
        ];

        // Digital Storage in bytes
        let digital_units = [
            ("b", 1.0),
            ("byte", 1.0),
            ("kb", 1000.0),
            ("kib", 1024.0),
            ("mb", 1_000_000.0),
            ("mib", 1_048_576.0),
            ("gb", 1_000_000_000.0),
            ("gib", 1_073_741_824.0),
            ("tb", 1_000_000_000_000.0),
            ("tib", 1_099_511_627_776.0),
        ];

        // Time in seconds
        let time_units = [
            ("s", 1.0),
            ("sec", 1.0),
            ("second", 1.0),
            ("ms", 0.001),
            ("millisecond", 0.001),
            ("min", 60.0),
            ("minute", 60.0),
            ("h", 3600.0),
            ("hr", 3600.0),
            ("hour", 3600.0),
            ("day", 86400.0),
            ("d", 86400.0),
            ("week", 604800.0),
            ("year", 31536000.0),
        ];

        if let (Some(f), Some(t)) = (
            Self::find_factor(&from, &length_units),
            Self::find_factor(&to, &length_units),
        ) {
            return Ok(val * f / t);
        }
        if let (Some(f), Some(t)) = (
            Self::find_factor(&from, &mass_units),
            Self::find_factor(&to, &mass_units),
        ) {
            return Ok(val * f / t);
        }
        if let (Some(f), Some(t)) = (
            Self::find_factor(&from, &digital_units),
            Self::find_factor(&to, &digital_units),
        ) {
            return Ok(val * f / t);
        }
        if let (Some(f), Some(t)) = (
            Self::find_factor(&from, &time_units),
            Self::find_factor(&to, &time_units),
        ) {
            return Ok(val * f / t);
        }

        bail!(
            "Unsupported unit conversion from '{}' to '{}'",
            from_unit,
            to_unit
        )
    }

    fn find_factor(unit: &str, table: &[(&str, f64)]) -> Option<f64> {
        table
            .iter()
            .find(|(name, _)| *name == unit)
            .map(|(_, factor)| *factor)
    }

    fn convert_temperature(val: f64, from: &str, to: &str) -> Result<f64> {
        // Convert to Kelvin first
        let k = match from {
            "c" | "celsius" => val + 273.15,
            "f" | "fahrenheit" => (val - 32.0) * 5.0 / 9.0 + 273.15,
            "k" | "kelvin" => val,
            _ => bail!("Unknown temperature unit: {}", from),
        };

        // Convert Kelvin to target
        let out = match to {
            "c" | "celsius" => k - 273.15,
            "f" | "fahrenheit" => (k - 273.15) * 9.0 / 5.0 + 32.0,
            "k" | "kelvin" => k,
            _ => bail!("Unknown temperature unit: {}", to),
        };

        Ok(out)
    }
}

// ============================================================================
// 6. DATE & CALENDAR COMPUTATION
// ============================================================================

pub struct DateCalculator;

impl DateCalculator {
    pub fn parse_ymd(s: &str) -> Result<(i32, u32, u32)> {
        let parts: Vec<&str> = s.trim().split('-').collect();
        if parts.len() != 3 {
            bail!("Invalid date format '{}', expected YYYY-MM-DD", s);
        }
        let y: i32 = parts[0]
            .parse()
            .map_err(|_| anyhow!("Invalid year in date"))?;
        let m: u32 = parts[1]
            .parse()
            .map_err(|_| anyhow!("Invalid month in date"))?;
        let d: u32 = parts[2]
            .parse()
            .map_err(|_| anyhow!("Invalid day in date"))?;
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            bail!("Date components out of valid range: {}", s);
        }
        Ok((y, m, d))
    }

    fn is_leap_year(y: i32) -> bool {
        (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
    }

    fn days_in_month(y: i32, m: u32) -> u32 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(y) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Rata Die days calculation (days since 0001-01-01)
    fn to_rata_die(y: i32, m: u32, d: u32) -> i64 {
        let mut days: i64 = d as i64;
        let year = y - 1;
        days += (year as i64) * 365 + (year as i64) / 4 - (year as i64) / 100 + (year as i64) / 400;
        for month in 1..m {
            days += Self::days_in_month(y, month) as i64;
        }
        days
    }

    pub fn days_between(start_date: &str, end_date: &str) -> Result<i64> {
        let (y1, m1, d1) = Self::parse_ymd(start_date)?;
        let (y2, m2, d2) = Self::parse_ymd(end_date)?;
        let rd1 = Self::to_rata_die(y1, m1, d1);
        let rd2 = Self::to_rata_die(y2, m2, d2);
        Ok(rd2 - rd1)
    }

    pub fn business_days_between(start_date: &str, end_date: &str) -> Result<i64> {
        let (y1, m1, d1) = Self::parse_ymd(start_date)?;
        let (y2, m2, d2) = Self::parse_ymd(end_date)?;
        let rd1 = Self::to_rata_die(y1, m1, d1);
        let rd2 = Self::to_rata_die(y2, m2, d2);

        let (from, to, sign) = if rd1 <= rd2 {
            (rd1, rd2, 1)
        } else {
            (rd2, rd1, -1)
        };

        let mut b_days = 0;
        for day in from..to {
            // In Rata Die, day 1 is Monday, day 7 is Sunday
            // day % 7: 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat, 0=Sun
            let day_of_week = day % 7;
            if day_of_week != 6 && day_of_week != 0 {
                b_days += 1;
            }
        }

        Ok(b_days * sign)
    }
}

// ============================================================================
// 7. CALC AGENT TOOL IMPLEMENTATION
// ============================================================================

#[derive(Default)]
pub struct CalcTool;

impl CalcTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CalcTool {
    fn name(&self) -> &'static str {
        "calc"
    }

    fn description(&self) -> &'static str {
        "High-performance scientific, statistical, financial, and unit calculation engine powered by pure Rust. Supports complex formulas, regressions, matrices, loan PMT, CAGR, unit conversions, and date calculations."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["eval", "stat", "regression", "matrix", "vector", "finance", "unit", "date"],
                    "description": "Calculation category: 'eval' (scientific formula), 'stat' (descriptive statistics), 'regression' (linear regression & correlation), 'matrix' (matrix operations), 'vector' (vector operations), 'finance' (loan PMT, compound interest, CAGR, ROI), 'unit' (unit conversions), 'date' (days difference and business days)"
                },
                "expr": {
                    "type": "string",
                    "description": "Mathematical expression string for 'eval', e.g. 'sin(pi/4) + sqrt(25) * 2^3' or 'ln(100) + fact(5)'"
                },
                "data": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "List of numeric values for 'stat' analysis"
                },
                "x": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "X coordinates for 'regression'"
                },
                "y": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Y coordinates for 'regression'"
                },
                "operation": {
                    "type": "string",
                    "description": "Operation sub-type for matrix/vector/finance/date"
                },
                "matrix_a": {
                    "type": "array",
                    "items": { "type": "array", "items": { "type": "number" } },
                    "description": "First 2D matrix for matrix operations"
                },
                "matrix_b": {
                    "type": "array",
                    "items": { "type": "array", "items": { "type": "number" } },
                    "description": "Second 2D matrix for matrix multiplication"
                },
                "vector_a": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "First vector for vector operations"
                },
                "vector_b": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Second vector for vector operations"
                },
                "principal": { "type": "number", "description": "Principal amount for finance" },
                "rate": { "type": "number", "description": "Interest rate or growth rate (as decimal, e.g. 0.05 for 5%)" },
                "periods": { "type": "number", "description": "Number of periods / years for finance" },
                "value": { "type": "number", "description": "Value to convert for unit conversion" },
                "from_unit": { "type": "string", "description": "Origin unit, e.g. 'km', 'celsius', 'gb', 'hour'" },
                "to_unit": { "type": "string", "description": "Target unit, e.g. 'mile', 'fahrenheit', 'gib', 'min'" },
                "start_date": { "type": "string", "description": "Start date 'YYYY-MM-DD' for date calculation" },
                "end_date": { "type": "string", "description": "End date 'YYYY-MM-DD' for date calculation" }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'action' parameter"))?;

        match action {
            "eval" => {
                let expr = arguments
                    .get("expr")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'expr' for eval action"))?;
                let val = MathEvaluator::eval(expr)?;
                Ok(format!(
                    "Expression: {}\nResult: {}\nFormatted: {:.8}",
                    expr, val, val
                ))
            }
            "stat" => {
                let data = arguments
                    .get("data")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing 'data' array for stat action"))?
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| anyhow!("Expected a numeric array element"))
                    })
                    .collect::<Result<Vec<f64>>>()?;
                let summary = StatEngine::compute(&data)?;
                let json_res = serde_json::to_string_pretty(&summary)?;
                let mut out = String::new();
                out.push_str(&format!(
                    "📊 Descriptive Statistics (N = {}):\n",
                    summary.count
                ));
                out.push_str(&format!("  • Sum: {:.4}\n", summary.sum));
                out.push_str(&format!("  • Mean (平均值): {:.4}\n", summary.mean));
                out.push_str(&format!("  • Median (中位数): {:.4}\n", summary.median));
                if let Some(m) = summary.mode {
                    out.push_str(&format!("  • Mode (众数): {:.4}\n", m));
                }
                out.push_str(&format!(
                    "  • Std Dev (标准差): {:.4} (Variance: {:.4})\n",
                    summary.std_dev, summary.variance
                ));
                out.push_str(&format!(
                    "  • Min: {:.4} | Max: {:.4} | Range: {:.4}\n",
                    summary.min, summary.max, summary.range
                ));
                out.push_str(&format!(
                    "  • IQR (四分位距): {:.4} [Q1: {:.4}, Q3: {:.4}]\n",
                    summary.iqr, summary.q1, summary.q3
                ));
                out.push_str(&format!(
                    "  • Percentiles: P90: {:.4}, P95: {:.4}, P99: {:.4}\n\n",
                    summary.p90, summary.p95, summary.p99
                ));
                out.push_str(&format!("Raw JSON:\n{}", json_res));
                Ok(out)
            }
            "regression" => {
                let x = arguments
                    .get("x")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing 'x' array for regression"))?
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| anyhow!("Expected a numeric array element"))
                    })
                    .collect::<Result<Vec<f64>>>()?;
                let y = arguments
                    .get("y")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow!("Missing 'y' array for regression"))?
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| anyhow!("Expected a numeric array element"))
                    })
                    .collect::<Result<Vec<f64>>>()?;
                let (slope, intercept, r, r_sq) = StatEngine::linear_regression(&x, &y)?;
                let sign = if intercept >= 0.0 { "+" } else { "-" };
                let out = format!(
                    "📈 Linear Regression Analysis:\n\
                     • Formula: y = {:.6} * x {} {:.6}\n\
                     • Slope (斜率 m): {:.6}\n\
                     • Intercept (截距 b): {:.6}\n\
                     • Pearson Correlation (相关系数 r): {:.6}\n\
                     • R² (拟合优度): {:.6} ({:.2}% variance explained)\n\
                     • Sample Size: {}",
                    slope,
                    sign,
                    intercept.abs(),
                    slope,
                    intercept,
                    r,
                    r_sq,
                    r_sq * 100.0,
                    x.len()
                );
                Ok(out)
            }
            "matrix" => {
                let op = arguments
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("multiply");
                match op {
                    "multiply" | "mul" => {
                        let a: Vec<Vec<f64>> = serde_json::from_value(
                            arguments.get("matrix_a").cloned().unwrap_or(json!([])),
                        )?;
                        let b: Vec<Vec<f64>> = serde_json::from_value(
                            arguments.get("matrix_b").cloned().unwrap_or(json!([])),
                        )?;
                        let res = LinearAlgebra::matrix_multiply(&a, &b)?;
                        let cols = res.first().map(|r| r.len()).unwrap_or(0);
                        Ok(format!(
                            "Matrix Multiplication Result ({}x{}):\n{}",
                            res.len(),
                            cols,
                            serde_json::to_string_pretty(&res)?
                        ))
                    }
                    "transpose" => {
                        let a: Vec<Vec<f64>> = serde_json::from_value(
                            arguments.get("matrix_a").cloned().unwrap_or(json!([])),
                        )?;
                        let res = LinearAlgebra::matrix_transpose(&a)?;
                        let cols = res.first().map(|r| r.len()).unwrap_or(0);
                        Ok(format!(
                            "Transposed Matrix ({}x{}):\n{}",
                            res.len(),
                            cols,
                            serde_json::to_string_pretty(&res)?
                        ))
                    }
                    "det" | "determinant" => {
                        let a: Vec<Vec<f64>> = serde_json::from_value(
                            arguments.get("matrix_a").cloned().unwrap_or(json!([])),
                        )?;
                        let det = LinearAlgebra::matrix_determinant(&a)?;
                        Ok(format!("Matrix Determinant: {:.8}", det))
                    }
                    unknown => bail!("Unknown matrix operation: '{}'", unknown),
                }
            }
            "vector" => {
                let op = arguments
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dot");
                let a: Vec<f64> = serde_json::from_value(
                    arguments.get("vector_a").cloned().unwrap_or(json!([])),
                )?;
                let b: Vec<f64> = serde_json::from_value(
                    arguments.get("vector_b").cloned().unwrap_or(json!([])),
                )?;
                match op {
                    "dot" => {
                        let dot = LinearAlgebra::vector_dot(&a, &b)?;
                        Ok(format!("Vector Dot Product: {:.8}", dot))
                    }
                    "cosine" | "similarity" => {
                        let sim = LinearAlgebra::vector_cosine_similarity(&a, &b)?;
                        Ok(format!(
                            "Vector Cosine Similarity: {:.8} (Angle: {:.2}°)",
                            sim,
                            sim.acos().to_degrees()
                        ))
                    }
                    "cross" => {
                        let cross = LinearAlgebra::vector_cross_3d(&a, &b)?;
                        Ok(format!("3D Cross Product: {:?}", cross))
                    }
                    "norm" => {
                        let norm = LinearAlgebra::vector_norm(&a);
                        Ok(format!("Vector Euclidean Norm (L2): {:.8}", norm))
                    }
                    unknown => bail!("Unknown vector operation: '{}'", unknown),
                }
            }
            "finance" => {
                let op = arguments
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pmt");
                match op {
                    "pmt" => {
                        let pv = arguments
                            .get("principal")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing principal"))?;
                        let rate = arguments
                            .get("rate")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing annual rate"))?;
                        let periods = arguments
                            .get("periods")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing periods (months)"))?;
                        let (monthly, total, interest) =
                            FinanceEngine::pmt(rate / 12.0, periods, pv)?;
                        Ok(format!(
                            "💰 Loan Payment (PMT) Calculation:\n\
                             • Monthly Payment (月供): {:.2}\n\
                             • Total Repayment (总还款额): {:.2}\n\
                             • Total Interest (总利息): {:.2}\n\
                             • Principal: {:.2} | Term: {} months | Annual Rate: {:.2}%",
                            monthly,
                            total,
                            interest,
                            pv,
                            periods,
                            rate * 100.0
                        ))
                    }
                    "compound" => {
                        let p = arguments
                            .get("principal")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing principal"))?;
                        let r = arguments
                            .get("rate")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing annual rate"))?;
                        let years = arguments
                            .get("periods")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing periods (years)"))?;
                        let (amount, interest) =
                            FinanceEngine::compound_interest(p, r, 12.0, years)?;
                        Ok(format!(
                            "📈 Compound Interest Result:\n\
                             • Final Future Value (本息合计): {:.2}\n\
                             • Total Interest Earned (收益利息): {:.2}\n\
                             • Principal: {:.2} | Rate: {:.2}% | Time: {} years (compounded monthly)",
                            amount, interest, p, r * 100.0, years
                        ))
                    }
                    "cagr" => {
                        let begin = arguments
                            .get("principal")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing beginning principal"))?;
                        let end = arguments
                            .get("value")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing ending value"))?;
                        let years = arguments
                            .get("periods")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing years"))?;
                        let cagr = FinanceEngine::cagr(begin, end, years)?;
                        Ok(format!(
                            "📊 CAGR (复合年均增长率): {:.4} ({:.2}%)",
                            cagr,
                            cagr * 100.0
                        ))
                    }
                    "roi" => {
                        let cost = arguments
                            .get("principal")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing investment cost"))?;
                        let gain = arguments
                            .get("value")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow!("Missing total gain/return"))?;
                        let roi = FinanceEngine::roi(gain, cost)?;
                        Ok(format!(
                            "📊 ROI (投资回报率): {:.4} ({:.2}%)",
                            roi,
                            roi * 100.0
                        ))
                    }
                    unknown => bail!("Unknown finance operation: '{}'", unknown),
                }
            }
            "unit" => {
                let val = arguments
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Missing value for unit conversion"))?;
                let from = arguments
                    .get("from_unit")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing from_unit"))?;
                let to = arguments
                    .get("to_unit")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing to_unit"))?;
                let converted = UnitConverter::convert(val, from, to)?;
                Ok(format!(
                    "🔄 Unit Conversion:\n{} {} = {} {}",
                    val, from, converted, to
                ))
            }
            "date" => {
                let d1 = arguments
                    .get("start_date")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing start_date (YYYY-MM-DD)"))?;
                let d2 = arguments
                    .get("end_date")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing end_date (YYYY-MM-DD)"))?;
                let days = DateCalculator::days_between(d1, d2)?;
                let b_days = DateCalculator::business_days_between(d1, d2)?;
                Ok(format!(
                    "📅 Date Calculation between {} and {}:\n\
                     • Calendar Days: {} days\n\
                     • Business Days (工作日，排除周六日): {} days\n\
                     • Approximate Weeks: {:.1} weeks",
                    d1,
                    d2,
                    days,
                    b_days,
                    (days as f64) / 7.0
                ))
            }
            unknown => bail!("Unknown calc action: '{}'", unknown),
        }
    }
}
