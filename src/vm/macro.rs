use crate::vm::opcode;
use std::process::exit;

fn to_string(value: opcode::Value) -> String {
    match value {
        opcode::Value::String(str) => str,
        opcode::Value::Integer(int) => int.to_string(),
        opcode::Value::Float(float) => float.to_string(),
        opcode::Value::Array(list) => {
            let mut string = "[".to_string();
            let listlen = list.len();

            for (i, value) in list.into_iter().enumerate() {
                string.push_str(&to_string(value));

                if i + 1 < listlen {
                    string.push_str(", ");
                }
            }

            string.push(']');
            string
        }
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
    opcode::Value::None
}

pub fn lace_exit(_: Vec<opcode::Value>) -> opcode::Value {
    exit(0);
}
