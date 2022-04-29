use compiler;
use std::{env, process::exit};
use colored::*;

fn error(err: &str) -> ! {
    println!("{}: {}", "Error".red(), err);
    exit(0)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        error("Expected command name.")
    }

    match args[1].as_str() {
        "build" => {
            if args.len() == 2 {
                error("Expected source file.")
            }

            let source = &args[2];

            let contents = std::fs::read_to_string(source)
                .expect("Something went wrong reading the file.");
    

            let ast = compiler::pipeline::lace_pipeline_init(&contents);
            let mut typechecker = compiler::typecheck::Typechecker::new();
            typechecker.check(ast);
        },
        "run" => todo!(),
        _ => error("Command not found.")
    }
}
