use crate::vm::opcode::Value;

pub fn get_type(value: &Value) -> String {
    match value {
        Value::String(_) => "string".to_string(),
        Value::Integer(_) => "integer".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::None => "none".to_string(),
    }
}

pub fn unsupported_operation(a: &Value, b: &Value, o: &str) -> String {
    format!(
        "Unsupported operation [{} {} {}]",
        get_type(a),
        o,
        get_type(b)
    )
}

pub fn unsupported_conversion(a: &Value, b: u8) -> String {
    format!(
        "Unsupported operation [{} as {}]",
        get_type(a),
        match b {
            0 => "Int",
            1 => "Float",
            2 => "String",
            3 => "Array",
            4 => "Bool",
            5 => "None",
            _ => panic!(""),
        }
    )
}
