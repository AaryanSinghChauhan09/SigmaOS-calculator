"""Safe scientific calculator engine for SigmaOS-calculator (inspired by omacalc).

Principles: least astonishment, defensive programming (no eval of arbitrary
code -- AST-whitelisted evaluator), fail-fast on bad input, ZERO-COPY of the
parsed token stream.
"""
import ast, operator, math, argparse, sys

ENGINE = {
    "name": "sigma-calc",
    "summary": "Safe expression calculator",
    "language": "python",
    "verbs": ["eval", "repl"],
    "principles": ["Abstraction", "Least Astonishment", "Defensive Programming",
                   "Fail-Fast", "KISS", "Separation of Concerns"],
}

cli_doc = """\
$ sigma-calc eval "2 + 3 * 4"
14
$ sigma-calc eval "sqrt(16) + sin(0)"
4.0
$ sigma-calc repl
"""

_BINOPS = {
    ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul,
    ast.Div: operator.truediv, ast.FloorDiv: operator.floordiv,
    ast.Mod: operator.mod, ast.Pow: operator.pow,
}
_UNARY = {ast.UAdd: operator.pos, ast.USub: operator.neg}
_FUNCS = {
    "sqrt": math.sqrt, "sin": math.sin, "cos": math.cos, "tan": math.tan,
    "log": math.log, "log10": math.log10, "exp": math.exp, "abs": abs,
    "floor": math.floor, "ceil": math.ceil, "round": round,
}
_CONSTS = {"pi": math.pi, "e": math.e, "tau": math.tau}


class CalcError(ValueError):
    pass


def _eval(node):
    if isinstance(node, ast.Expression):
        return _eval(node.body)
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
        return node.value
    if isinstance(node, ast.Name) and node.id in _CONSTS:
        return _CONSTS[node.id]
    if isinstance(node, ast.BinOp) and type(node.op) in _BINOPS:
        return _BINOPS[type(node.op)](_eval(node.left), _eval(node.right))
    if isinstance(node, ast.UnaryOp) and type(node.op) in _UNARY:
        return _UNARY[type(node.op)](_eval(node.operand))
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
        fn = _FUNCS.get(node.func.id)
        if fn is None:
            raise CalcError(f"unknown function: {node.func.id}")
        args = [_eval(a) for a in node.args]
        return fn(*args)
    raise CalcError(f"unsupported syntax: {ast.dump(node)}")


def evaluate(expr: str) -> float:
    """Evaluate a math expression safely. Raises CalcError on invalid input."""
    if not isinstance(expr, str) or not expr.strip():
        raise CalcError("empty expression")
    try:
        tree = ast.parse(expr.strip(), mode="eval")
    except SyntaxError as e:
        raise CalcError(f"syntax error: {e.msg}") from e
    return _eval(tree)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog=ENGINE["name"], description=ENGINE["summary"])
    sub = p.add_subparsers(dest="cmd", required=True)
    ev = sub.add_parser("eval", help="evaluate an expression")
    ev.add_argument("expr", nargs="+", help="math expression")
    sub.add_parser("repl", help="interactive prompt")
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if args.cmd == "eval":
        try:
            print(evaluate(" ".join(args.expr)))
            return 0
        except (CalcError, ZeroDivisionError) as e:
            print(f"error: {e}", file=sys.stderr)
            return 1
    if args.cmd == "repl":
        while True:
            try:
                line = input("calc> ").strip()
            except (EOFError, KeyboardInterrupt):
                print()
                return 0
            if not line:
                continue
            try:
                print(evaluate(line))
            except (CalcError, ZeroDivisionError) as e:
                print(f"error: {e}", file=sys.stderr)
    return 0


def self_test() -> None:
    assert evaluate("2 + 3 * 4") == 14
    assert evaluate("sqrt(16)") == 4.0
    assert abs(evaluate("sin(0)")) < 1e-12
    assert evaluate("(10 - 4) / 3") == 2.0
    assert evaluate("-5 + 3") == -2
    assert evaluate("2 ** 10") == 1024
    assert abs(evaluate("pi") - 3.14159265) < 1e-6
    for bad in ("__import__('os')", "open('/etc/passwd')", "1 +", ""):
        try:
            evaluate(bad)
        except CalcError:
            pass
        else:
            raise AssertionError(f"should have rejected: {bad!r}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
