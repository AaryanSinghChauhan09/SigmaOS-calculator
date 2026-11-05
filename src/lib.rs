pub struct CalculatorEngine;
impl CalculatorEngine {
    pub fn evaluate_add(a: f64, b: f64) -> f64 { a + b }
    pub fn evaluate_multiply(a: f64, b: f64) -> f64 { a * b }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculator_math() {
        assert_eq!(CalculatorEngine::evaluate_add(10.5, 5.5), 16.0);
        assert_eq!(CalculatorEngine::evaluate_multiply(4.0, 5.0), 20.0);
    }
}
