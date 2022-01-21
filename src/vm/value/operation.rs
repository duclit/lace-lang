use super::exception::*;
use crate::error::Data;
use crate::vm::opcode::{OpCode, Value};

// custom function for epxonentiating i64
pub fn exponentiate(num: i32, exp: i32) -> Option<i32> {
    let mut ret: i32 = num;

    for _ in 1..exp {
        let iret = ret.checked_mul(num);

        match iret {
            Option::Some(iret) => ret = iret,
            Option::None => return Option::None,
        }
    }

    Option::Some(ret)
}

#[inline(always)]
pub fn add(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::String(av) => match b {
            Value::String(bv) => return Value::String(format!("{}{}", av, bv)),
            _ => context.raise(unsupported_operation(a, b, "+")),
        },
        Value::Integer(av) => match &b {
            Value::Integer(bv) => match av.checked_add(*bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer addition resulted in overflow".to_string()),
            },
            Value::Float(bv) => Value::Float(*av as f32 + bv),
            _ => context.raise(unsupported_operation(a, b, "+")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Float(av + bv),
            Value::Integer(bv) => Value::Float(av + *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "+")),
        },
        _ => context.raise(unsupported_operation(a, b, "+")),
    }
}

#[inline(always)]
pub fn sub(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(_av) => match &b {
            Value::Integer(bv) => match _av.checked_sub(*bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer subtraction resulted in overflow".to_string()),
            },
            Value::Float(bv) => Value::Float(*_av as f32 - bv),
            _ => context.raise(unsupported_operation(a, b, "-")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Float(av - bv),
            Value::Integer(bv) => Value::Float(av - *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "-")),
        },
        _ => context.raise(unsupported_operation(a, b, "-")),
    }
}

#[inline(always)]
pub fn mul(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::String(av) => match b {
            Value::Integer(bv) => Value::String(av.repeat(*bv as usize)),
            _ => context.raise(unsupported_operation(a, b, "*")),
        },
        Value::Integer(av) => match &b {
            Value::Integer(bv) => match av.checked_mul(*bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer multiplication resulted in overflow".to_string()),
            },
            Value::Float(bv) => Value::Float(*av as f32 * bv),
            _ => context.raise(unsupported_operation(a, b, "*")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Float(av * bv),
            Value::Integer(bv) => Value::Float(av * *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "*")),
        },
        _ => context.raise(unsupported_operation(a, b, "*")),
    }
}

#[inline(always)]
pub fn div(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => match av.checked_div(*bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer division resulted in overflow".to_string()),
            },
            Value::Float(bv) => Value::Float(*av as f32 / bv),
            _ => context.raise(unsupported_operation(a, b, "/")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Float(av / bv),
            Value::Integer(bv) => Value::Float(av / *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "/")),
        },
        _ => context.raise(unsupported_operation(a, b, "/")),
    }
}

#[inline(always)]
pub fn rem(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => match av.checked_rem(*bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer remainder resulted in overflow".to_string()),
            },
            Value::Float(bv) => Value::Float(*av as f32 % bv),
            _ => context.raise(unsupported_operation(a, b, "%")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Float(av % bv),
            Value::Integer(bv) => Value::Float(av % *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "%")),
        },
        _ => context.raise(unsupported_operation(a, b, "%")),
    }
}

//#[inline(always)]
//pub fn lshift(a: &Value, b: &Value, context: Data) -> Value {
//    match a {
//        Value::Integer(av) => match &b {
//            Value::Integer(bv) => match av.checked_shl(*bv) {
//                Option::Some(int) => return Value::Integer(int),
//                None => context.raise("Integer remainder resulted in overflow".to_string()),
//            },
//            _ => context.raise(unsupported_operation(a, b, "%")),
//        },
//        _ => context.raise(unsupported_operation(a, b, "%")),
//    }
//}
//
//#[inline(always)]
//pub fn rshift(a: &Value, b: &Value, context: Data) -> Value {
//    match a {
//        Value::Integer(av) => match &b {
//            Value::Integer(bv) => match av.checked_rem(*bv) {
//                Option::Some(int) => return Value::Integer(int),
//                None => context.raise("Integer remainder resulted in overflow".to_string()),
//            },
//            _ => context.raise(unsupported_operation(a, b, "%")),
//        },
//        _ => context.raise(unsupported_operation(a, b, "%")),
//    }
//}

