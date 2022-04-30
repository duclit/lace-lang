use colored::*;
use compiler;
use hlvm;
use std::{env, process::exit, time::Instant};
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

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

            let contents =
                std::fs::read_to_string(source).expect("Something went wrong reading the file.");

            let ast = compiler::pipeline::lace_pipeline_init(&contents);
            let mut typechecker = compiler::typecheck::Typechecker::new();
            typechecker.check(ast.clone());

            let hir_instructions = compiler::codegen::compile(ast);
            let lir_instructions = hlvm::hir::from_hir(hir_instructions);

            println!("{:?}", lir_instructions);

            std::fs::write("./main.o", bincode::serialize(&lir_instructions).unwrap())
                .expect("Unable to write file");
        }
        "run" => {
            if args.len() == 2 {
                error("Expected source file.")
            }

            let source = &args[2];

            let f = File::open(source)
                .expect("Could not open file");
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::new();

            reader.read_to_end(&mut buffer)
                .expect("Something went wrong while reading the file");

            let instructions = bincode::deserialize::<Vec<hlvm::lir::HlvmInstruction>>(&buffer)
                .expect("Unable to deserialize instructions");

            let mut executor = hlvm::vm::HighLevelVirtualMachine::new(Some(1));

            let start = Instant::now();
            executor.execute(&instructions).expect("An error occured");
            let end = start.elapsed();

            println!("{:#?}", executor.call_stack);
            println!("Execution took {:.2?}", end);
        }
        _ => error("Command not found."),
    }
}
