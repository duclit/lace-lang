use lacec::common::*;
use std::process::exit;

fn to_string(value: Value) -> String {
    match value {
        Value::String(str) => str,
        Value::Number(int) => int.to_string(),
        Value::Byte(int) => int.to_string(),
        Value::Float(float) => float.to_string(),
        Value::Array(list) => {
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
        Value::True => String::from("true"),
        Value::False => String::from("false"),
        Value::None => String::from("none"),
        Value::Function { .. } => String::from("<fn>"),
        _ => panic!(),
    }
}

pub fn lace_writeln(arguments: Vec<Value>) -> Value {
    let mut string = String::new();

    for argument in arguments {
        string.push_str(&to_string(argument));
        string.push(' ');
    }

    println!("{}", &string);
    Value::None
}

pub fn lace_exit(_: Vec<Value>) -> Value {
    exit(0);
}
