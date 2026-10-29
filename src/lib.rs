//! SigmaOS Calculator — arithmetic engine with command-query separation.
//!
//! Principles: Command-Query Separation, Single Responsibility, Encapsulation,
//! Composition over Inheritance, Defensive Programming, DRY.

use std::fmt;

/// Errors that can occur during calculation.
#[derive(Debug, PartialEq)]
pub enum CalcError {
    DivisionByZero,
    Overflow,
    InvalidInput(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "division by zero"),
            CalcError::Overflow => write!(f, "arithmetic overflow"),
            CalcError::InvalidInput(s) => write!(f, "invalid input: {s}"),
        }
    }
}

pub type CalcResult = Result<f64, CalcError>;

/// The calculator state — encapsulated, no public fields.
#[derive(Debug, Default, Clone)]
pub struct Calculator {
    history: Vec<String>,
    memory: f64,
}

impl Calculator {
    /// Create a new Calculator instance.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Query operations (no side effects on main state) ─────────────────────

    /// Return the current memory value.
    pub fn memory(&self) -> f64 {
        self.memory
    }

    /// Return a copy of the calculation history.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    // ── Command operations (mutate state, return Result) ─────────────────────

    /// Add two numbers.
    pub fn add(&mut self, a: f64, b: f64) -> CalcResult {
        let result = a + b;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
        self.record(format!("{a} + {b} = {result}"));
        Ok(result)
    }

    /// Subtract b from a.
    pub fn subtract(&mut self, a: f64, b: f64) -> CalcResult {
        let result = a - b;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
        self.record(format!("{a} - {b} = {result}"));
        Ok(result)
    }

    /// Multiply two numbers.
    pub fn multiply(&mut self, a: f64, b: f64) -> CalcResult {
        let result = a * b;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
        self.record(format!("{a} * {b} = {result}"));
        Ok(result)
    }

    /// Divide a by b. Returns Err if b is zero.
    pub fn divide(&mut self, a: f64, b: f64) -> CalcResult {
        if b == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        let result = a / b;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
        self.record(format!("{a} / {b} = {result}"));
        Ok(result)
    }

    /// Compute square root of x. Returns Err for negative input.
    pub fn sqrt(&mut self, x: f64) -> CalcResult {
        if x < 0.0 {
            return Err(CalcError::InvalidInput(format!("sqrt of negative number: {x}")));
        }
        let result = x.sqrt();
        self.record(format!("sqrt({x}) = {result}"));
        Ok(result)
    }

    /// Compute x raised to the power of exp.
    pub fn power(&mut self, x: f64, exp: f64) -> CalcResult {
        let result = x.powf(exp);
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if result.is_nan() {
            return Err(CalcError::InvalidInput(format!("{x}^{exp} is undefined")));
        }
        self.record(format!("{x}^{exp} = {result}"));
        Ok(result)
    }

    /// Store a value in memory.
    pub fn memory_store(&mut self, value: f64) {
        self.memory = value;
    }

    /// Recall memory value (also a query — no side effect).
    pub fn memory_recall(&self) -> f64 {
        self.memory
    }

    /// Clear all history and reset memory.
    pub fn clear(&mut self) {
        self.history.clear();
        self.memory = 0.0;
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn record(&mut self, entry: String) {
        self.history.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.add(2.0, 3.0).unwrap();
        // Assert
        assert_eq!(result, 5.0);
        assert_eq!(calc.history().len(), 1);
    }

    #[test]
    fn test_subtract() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.subtract(10.0, 4.0).unwrap();
        // Assert
        assert_eq!(result, 6.0);
    }

    #[test]
    fn test_multiply() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.multiply(3.0, 7.0).unwrap();
        // Assert
        assert_eq!(result, 21.0);
    }

    #[test]
    fn test_divide() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.divide(10.0, 4.0).unwrap();
        // Assert
        assert_eq!(result, 2.5);
    }

    #[test]
    fn test_divide_by_zero_returns_error() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.divide(5.0, 0.0);
        // Assert
        assert_eq!(result, Err(CalcError::DivisionByZero));
    }

    #[test]
    fn test_sqrt() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.sqrt(16.0).unwrap();
        // Assert
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_sqrt_negative_returns_error() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.sqrt(-1.0);
        // Assert
        assert!(matches!(result, Err(CalcError::InvalidInput(_))));
    }

    #[test]
    fn test_power() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        let result = calc.power(2.0, 10.0).unwrap();
        // Assert
        assert_eq!(result, 1024.0);
    }

    #[test]
    fn test_memory_store_recall() {
        // Arrange
        let mut calc = Calculator::new();
        // Act
        calc.memory_store(42.0);
        let recalled = calc.memory_recall();
        // Assert
        assert_eq!(recalled, 42.0);
    }

    #[test]
    fn test_clear_resets_state() {
        // Arrange
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0).unwrap();
        calc.memory_store(99.0);
        // Act
        calc.clear();
        // Assert
        assert!(calc.history().is_empty());
        assert_eq!(calc.memory(), 0.0);
    }
}
