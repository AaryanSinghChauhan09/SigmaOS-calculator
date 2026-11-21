// SPDX-License-Identifier: Apache-2.0
//!
//! SigmaOS-calculator — expression evaluator & calculator history.
//!
//! Tokenizes and evaluates arithmetic expressions with + - * / % ^ ( ),
//! unary minus, constants (pi, e), functions (sqrt, sin, cos, tan, ln,
//! log, abs, floor, ceil, round), and implicit multiplication. Keeps a
//! persistent history of evaluations. All parsing is deterministic —
//! shunting-yard to RPN then stack evaluation.

use std::collections::VecDeque;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    EmptyExpr,
    BadToken(String),
    Unbalanced(String),
    DivByZero,
    DomainError(String),
    TooManyArgs(String),
    UnknownFunction(String),
    Syntax(String),
}
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExpr=>write!(f,"empty expression"),
            Self::BadToken(t)=>write!(f,"bad token: {}",t),
            Self::Unbalanced(m)=>write!(f,"unbalanced: {}",m),
            Self::DivByZero=>write!(f,"division by zero"),
            Self::DomainError(m)=>write!(f,"domain: {}",m),
            Self::TooManyArgs(n)=>write!(f,"too many args for {}",n),
            Self::UnknownFunction(n)=>write!(f,"unknown function: {}",n),
            Self::Syntax(m)=>write!(f,"syntax: {}",m),
        }
    }
}
impl std::error::Error for CalcError {}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),          // + - * / % ^
    LParen,
    RParen,
    Comma,
    Func(String),
    Ident(String),     // constants: pi, e, tau
    UnaryNeg,
}

/// Tokenize an expression string.
pub fn tokenize(input: &str) -> Result<Vec<Tok>, CalcError> {
    let chars: Vec<char> = input.chars().collect();
    let mut toks = vec![];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            let mut seen_dot = false;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                if chars[i] == '.' {
                    if seen_dot { return Err(CalcError::BadToken("multiple dots".into())); }
                    seen_dot = true;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            let n: f64 = s.parse().map_err(|_| CalcError::BadToken(s.clone()))?;
            toks.push(Tok::Num(n));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') { i += 1; }
            let name: String = chars[start..i].iter().collect::<String>().to_lowercase();
            match name.as_str() {
                "pi" => toks.push(Tok::Num(std::f64::consts::PI)),
                "e" => toks.push(Tok::Num(std::f64::consts::E)),
                "tau" => toks.push(Tok::Num(std::f64::consts::TAU)),
                _ => toks.push(Tok::Func(name)),
            }
            continue;
        }
        match c {
            '+' | '-' | '*' | '/' | '%' | '^' => {
                // unary minus detection: '-' at start or after op/lparen/comma
                if c == '-' {
                    let prev_op = toks.last().map(|t| matches!(t,
                        Tok::Op(_) | Tok::LParen | Tok::Comma)).unwrap_or(true);
                    if prev_op { toks.push(Tok::UnaryNeg); i += 1; continue; }
                }
                toks.push(Tok::Op(c));
            }
            '(' => toks.push(Tok::LParen),
            ')' => toks.push(Tok::RParen),
            ',' => toks.push(Tok::Comma),
            _ => return Err(CalcError::BadToken(c.to_string())),
        }
        i += 1;
    }
    // implicit multiplication: Num/Func/RParen followed by Num/Func/LParen
    let mut out: Vec<Tok> = vec![];
    for t in toks {
        if let Some(last) = out.last() {
            let implicit = matches!(last, Tok::Num(_) | Tok::RParen)
                && matches!(t, Tok::Num(_) | Tok::Func(_) | Tok::LParen);
            if implicit { out.push(Tok::Op('*')); }
        }
        out.push(t);
    }
    Ok(out)
}

fn prec(op: char) -> u8 {
    match op { '^' => 4, '*' | '/' | '%' => 3, '+' | '-' => 2, _ => 0 }
}
fn right_assoc(op: char) -> bool { op == '^' }

