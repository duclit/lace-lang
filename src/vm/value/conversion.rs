use super::exception::*;
use crate::error::Data;
use crate::vm::opcode::Value;

fn to_string(value: Value) -> String {
    match value {
        Value::String(str) => str,
        Value::Integer(int) => int.to_string(),
        Value::Float(float) => float.to_string(),
        Value::Array(list) => {
            let mut string = "[".to_string();

            for value in list {
                string.push_str(&to_string(value));
            }

            string.push(']');
            string
        }
        Value::Bool(bool) => bool.to_string(),
        Value::None => String::from("none"),
    }
}

#[inline(always)]
pub fn convert(a: &Value, b: u8, context: Data) -> Value {
    match a {
        Value::Integer(int) => match b {
            2 => return Value::String(format!("{}", int)),
            1 => Value::Float(*int as f32),
            3 => Value::Array(vec![Value::Integer(*int)]),
            4 => Value::Bool(*int > 0),
            0 => Value::Integer(*int),
            _ => context.raise(unsupported_conversion(a, b)),
        },
        Value::Float(float) => match b {
            2 => return Value::String(format!("{}", float)),
            1 => Value::Float(*float as f32),
            3 => Value::Array(vec![Value::Float(*float)]),
            4 => Value::Bool(*float > 0.0),
            0 => Value::Integer(*float as i32),
            _ => context.raise(unsupported_conversion(a, b)),
        },
        Value::String(str) => match b {
            2 => Value::String(str.to_string()),
            1 => Value::Float(match str.parse::<f32>() {
                Result::Ok(float) => float,
                Result::Err(_) => context.raise("Cannot convert string to float.".to_string()),
            }),
            3 => Value::Array(vec![Value::String(str.to_string())]),
            4 => Value::Bool(!str.is_empty()),
            0 => Value::Integer(match str.parse::<i32>() {
                Result::Ok(int) => int,
                Result::Err(_) => context.raise("Cannot convert string to float.".to_string()),
            }),
            _ => context.raise(unsupported_conversion(a, b)),
        },
        Value::Bool(boolean) => match b {
            2 => return Value::String(format!("{}", boolean)),
            1 => Value::Float(if *boolean { 1.0 } else { 0.0 }),
            3 => Value::Array(vec![Value::Bool(*boolean)]),
            4 => Value::Bool(*boolean),
            0 => Value::Integer(if *boolean { 1 } else { 0 }),
            _ => context.raise(unsupported_conversion(a, b)),
        },
        Value::Array(_) => match b {
            2 => Value::String(to_string(a.clone())),
            _ => context.raise(unsupported_conversion(a, b)),
        },
        _ => context.raise(unsupported_conversion(a, b)),
    }
}
