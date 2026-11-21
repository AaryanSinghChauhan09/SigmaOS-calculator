//! SigmaOS-calculator — calculator CLI.
//!
//! Usage:
//!   sigmaos-calc "EXPR"           evaluate expression
//!   sigmaos-calc history [N]      show history
//!   sigmaos-calc search TERM      search history
//!   sigmaos-calc clear            clear history
//!   sigmaos-calc functions        list functions
//!   sigmaos-calc repl             interactive (stdin lines)
//!
//! History: ~/.local/share/sigmaos/calculator.json

use sigmaos_calculator::{SigmaOSCalculator};
use std::io::{self, BufRead, Write};
use std::process::exit;

fn usage() -> ! {
    eprintln!("usage: sigmaos-calc <expr|history|search|clear|functions|repl> [args]");
    exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { usage(); }
    let db = SigmaOSCalculator::default_path();
    let mut c = SigmaOSCalculator::load(&db).unwrap_or_else(|_| SigmaOSCalculator::new());
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0);

    let result: Result<(), String> = (|| -> Result<(), String> { match args[0].as_str() {
        "functions" => {
            println!("sqrt sin cos tan asin acos atan ln log log2 abs floor ceil round exp pow min max hypot");
            println!("constants: pi e tau");
            println!("operators: + - * / % ^  ()  implicit multiplication (2pi, 2(3+4))");
            Ok(())
        }
        "history" => {
            let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);
            for r in c.history().iter().take(n) {
                println!("{:<24} = {:<16} ({})", r.expr, r.result, r.ts);
            }
            Ok(())
        }
        "search" => {
            let term = args.get(1).map(|s| s.as_str()).unwrap_or("");
            for r in c.search_history(term) {
                println!("{:<24} = {:<16} ({})", r.expr, r.result, r.ts);
            }
            Ok(())
        }
        "clear" => { c.clear_history(); println!("history cleared"); Ok(()) }
        "repl" => {
            println!("sigmaos-calc repl — enter expressions, 'q' to quit");
            let stdin = io::stdin();
            let mut out = io::stdout();
            for line in stdin.lock().lines() {
                let line = line.map_err(|e| e.to_string())?;
                let line = line.trim();
                if line == "q" || line == "quit" { break; }
                if line.is_empty() { continue; }
                match c.calculate(line, now) {
                    Ok(r) => { let _ = writeln!(out, "{}", r); let _ = out.flush(); }
                    Err(e) => { let _ = writeln!(out, "error: {}", e); let _ = out.flush(); }
                }
            }
            Ok(())
        }
        expr => {
            match c.calculate(expr, now) {
                Ok(r) => { println!("{}", r); Ok(()) }
                Err(e) => Err(format!("{}", e)),
            }
        }
    } })();

    match result {
        Ok(()) => { if let Err(e)=c.save(&db) { eprintln!("sigmaos-calc: warn: save: {e}"); } }
        Err(e) => { eprintln!("sigmaos-calc: {}", e); exit(1); }
    }
}
