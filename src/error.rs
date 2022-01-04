use crate::lexer::Token;
use std::process::exit;

// context for an error, passed into the raise function
pub struct Context {
    pub line: String,
    pub idx: usize,
    pub pointer: Option<usize>,
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
    println!("{} | {}", line_idx, ctx.line);

    match ctx.pointer {
        Option::None => println!("{} |", empty),
        Option::Some(ptr) => println!("{} | {}^", empty, " ".repeat(ptr)),
    }

    println!("Error: {}", err);
    exit(0);
}