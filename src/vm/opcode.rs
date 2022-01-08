use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum OpCode {
    LoadConst,
    LoadVariable,
    AssignVar,
    CallMacro,

    // followed by number ranging from 0-2 (none, true & false)
    LoadBuiltinValue,

    FormatString,
    BuildList,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    LShift,
    RShift,
}

pub trait Extract<T> {
    fn extract(self) -> Option<T>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Code {
    OpCode(OpCode),
    Number(usize),
}

impl Extract<usize> for Code {
    fn extract(self) -> Option<usize> {
        if let Code::Number(num) = self {
            Option::Some(num)
        } else {
            Option::None
        }
    }
}

impl Extract<OpCode> for Code {
    fn extract(self) -> Option<OpCode> {
        if let Code::OpCode(opc) = self {
            Option::Some(opc)
        } else {
            Option::None
        }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    List(Vec<Value>),
    Bool(bool),
    None
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CodeObject {
    pub code: Vec<Code>,
    pub constants: Vec<Value>,
}

impl CodeObject {
    pub fn add_code(&mut self, code: Code) {
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
