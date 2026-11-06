//! SigmaOS-calculator — safe arithmetic expression evaluator.
//!
//! Inspired by Omarchy's `omacalc`: a calculator that evaluates infix
//! expressions without `eval`. Supports + - * / % ^, parentheses, unary
//! minus, and decimal numbers. Fail-fast on malformed input.

/// Errors returned by the evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    EmptyInput,
    UnexpectedToken(char),
    UnbalancedParenthesis,
    MissingOperand,
    DivisionByZero,
    TrailingTokens,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Num(f64),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    LParen,
    RParen,
    Neg, // unary minus
}

fn tokenize(input: &str) -> Result<Vec<Tok>, CalcError> {
    let mut toks = Vec::new();
    let mut chars = input.chars().peekable();
    let mut prev_was_operand = false;

    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut s = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        s.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let v: f64 = s.parse().map_err(|_| CalcError::UnexpectedToken('.'))?;
                toks.push(Tok::Num(v));
                prev_was_operand = true;
            }
            '+' => {
                chars.next();
                toks.push(Tok::Add);
                prev_was_operand = false;
            }
            '-' => {
                chars.next();
                // unary minus when no operand precedes it
                toks.push(if prev_was_operand { Tok::Sub } else { Tok::Neg });
                prev_was_operand = false;
            }
            '*' => {
                chars.next();
                toks.push(Tok::Mul);
                prev_was_operand = false;
            }
            '/' => {
                chars.next();
                toks.push(Tok::Div);
                prev_was_operand = false;
            }
            '%' => {
                chars.next();
                toks.push(Tok::Mod);
                prev_was_operand = false;
            }
            '^' => {
                chars.next();
                toks.push(Tok::Pow);
                prev_was_operand = false;
            }
            '(' => {
                chars.next();
                toks.push(Tok::LParen);
                prev_was_operand = false;
            }
            ')' => {
                chars.next();
                toks.push(Tok::RParen);
                prev_was_operand = true;
            }
            other => return Err(CalcError::UnexpectedToken(other)),
        }
    }
    Ok(toks)
}

/// Recursive-descent parser/evaluator.
/// Grammar: expr := term (('+'|'-') term)* ; term := unary (('*'|'/'|'%') unary)*
/// unary := '-' unary | power ; power := primary ('^' unary)? (right assoc)
/// primary := number | '(' expr ')'
struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Tok> {
        self.toks.get(self.pos).copied()
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.peek();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_expr(&mut self) -> Result<f64, CalcError> {
        let mut lhs = self.parse_term()?;
        while matches!(self.peek(), Some(Tok::Add) | Some(Tok::Sub)) {
            let op = self.next().unwrap();
            let rhs = self.parse_term()?;
            lhs = match op {
                Tok::Add => lhs + rhs,
                Tok::Sub => lhs - rhs,
                _ => unreachable!(),
            };
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<f64, CalcError> {
        let mut lhs = self.parse_unary()?;
        while matches!(self.peek(), Some(Tok::Mul) | Some(Tok::Div) | Some(Tok::Mod)) {
            let op = self.next().unwrap();
            let rhs = self.parse_unary()?;
            lhs = match op {
                Tok::Mul => lhs * rhs,
                Tok::Div => {
                    if rhs == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    lhs / rhs
                }
                Tok::Mod => {
                    if rhs == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    lhs % rhs
                }
                _ => unreachable!(),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<f64, CalcError> {
        if matches!(self.peek(), Some(Tok::Neg)) {
            self.next();
            return Ok(-self.parse_unary()?);
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<f64, CalcError> {
        let base = self.parse_primary()?;
        if matches!(self.peek(), Some(Tok::Pow)) {
            self.next();
            // right-associative: 2^3^2 = 2^(3^2)
            let exp = self.parse_unary()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<f64, CalcError> {
        match self.next() {
            Some(Tok::Num(v)) => Ok(v),
            Some(Tok::LParen) => {
                let v = self.parse_expr()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(v),
                    _ => Err(CalcError::UnbalancedParenthesis),
                }
            }
            Some(_) => Err(CalcError::MissingOperand),
            None => Err(CalcError::MissingOperand),
        }
    }
}

/// Evaluate an arithmetic expression string. Safe: no eval, bounded
/// recursion depth is implicit in expression length.
pub struct CalculatorEngine;

impl CalculatorEngine {
    pub fn evaluate(expression: &str) -> Result<f64, CalcError> {
        let trimmed = expression.trim();
        if trimmed.is_empty() {
            return Err(CalcError::EmptyInput);
        }
        let toks = tokenize(trimmed)?;
        let mut p = Parser { toks: &toks, pos: 0 };
        let v = p.parse_expr()?;
        if p.pos != toks.len() {
            return Err(CalcError::TrailingTokens);
        }
        Ok(v)
    }

    // Backwards-compatible helpers
    pub fn evaluate_add(a: f64, b: f64) -> f64 {
        a + b
    }
    pub fn evaluate_multiply(a: f64, b: f64) -> f64 {
        a * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(CalculatorEngine::evaluate("1+2*3").unwrap(), 7.0);
        assert_eq!(CalculatorEngine::evaluate("(1+2)*3").unwrap(), 9.0);
        assert_eq!(CalculatorEngine::evaluate("10/4").unwrap(), 2.5);
        assert_eq!(CalculatorEngine::evaluate("10%3").unwrap(), 1.0);
    }

    #[test]
    fn test_power_and_unary() {
        assert_eq!(CalculatorEngine::evaluate("2^3^2").unwrap(), 512.0); // right assoc
        assert_eq!(CalculatorEngine::evaluate("-3^2").unwrap(), -9.0); // unary binds looser than ^? (-3)^2=9 vs -(3^2)=-9 -> we produce -9
        assert_eq!(CalculatorEngine::evaluate("--5").unwrap(), 5.0);
        assert_eq!(CalculatorEngine::evaluate("3*-2").unwrap(), -6.0);
    }

    #[test]
    fn test_errors() {
        assert_eq!(CalculatorEngine::evaluate(""), Err(CalcError::EmptyInput));
        assert_eq!(CalculatorEngine::evaluate("1+"), Err(CalcError::MissingOperand));
        assert_eq!(CalculatorEngine::evaluate("(1+2"), Err(CalcError::UnbalancedParenthesis));
        assert_eq!(CalculatorEngine::evaluate("1 2"), Err(CalcError::TrailingTokens));
        assert_eq!(CalculatorEngine::evaluate("1/0"), Err(CalcError::DivisionByZero));
        assert_eq!(CalculatorEngine::evaluate("1@2"), Err(CalcError::UnexpectedToken('@')));
    }

    #[test]
    fn test_legacy_helpers() {
        assert_eq!(CalculatorEngine::evaluate_add(10.5, 5.5), 16.0);
        assert_eq!(CalculatorEngine::evaluate_multiply(4.0, 5.0), 20.0);
    }
}
