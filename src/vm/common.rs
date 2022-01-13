use crate::vm::opcode::Value;

pub fn get_type(value: &Value) -> String {
    match value {
        Value::String(_) => "string".to_string(),
        Value::Integer(_) => "integer".to_string(),
        Value::Float(_) => "float".to_string(),
        //Value::List(_) => "list".to_string(),
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

// custom function for epxonentiating i64
pub fn exponentiate(num: i64, exp: i64) -> Option<i64> {
    let mut ret: i64 = num;

    for _ in 1..exp {
        let iret = ret.checked_mul(num);

        match iret {
            Option::Some(iret) => ret = iret,
            Option::None => return Option::None,
        }
    }

    return Option::Some(ret);
}
