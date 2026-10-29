"""
SigmaOS-calculator tests.

Principles: Arrange-Act-Assert, Fast-Isolated-Repeatable-Self-Validating-Timely,
Deterministic Testing, Defensive Programming.
"""
import math
import unittest

from src.core import CalcError, evaluate


class TestArithmetic(unittest.TestCase):
    """Basic arithmetic operations — Arrange-Act-Assert pattern throughout."""

    def test_addition(self):
        # Arrange
        expr = "2 + 3"
        # Act
        result = evaluate(expr)
        # Assert
        self.assertEqual(result, 5)

    def test_subtraction(self):
        # Arrange / Act / Assert
        self.assertEqual(evaluate("10 - 4"), 6)

    def test_multiplication(self):
        self.assertEqual(evaluate("3 * 7"), 21)

    def test_division(self):
        self.assertEqual(evaluate("10 / 4"), 2.5)

    def test_floor_division(self):
        self.assertEqual(evaluate("10 // 3"), 3)

    def test_modulo(self):
        self.assertEqual(evaluate("10 % 3"), 1)

    def test_power(self):
        self.assertEqual(evaluate("2 ** 10"), 1024)

    def test_operator_precedence(self):
        # 2 + 3 * 4 = 14, not 20
        self.assertEqual(evaluate("2 + 3 * 4"), 14)

    def test_parentheses_override_precedence(self):
        self.assertEqual(evaluate("(2 + 3) * 4"), 20)

    def test_unary_negation(self):
        self.assertEqual(evaluate("-5 + 3"), -2)


class TestMathFunctions(unittest.TestCase):

    def test_sqrt(self):
        self.assertEqual(evaluate("sqrt(16)"), 4.0)

    def test_sin_zero(self):
        self.assertAlmostEqual(evaluate("sin(0)"), 0.0, places=10)

    def test_cos_zero(self):
        self.assertAlmostEqual(evaluate("cos(0)"), 1.0, places=10)

    def test_log_e(self):
        self.assertAlmostEqual(evaluate("log(e)"), 1.0, places=10)

    def test_abs_negative(self):
        self.assertEqual(evaluate("abs(-42)"), 42)

    def test_floor(self):
        self.assertEqual(evaluate("floor(3.7)"), 3)

    def test_ceil(self):
        self.assertEqual(evaluate("ceil(3.2)"), 4)


class TestConstants(unittest.TestCase):

    def test_pi_constant(self):
        self.assertAlmostEqual(evaluate("pi"), math.pi, places=10)

    def test_e_constant(self):
        self.assertAlmostEqual(evaluate("e"), math.e, places=10)

    def test_tau_constant(self):
        self.assertAlmostEqual(evaluate("tau"), math.tau, places=10)


class TestSecurityAndDefensiveProgramming(unittest.TestCase):
    """Ensure the safe evaluator blocks dangerous expressions (Defensive Programming)."""

    def test_import_rejected(self):
        # Arrange
        dangerous = "__import__('os')"
        # Act / Assert — must raise CalcError, not execute
        with self.assertRaises(CalcError):
            evaluate(dangerous)

    def test_open_rejected(self):
        with self.assertRaises(CalcError):
            evaluate("open('/etc/passwd')")

    def test_empty_expression_rejected(self):
        with self.assertRaises(CalcError):
            evaluate("")

    def test_syntax_error_raises_calc_error(self):
        with self.assertRaises(CalcError):
            evaluate("1 +")

    def test_unknown_function_raises_calc_error(self):
        with self.assertRaises(CalcError):
            evaluate("evil(1)")

    def test_zero_division_raises(self):
        with self.assertRaises(ZeroDivisionError):
            evaluate("1 / 0")


class TestSelf(unittest.TestCase):
    def test_self_test_passes(self):
        from src.core import self_test
        self_test()  # must not raise


if __name__ == "__main__":
    unittest.main()
