mod codegen;
mod error;
mod optimizer;
mod parser;
mod scanner;

pub mod pipeline;

fn main() {
    pipeline::lace_pipeline_init(
        "
        fn some(argument: number, argument?: number, mut argument?: number) {

        }
        ",
    );
}
