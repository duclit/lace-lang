use std::collections::HashMap;

use serde::{Deserialize, Serialize};

type ValueIdx = usize;
type NameIdx = usize;
type Length = usize;
type TypeIdx = u8;

#[repr(u8)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum OpCode {
    LoadConst(ValueIdx),
    LoadVariable(NameIdx),
    AssignVar(NameIdx),
    CallMacro(NameIdx, Length),
    CallFunction(NameIdx, Length),
    LoadBuiltinValue(ValueIdx),

    FormatString,
    BuildList(Length),
    ConvertTo(TypeIdx),

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    LShift,
    RShift,
    Equal,
    NotEqual,
    More,
    Less,
    MoreOrEqual,
    LessOrEqual,

    Return,
    ReturnNone,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool),
    Array(Vec<Value>),
    None,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    String,
    Integer,
    Float,
    Bool,
    None,
    Array,
}

impl Value {
    fn _is_truthy(&self) -> bool {
        match self {
            Value::String(str) => str.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Integer(int) => int < &1,
            Value::Float(float) => float < &1.0,
            Value::Bool(bool) => *bool,
            Value::None => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CodeObject {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub parameters: Vec<(String, bool)>,
    pub functions: HashMap<String, CodeObject>,
    pub file: String,
}

impl CodeObject {
    pub fn add_code(&mut self, code: OpCode) {
        self.code.push(code);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        match self.constants.iter().position(|x| x == &value) {
            Option::Some(index) => index,
            Option::None => {
                self.constants.push(value);
                self.constants.len() - 1
            }
        }
    }
}