/// Convert tokens to RPN via shunting-yard.
pub fn to_rpn(toks: &[Tok]) -> Result<Vec<Tok>, CalcError> {
    let mut out: Vec<Tok> = vec![];
    let mut ops: Vec<Tok> = vec![];
    let mut parens = 0i32;
    for t in toks {
        match t {
            Tok::Num(_) | Tok::Ident(_) => out.push(t.clone()),
            Tok::Func(_) => ops.push(t.clone()),
            Tok::UnaryNeg => ops.push(t.clone()),
            Tok::Op(c) => {
                while let Some(top) = ops.last() {
                    let top_prec = match top {
                        Tok::Op(o) => prec(*o),
                        Tok::UnaryNeg => 5,
                        _ => 0,
                    };
                    let assoc_ok = match top {
                        Tok::Op(o) => prec(*o) > prec(*c) || (prec(*o) == prec(*c) && !right_assoc(*c)),
                        Tok::UnaryNeg => true,
                        _ => false,
                    };
                    let _ = top_prec;
                    if assoc_ok {
                        out.push(ops.pop().unwrap());
                    } else { break; }
                }
                ops.push(t.clone());
            }
            Tok::LParen => { parens += 1; ops.push(t.clone()); }
            Tok::RParen => {
                parens -= 1;
                if parens < 0 { return Err(CalcError::Unbalanced("extra )".into())); }
                let mut closed = false;
                while let Some(top) = ops.pop() {
                    if matches!(top, Tok::LParen) { closed = true; break; }
                    out.push(top);
                }
                if closed {
                    if let Some(Tok::Func(_)) = ops.last() {
                        out.push(ops.pop().unwrap());
                    }
                }
            }
            Tok::Comma => {
                while let Some(top) = ops.last() {
                    if matches!(top, Tok::LParen) { break; }
                    out.push(ops.pop().unwrap());
                }
            }
        }
    }
    if parens != 0 { return Err(CalcError::Unbalanced("unclosed (".into())); }
    while let Some(op) = ops.pop() {
        if matches!(op, Tok::LParen) { return Err(CalcError::Unbalanced("unclosed (".into())); }
        out.push(op);
    }
    Ok(out)
}

fn apply_func(name: &str, args: &[f64]) -> Result<f64, CalcError> {
    let need = |n: usize| -> Result<(), CalcError> {
        if args.len() != n { Err(CalcError::TooManyArgs(name.into())) } else { Ok(()) }
    };
    match name {
        "sqrt" => { need(1)?; if args[0] < 0.0 { Err(CalcError::DomainError("sqrt of negative".into())) } else { Ok(args[0].sqrt()) } }
        "sin" => { need(1)?; Ok(args[0].sin()) }
        "cos" => { need(1)?; Ok(args[0].cos()) }
        "tan" => { need(1)?; Ok(args[0].tan()) }
        "asin" => { need(1)?; if args[0].abs() > 1.0 { Err(CalcError::DomainError("asin domain".into())) } else { Ok(args[0].asin()) } }
        "acos" => { need(1)?; if args[0].abs() > 1.0 { Err(CalcError::DomainError("acos domain".into())) } else { Ok(args[0].acos()) } }
        "atan" => { need(1)?; Ok(args[0].atan()) }
        "ln" => { need(1)?; if args[0] <= 0.0 { Err(CalcError::DomainError("ln of non-positive".into())) } else { Ok(args[0].ln()) } }
        "log" => { need(1)?; if args[0] <= 0.0 { Err(CalcError::DomainError("log of non-positive".into())) } else { Ok(args[0].log10()) } }
        "log2" => { need(1)?; if args[0] <= 0.0 { Err(CalcError::DomainError("log2 of non-positive".into())) } else { Ok(args[0].log2()) } }
        "abs" => { need(1)?; Ok(args[0].abs()) }
        "floor" => { need(1)?; Ok(args[0].floor()) }
        "ceil" => { need(1)?; Ok(args[0].ceil()) }
        "round" => { need(1)?; Ok(args[0].round()) }
        "exp" => { need(1)?; Ok(args[0].exp()) }
        "pow" => { need(2)?; Ok(args[0].powf(args[1])) }
        "min" => { need(2)?; Ok(args[0].min(args[1])) }
        "max" => { need(2)?; Ok(args[0].max(args[1])) }
        "hypot" => { need(2)?; Ok(args[0].hypot(args[1])) }
        _ => Err(CalcError::UnknownFunction(name.into())),
    }
}

