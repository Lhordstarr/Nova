use std::fs;

use crate::errors::NovaError;
use crate::interpreter;
use crate::lexer;
use crate::parser;
use crate::semantic;

pub fn run() -> Result<(), NovaError> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        None | Some("help") => {
            print_help();
            Ok(())
        }
        Some("version") => {
            println!("NOVA 0.1.3");
            Ok(())
        }
        Some("run") => {
            let path = args.get(2).ok_or_else(|| {
                NovaError::new(crate::errors::ErrorKind::Cli, "missing file path")
            })?;
            let source = fs::read_to_string(path)?;
            let tokens = lexer::tokenize(&source)?;
            let program = parser::parse(&tokens)?;
            semantic::analyze(&program)?;
            interpreter::execute(&program)?;
            Ok(())
        }
        Some("check") => {
            let path = args.get(2).ok_or_else(|| {
                NovaError::new(crate::errors::ErrorKind::Cli, "missing file path")
            })?;
            let source = fs::read_to_string(path)?;
            let tokens = lexer::tokenize(&source)?;
            let program = parser::parse(&tokens)?;
            semantic::analyze(&program)?;
            Ok(())
        }
        Some(command) => Err(NovaError::new(
            crate::errors::ErrorKind::Cli,
            format!("unknown command '{}'", command),
        )),
    }
}

fn print_help() {
    println!("NOVA Programming Language");
    println!();
    println!("Usage:");
    println!("  nova run <file.nova>     Run a NOVA program");
    println!("  nova check <file.nova>   Check a NOVA program");
    println!("  nova version             Show the NOVA version");
    println!("  nova help                Show this help message");
}