#[inline(always)]
pub fn pow(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => match exponentiate(*av, *bv) {
                Option::Some(int) => Value::Integer(int),
                None => context.raise("Integer exponentiation resulted in overflow".to_string()),
            },
            _ => context.raise(unsupported_operation(a, b, "%")),
        },
        _ => context.raise(unsupported_operation(a, b, "%")),
    }
}

#[inline(always)]
pub fn eq(a: &Value, b: &Value, _context: Data) -> Value {
    Value::Bool(a.clone() == b.clone())
}

#[inline(always)]
pub fn neq(a: &Value, b: &Value, _context: Data) -> Value {
    Value::Bool(a.clone() != b.clone())
}

#[inline(always)]
pub fn more(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => Value::Bool(av > bv),
            Value::Float(bv) => Value::Bool(*av as f32 > *bv),
            _ => context.raise(unsupported_operation(a, b, ">")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Bool(av > bv),
            Value::Integer(bv) => Value::Bool(*av > *bv as f32),
            _ => context.raise(unsupported_operation(a, b, ">")),
        },
        _ => context.raise(unsupported_operation(a, b, ">")),
    }
}

#[inline(always)]
pub fn less(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => Value::Bool(av < bv),
            Value::Float(bv) => Value::Bool((*av as f32) < *bv),
            _ => context.raise(unsupported_operation(a, b, "<")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Bool(av < bv),
            Value::Integer(bv) => Value::Bool(*av < *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "<")),
        },
        _ => context.raise(unsupported_operation(a, b, "<")),
    }
}

#[inline(always)]
pub fn more_than(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => Value::Bool(av >= bv),
            Value::Float(bv) => Value::Bool(*av as f32 >= *bv),
            _ => context.raise(unsupported_operation(a, b, ">=")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Bool(av >= bv),
            Value::Integer(bv) => Value::Bool(*av >= *bv as f32),
            _ => context.raise(unsupported_operation(a, b, ">=")),
        },
        _ => context.raise(unsupported_operation(a, b, ">=")),
    }
}

#[inline(always)]
pub fn less_than(a: &Value, b: &Value, context: Data) -> Value {
    match a {
        Value::Integer(av) => match &b {
            Value::Integer(bv) => Value::Bool(av <= bv),
            Value::Float(bv) => Value::Bool((*av as f32) <= *bv),
            _ => context.raise(unsupported_operation(a, b, "<=")),
        },
        Value::Float(av) => match &b {
            Value::Float(bv) => Value::Bool(av <= bv),
            Value::Integer(bv) => Value::Bool(*av <= *bv as f32),
            _ => context.raise(unsupported_operation(a, b, "<=")),
        },
        _ => context.raise(unsupported_operation(a, b, "<=")),
    }
}

pub fn operate(a: &Value, b: &Value, code: OpCode, context: Data) -> Value {
    match code {
        OpCode::Add => add(a, b, context),
        OpCode::Sub => sub(a, b, context),
        OpCode::Mul => mul(a, b, context),
        OpCode::Div => div(a, b, context),
        OpCode::Mod => rem(a, b, context),
        OpCode::Pow => pow(a, b, context),
        OpCode::Equal => eq(a, b, context),
        OpCode::NotEqual => neq(a, b, context),
        OpCode::More => more(a, b, context),
        OpCode::Less => less(a, b, context),
        OpCode::MoreOrEqual => more_than(a, b, context),
        OpCode::LessOrEqual => less_than(a, b, context),
        _ => panic!(""),
    }
}
