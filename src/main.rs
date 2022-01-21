mod compiler;
mod error;
mod io;
mod lexer;
mod parser;
mod vm;

use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use std::{env, process::exit};

use crate::io as lace_io;

fn error(msg: &str) -> ! {
    println!("{}", msg);
    exit(0)
}

fn is_of_ext(ext: &str, path: &str) -> bool {
    path.ends_with(ext)
}

fn compile(path: &str) -> String {
    let data = fs::read_to_string(path);
    let filename = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();

    match data {
        Ok(data) => {
            if !is_of_ext(".lc", &filename) {
                error("Lace files must end with .lc")
            }

            let mut tokenizer = lexer::Tokenizer::new(data.to_string());
            tokenizer.tokenize();
            let tokens = tokenizer.tokens;

            let mut main = parser::Function {
                name: "<main>".to_string(),
                args: vec![],
                body: vec![],
                local_functions: HashMap::new(),
                file: "main.lc".to_string()
            };

            let start = Instant::now();
            let mut parser_: parser::Parser = parser::Parser::new(
                tokens,
                data.split('\n')
                    .map(str::to_string)
                    .collect::<Vec<String>>(),
            );
            parser_.parse(&mut main);
            println!("{:?}", main.body);
            let code = compiler::compile(main);

            println!("{:?}", code);

            let object_file_name = format!("{}.o", &filename[0..filename.len() - 3]);

            fs::write(object_file_name.to_string(), lace_io::serialize(code)).unwrap();
            println!("Compiled succesfully in {:.2?}", start.elapsed());

            object_file_name
        }
        Err(_) => {
            error("😐 Unable to read file");
        }
    }
}

fn run(path: &str) {
    let filename = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();

    if !is_of_ext(".o", &filename) {
        error("Compiled lace files must end with .o")
    }

    let bytes = fs::read(path);

    match bytes {
        Ok(bytes) => {
            let start = Instant::now();
            let main = lace_io::deserialize(bytes);
            vm::run(main, HashMap::new(), Option::None);
            println!("Execution took {:.2?}", start.elapsed());
        }
        Err(_) => error("😐 Unable to read file"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 2 {
        error("🤔 Expected argument.")
    }

    match args[1].as_str() {
        "build" => {
            compile(&args[2]);
        }
        "run" => run(&args[2]),
        _ => error(format!("🔎 Command '{}' not found.", args[1]).as_str()),
    }
}
