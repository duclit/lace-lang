mod compiler;
mod error;
mod io;
mod lexer;
mod parser;
mod vm;

use std::collections::HashMap;
use std::fs;
use std::{env, process::exit};

use crate::io as lace_io;

fn error(msg: &str) -> ! {
    println!("{}", msg);
    exit(0)
}

fn is_of_ext(ext: &str, path: &String) -> bool {
    path.ends_with(ext)
}

fn compile(path: &String) {
    let data = fs::read_to_string(path);
    let filename = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();

    match data {
        Result::Ok(data) => {
            if !is_of_ext(".lc", &filename) {
                error("Lace files must end with .lc")
            }

            let tokens = lexer::tokenize(data.clone());
            let mut main = parser::Function {
                name: "<main>".to_string(),
                args: vec![],
                body: vec![],
                local_functions: HashMap::new()
            };

            parser::parse(tokens, data, &mut main);

            println!("{:#?}", main);

            let code = compiler::compile(main);

            let object_file_name = format!("{}.o", &filename[0..filename.len() - 3]);

            fs::write(object_file_name, lace_io::serialize(code.clone())).unwrap();
        }
        Result::Err(_) => {
            println!("😐 Unable to read file");
            exit(0);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 2 {
        error("🤔 Expected argument.")
    }

    match args[1].as_str() {
        "build" => compile(&args[2]),
        "run" => {}
        _ => error(format!("🔎 Command '{}' not found.", args[1]).as_str()),
    }
}