/// Evaluate an expression string → f64.
pub fn eval(input: &str) -> Result<f64, CalcError> {
    let toks = tokenize(input)?;
    if toks.is_empty() { return Err(CalcError::EmptyExpr); }
    let rpn = to_rpn(&toks)?;
    eval_rpn(&rpn)
}

fn eval_rpn(rpn: &[Tok]) -> Result<f64, CalcError> {
    let mut stack: Vec<f64> = vec![];
    // for function calls we need arg counts — functions are arity-known
    let arity = |name: &str| -> usize {
        match name {
            "pow"|"min"|"max"|"hypot" => 2,
            _ => 1,
        }
    };
    for t in rpn {
        match t {
            Tok::Num(n) => stack.push(*n),
            Tok::UnaryNeg => {
                let v = stack.pop().ok_or_else(|| CalcError::Syntax("missing operand".into()))?;
                stack.push(-v);
            }
            Tok::Func(name) => {
                let n = arity(name);
                if stack.len() < n { return Err(CalcError::Syntax(format!("{} needs {} args", name, n))); }
                let args: Vec<f64> = stack.split_off(stack.len() - n);
                let v = apply_func(name, &args)?;
                stack.push(v);
            }
            Tok::Op(c) => {
                let b = stack.pop().ok_or_else(|| CalcError::Syntax("missing operand".into()))?;
                let a = stack.pop().ok_or_else(|| CalcError::Syntax("missing operand".into()))?;
                let v = match c {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => { if b == 0.0 { return Err(CalcError::DivByZero); } a / b }
                    '%' => { if b == 0.0 { return Err(CalcError::DivByZero); } a % b }
                    '^' => a.powf(b),
                    _ => return Err(CalcError::BadToken(c.to_string())),
                };
                stack.push(v);
            }
            _ => return Err(CalcError::Syntax("unexpected token in rpn".into())),
        }
    }
    if stack.len() != 1 {
        return Err(CalcError::Syntax(format!("expected 1 result, got {}", stack.len())));
    }
    Ok(stack[0])
}

/// Format result nicely.
pub fn fmt_result(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// History entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CalcRecord {
    pub expr: String,
    pub result: String,
    pub ts: u64,
}

/// Calculator with persistent history.
#[derive(Debug, Default)]
pub struct SigmaOSCalculator {
    pub history: VecDeque<CalcRecord>,
    pub max_history: usize,
}
impl SigmaOSCalculator {
    pub fn new() -> Self {
        Self { history: VecDeque::new(), max_history: 200 }
    }

    /// Evaluate and record.
    pub fn calculate(&mut self, expr: &str, now_unix: u64) -> Result<String, CalcError> {
        let v = eval(expr)?;
        let result = fmt_result(v);
        self.history.push_front(CalcRecord {
            expr: expr.to_string(), result: result.clone(), ts: now_unix,
        });
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }
        Ok(result)
    }

    /// Evaluate without recording.
    pub fn evaluate(&self, expr: &str) -> Result<String, CalcError> {
        eval(expr).map(fmt_result)
    }

    pub fn history(&self) -> &VecDeque<CalcRecord> { &self.history }

    /// Search history.
    pub fn search_history(&self, needle: &str) -> Vec<&CalcRecord> {
        let n = needle.to_lowercase();
        self.history.iter()
            .filter(|r| r.expr.to_lowercase().contains(&n) || r.result.contains(needle))
            .collect()
    }

    pub fn clear_history(&mut self) { self.history.clear(); }
}

