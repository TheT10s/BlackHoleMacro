use std::env;
use std::fs;
use std::process;

use blackholemacro::interpreter::{Interpreter, LogEvent};
use blackholemacro::parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: bhc <script.singular>");
        eprintln!("");
        eprintln!("Run a SingularityScript file headlessly.");
        eprintln!("Press Ctrl+C to stop.");
        process::exit(1);
    }

    let path = &args[1];

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    let script = match parse(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

    eprintln!("[bhc] Running script: {}", script.name);
    eprintln!("---");

    let mut interp = Interpreter::new();

    match interp.run(&script) {
        Ok(()) => {
            // Print log
            for event in &interp.log {
                match event {
                    LogEvent::Info(msg) => eprintln!("  {}", msg),
                    LogEvent::VariableChanged(name, val) => {
                        eprintln!("  {} = {}", name, val);
                    }
                    LogEvent::ScriptStarted(name) => {
                        eprintln!("[bhc] Script started: {}", name);
                    }
                    LogEvent::ScriptFinished(name, ok) => {
                        if *ok {
                            eprintln!("[bhc] Script finished: {}", name);
                        } else {
                            eprintln!("[bhc] Script failed: {}", name);
                        }
                    }
                    _ => {}
                }
            }
            eprintln!("---");
            eprintln!("[bhc] Done.");
        }
        Err(e) => {
            for event in &interp.log {
                if let LogEvent::Info(msg) = event {
                    eprintln!("  {}", msg);
                }
            }
            eprintln!("[bhc] Error: {}", e);
            process::exit(1);
        }
    }
}
