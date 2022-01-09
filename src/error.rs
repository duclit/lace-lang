use crate::lexer::Token;
use std::process::exit;

// context for an error, passed into the raise function
pub struct Context {
    pub line: String,
    pub idx: usize,
    pub pointer: Option<usize>,
}

impl Context {
    pub fn new(idx: usize, source: &Vec<String>, pointer: Option<usize>) -> Context {
        Context {
            line: source[idx].to_string(),
            idx,
            pointer,
        }
    }
}

// base context, contains data required to build Context
pub struct BaseContext {
    pub tokens: Vec<Token>,
    pub base: usize,
    pub source: Vec<String>,
}

pub fn raise(err: &str, ctx: Context) -> ! {
    let line_idx = ctx.idx + 1;
    let empty = " ".repeat(format!("{}", line_idx).len());

    println!("{} |", empty);
    println!("{} | {}", line_idx, ctx.line.trim_start().to_string());

    match ctx.pointer {
        Option::None => println!("{} |", empty),
        Option::Some(ptr) => println!("{} | {}^", empty, " ".repeat(ptr)),
    }

    println!("Error: {}", err);
    exit(0);
}

// raised when something goes wrong unexpectedly in the pipeline
pub fn raise_internal(code: &str) -> ! {
    println!("An unexpected error has occured (#{})", code);
    exit(0);
}
