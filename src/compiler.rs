use std::{process::exit};
use crate::vm::opcode;

fn to_string(value: opcode::Value) -> String {
    match value {
        opcode::Value::String(str) => str,
        opcode::Value::Integer(int) => int.to_string(),
        opcode::Value::Float(float) => float.to_string(),
        opcode::Value::Bool(bool) => bool.to_string(),
        opcode::Value::None => String::from("none"),
    }
}

pub fn lace_writeln(arguments: Vec<opcode::Value>) -> opcode::Value {
    let mut string = String::new();

    for argument in arguments {
        string.push_str(&to_string(argument));
        string.push(' ');
    }

    println!("{}", &string);
    return opcode::Value::None;
}

pub fn lace_exit(_: Vec<opcode::Value>) -> opcode::Value {
    exit(0);
}
