mod vm;
mod macros;

use vm::*;

use lacec::common::*;
use bincode;
use colored::*;

use std::process::exit;
use std::time::Instant;

pub fn error(error: String) -> ! {
    println!("{}: {}", "lace_error".red(), error);
    exit(0);
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    match arguments.len() {
        0 => println!("lace v0.1.0"),
        1 => {
            let filename = &arguments[0];

            if !filename.ends_with(".o") {
                error("Lace bytecode files must end with .o".to_string())
            }

            let filename = std::path::Path::new(filename)
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();

            let bytes = std::fs::read(filename.to_string());

            match bytes {
                Ok(bytes) => {
                    let time = Instant::now();

                    let (constants, instructions): (Vec<Value>, Vec<Instruction>) =
                        bincode::deserialize(&bytes).unwrap();

                    let mut vm = VirtualMachine::new(constants);
                    vm.run(instructions);

                    println!(
                        "debug: execution took {}.",
                        format!("{:.2?}", time.elapsed()).magenta()
                    )
                }
                Err(_) => error(format!(
                    "Could not find file named '{}' in this folder.",
                    arguments[0]
                )),
            }
        }
        _ => error("Enter a file path to execute.".to_string()),
    }
}
