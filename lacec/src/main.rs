mod lace;

use lace::lacec::Compiler;
use lacec::common::*;
use lacec::parser::Parser;
use lacec::scanner::Scanner;
use lacec::scanner::Token;

use colored::*;
use std::{process::exit, time::Instant};

pub fn error(error: String) -> ! {
    println!("{}: {}", "lacec_error".red(), error);
    exit(0);
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    match arguments.len() {
        0 => println!("lacec v0.1.0"),
        1 | 2 => {
            let filename = &arguments[0];

            let text = match std::fs::read_to_string(filename) {
                Ok(str) => str,
                Err(_) => error(format!(
                    "Couldn't find file named `{}` in this folder.",
                    filename.magenta()
                )),
            };

            if !filename.ends_with(".lc") {
                error("Lace files must end with .lc".to_string())
            }

            let time = Instant::now();

            let mut scanner = Scanner::new(&text);
            let tokens = scanner.scan();

            // debugging
            println!("{:?}", tokens.clone().collect::<Vec<Token>>());

            let mut parser = Parser::new(tokens, text);
            parser.parse();

            // debugging
            println!("{:?}", parser.ast.clone());

            let mut compiler = Compiler::new(parser.ast);
            let mut code: Vec<Instruction> = vec![];
            compiler.compile(&mut code);

            // debugging
            println!("{:?}\n{:?}", code.clone(), compiler.constants.0.clone());

            let bytes = bincode::serialize(&(compiler.constants.0, code)).unwrap();
            let object_file_name = format!("{}.o", &filename[0..filename.len() - 3]);
            std::fs::write(object_file_name, bytes).unwrap();

            println!(
                "Compiled sucessfully in {}.",
                format!("{:.2?}", time.elapsed()).magenta()
            )
        }
        _ => error("Enter a file path to compile.".to_string()),
    }
}
