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

pub fn raise_rng(err: &str, ctx: Context, len: usize) -> ! {
    let line_idx = ctx.idx + 1;
    let empty = " ".repeat(format!("{}", line_idx).len());

    println!("{} |", empty);
    println!("{} | {}", line_idx, ctx.line.trim_start().to_string());

    match ctx.pointer {
        Option::None => println!("{} |", empty),
        Option::Some(ptr) => println!("{} | {}{}", empty, " ".repeat(ptr), "^".repeat(len)),
    }

    println!("Error: {}", err);
    exit(0);
}

pub struct Data {
    pub error: String,
}

impl Data {
    pub fn new(line: usize, filename: String) -> Data {
        Data {
            error: format!("{}:{}", filename, line),
        }
    }

    pub fn raise(&self, error: String) -> ! {
        println!("{} {}", self.error, error);
        exit(0);
    }
}
