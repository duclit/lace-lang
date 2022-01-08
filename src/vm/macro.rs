use crate::vm::opcode;

fn to_string(value: opcode::Value) -> String {
    match value {
        opcode::Value::String(str) => str,
        opcode::Value::Integer(int) => int.to_string(),
        opcode::Value::Float(float) => float.to_string(),
        opcode::Value::List(list) => {
            let mut string = "[".to_string();

            for value in list {
                string.push_str(&to_string(value));
            }

            string.push_str("]");
            string
        }
        opcode::Value::Bool(bool) => bool.to_string(),
        opcode::Value::None => String::from("none")
    }
}

pub fn writeln(arguments: Vec<opcode::Value>) -> opcode::Value {
    let mut string = String::new();

    for argument in arguments {
        string.push_str(&to_string(argument));
    }

    println!("{}", &string);
    return opcode::Value::None
}
