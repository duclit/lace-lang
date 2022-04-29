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
        let happiness: string = something + \" something else\"

        if something == happiness {
            print(\"yes\")
        } else if something != happiness {
            print(\"no\")
        } else if something == happiness {
            print(\"damn\")
        } else {
            print(\"ok\")
        }
        ",
    );

    let mut typechecker = typecheck::Typechecker::new();
    typechecker.check(ast);
}
