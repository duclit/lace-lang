mod compiler;
mod error;
mod lexer;

use std::fs;
use std::{env, process::exit};

use crate::compiler::parse;
use crate::lexer::{tokenize, Token};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        0 | 1 => {
            println!("No path provided. Nothing to do 😴");
            exit(0)
        }
        2 => {
            let path = &args[1];
            let data = fs::read_to_string(path);

            match data {
                Result::Ok(data) => {
                    let tokens = tokenize(data.clone());

                    println!("{:#?}", parse(tokens, data))
                }
                Result::Err(_) => {
                    println!("😐 Unable to read file");
                    exit(0);
                }
            }
        }
        _ => {
            println!("🔍 Expected only one argument (A file path).");
        }
    }
}