// ---- persistence ----
#[derive(serde::Serialize, serde::Deserialize)]
struct Persist { history: Vec<CalcRecord> }
impl SigmaOSCalculator {
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let p = Persist { history: self.history.iter().cloned().collect() };
        let j = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
        std::fs::write(path, j).map_err(|e| e.to_string())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let txt = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let p: Persist = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
        let mut c = Self::new();
        for r in p.history { c.history.push_back(r); }
        Ok(c)
    }
    pub fn default_path() -> PathBuf {
        if let Ok(p) = std::env::var("SIGMAOS_CALC_FILE") { return PathBuf::from(p); }
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
            .join(".local/share/sigmaos/calculator.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        assert_eq!(eval("1+2").unwrap(), 3.0);
        assert_eq!(eval("2*3+4").unwrap(), 10.0);
        assert_eq!(eval("2*(3+4)").unwrap(), 14.0);
        assert_eq!(eval("10/4").unwrap(), 2.5);
        assert_eq!(eval("10%3").unwrap(), 1.0);
        assert_eq!(eval("2^3").unwrap(), 8.0);
        assert_eq!(eval("2^3^2").unwrap(), 512.0); // right assoc
    }

    #[test]
    fn unary_and_implicit() {
        assert_eq!(eval("-5+3").unwrap(), -2.0);
        assert_eq!(eval("-(3+4)").unwrap(), -7.0);
        assert_eq!(eval("2(3+4)").unwrap(), 14.0); // implicit mul
        assert_eq!(eval("3pi").unwrap(), 3.0 * std::f64::consts::PI);
    }

    #[test]
    fn functions() {
        assert_eq!(eval("sqrt(16)").unwrap(), 4.0);
        assert_eq!(eval("abs(-5)").unwrap(), 5.0);
        assert_eq!(eval("floor(3.7)").unwrap(), 3.0);
        assert_eq!(eval("pow(2,10)").unwrap(), 1024.0);
        assert_eq!(eval("max(3,7)").unwrap(), 7.0);
        assert_eq!(eval("min(3,7)").unwrap(), 3.0);
        assert!((eval("sin(pi/2)").unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn errors() {
        assert!(matches!(eval("1/0"), Err(CalcError::DivByZero)));
        assert!(matches!(eval("sqrt(-1)"), Err(CalcError::DomainError(_))));
        assert!(matches!(eval("ln(0)"), Err(CalcError::DomainError(_))));
        assert!(matches!(eval("foo(1)"), Err(CalcError::UnknownFunction(_))));
        assert!(matches!(eval("(1+2"), Err(CalcError::Unbalanced(_))));
        assert!(matches!(eval("1+"), Err(_)));
        assert!(matches!(eval(""), Err(CalcError::EmptyExpr)));
    }

    #[test]
    fn constants() {
        assert!((eval("pi").unwrap() - std::f64::consts::PI).abs() < 1e-12);
        assert!((eval("e").unwrap() - std::f64::consts::E).abs() < 1e-12);
        assert!((eval("tau").unwrap() - std::f64::consts::TAU).abs() < 1e-12);
    }

    #[test]
    fn history_recorded() {
        let mut c = SigmaOSCalculator::new();
        let r = c.calculate("2+2", 1000).unwrap();
        assert_eq!(r, "4");
        assert_eq!(c.history.len(), 1);
        assert_eq!(c.history[0].expr, "2+2");
        assert_eq!(c.history[0].ts, 1000);
        // evaluate doesn't record
        c.evaluate("5*5").unwrap();
        assert_eq!(c.history.len(), 1);
    }

    #[test]
    fn history_search_and_cap() {
        let mut c = SigmaOSCalculator::new();
        c.max_history = 5;
        for i in 0..10 {
            c.calculate(&format!("{}+1", i), i as u64).unwrap();
        }
        assert_eq!(c.history.len(), 5);
        // newest first
        assert_eq!(c.history[0].expr, "9+1");
        let found = c.search_history("5+");
        assert_eq!(found.len(), 1);
        c.clear_history();
        assert!(c.history.is_empty());
    }

    #[test]
    fn fmt_result_test() {
        assert_eq!(fmt_result(4.0), "4");
        assert_eq!(fmt_result(2.5), "2.5");
        assert_eq!(fmt_result(0.1+0.2), "0.3");
    }

    #[test]
    fn persistence_roundtrip() {
        let dir = std::env::temp_dir().join(format!("sigmaos-calc-{}", std::process::id()));
        let p = dir.join("calc.json");
        let mut c = SigmaOSCalculator::new();
        c.calculate("1+1", 100).unwrap();
        c.calculate("2*3", 200).unwrap();
        c.save(&p).unwrap();
        let c2 = SigmaOSCalculator::load(&p).unwrap();
        assert_eq!(c2.history.len(), 2);
        assert_eq!(c2.history[0].expr, "2*3"); // newest first
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn complex_expression() {
        // 2 + 3 * 4 ^ 2 - sqrt(16) / 2
        let v = eval("2 + 3 * 4^2 - sqrt(16)/2").unwrap();
        // 4^2=16, 3*16=48, sqrt(16)=4, 4/2=2 → 2+48-2 = 48
        assert!((v - 48.0).abs() < 1e-9);
    }
}
