mod codegen;
mod error;
mod optimizer;
mod parser;
mod scanner;
mod typecheck;

pub mod pipeline;

fn main() {
    let ast = pipeline::lace_pipeline_init(
        "
        let something: string = \"string\" * 5
        let happiness: number = something + \" something else\" 
        ",
    );

    let mut typechecker = typecheck::Typechecker::new();
    typechecker.check(ast);
}
