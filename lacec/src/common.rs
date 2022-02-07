use crate::scanner::{ExtractValue, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Typeof,
    Negate,
    LogicalNot,
}

/* Represents a parameter in a function definition, and is used in the parser/compiler. */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FnParam {
    pub name: String,
    pub mutable: bool,
    pub annotation: Type,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Type {
    Number,
    Float,
    String,
    Byte,
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Bool,
    None,
    Dynamic,
}

// Helper function to convert a rust bool to a lace value
fn bool2value(rbool: bool) -> Value {
    if rbool {
        Value::True
    } else {
        Value::False
    }
}

/* Represents a parser node */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Node {
    Value(Value),
    Unary(Box<Node>, UnaryOp),

    Return(Box<Node>),
    ReturnNone,

    Function {
        name: String,
        params: Vec<FnParam>,
        function: Vec<Node>,
        coroutine: bool,
        return_annotation: Type,
        public: bool,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Node>,
    },
    PrimitiveFunctionCall {
        name: String,
        arguments: Vec<Node>,
    },

    WhileLoop {
        condition: Box<Node>,
        body: Vec<Node>,
    },

    Binary {
        left: Box<Node>,
        right: Box<Node>,
        operator: Token,
    },
    VariableDeclr {
        name: String,
        value: Box<Node>,
        public: bool,
        annotation: Type,
    },
    VariableAssignment {
        name: String,
        value: Box<Node>,
    },
}

/* Represents a value and is used in the parser, compiler and the VM. */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    FormattedString(String),
    Byte(i8),
    Array(Vec<Value>),
    Function {
        code: Vec<Instruction>,
        parameters: Vec<String>,
        coroutine: bool,
    },

    True,
    False,
    None,

    // Values that are not exposed to the people
    NodeArray(Vec<Node>),
    Identifier(String),
    NodeFunction(Vec<Node>),
}

impl Value {
    pub fn istruthy(self) -> bool {
        match self {
            Value::String(string) => string.len() > 0,
            Value::FormattedString(string) => string.len() > 0,
            Value::Function { code, .. } => code.len() > 0,
            Value::Byte(byte) => byte > 0,
            Value::Number(int) => int > 0,
            Value::Float(float) => float > 0.0,
            Value::True => true,
            Value::False => false,
            Value::Array(list) => list.len() > 0,
            Value::None => false,
            _ => panic!("istruthy on private variant"),
        }
    }
}

/* Utility function to extract the value of an Identifier. */
impl ExtractValue<String> for Value {
    fn extract(&self) -> String {
        match self {
            Value::Identifier(iden) => iden.to_string(),
            _ => panic!("Could not extract identifier."),
        }
    }
}

pub trait Operations {
    fn add(self, other: Value) -> Value;
    fn sub(self, other: Value) -> Value;
    fn mul(self, other: Value) -> Value;
    fn div(self, other: Value) -> Value;
    fn rem(self, other: Value) -> Value;
    fn pow(self, other: Value) -> Value;
    fn lsh(self, other: Value) -> Value;
    fn rsh(self, other: Value) -> Value;
    fn eq(self, other: Value) -> Value;
    fn ne(self, other: Value) -> Value;
    fn gt(self, other: Value) -> Value;
    fn st(self, other: Value) -> Value;
    fn ge(self, other: Value) -> Value;
    fn se(self, other: Value) -> Value;
    fn not(self) -> Value;
    fn negate(self) -> Value;
    fn tpyeof(self) -> Value;
}

impl Operations for Value {
    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Float(a), Value::Number(b)) => Value::Float(a + b as f32),
            (Value::Number(a), Value::Float(b)) => Value::Float(a as f32 + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::String(mut str), Value::String(str2)) => {
                str.push_str(&str2);
                Value::String(str)
            }

            _ => panic!(),
        }
    }

    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (Value::Float(a), Value::Number(b)) => Value::Float(a - b as f32),
            (Value::Number(a), Value::Float(b)) => Value::Float(a as f32 - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),

            _ => panic!(),
        }
    }

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (Value::Float(a), Value::Number(b)) => Value::Float(a * b as f32),
            (Value::Number(a), Value::Float(b)) => Value::Float(a as f32 * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::String(str), Value::Number(num)) => Value::String(str.repeat(num as usize)),

            _ => panic!(),
        }
    }

    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            (Value::Float(a), Value::Number(b)) => Value::Float(a / b as f32),
            (Value::Number(a), Value::Float(b)) => Value::Float(a as f32 / b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),

            _ => panic!(),
        }
    }

    fn rem(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
            (Value::Float(a), Value::Number(b)) => Value::Float(a % b as f32),
            (Value::Number(a), Value::Float(b)) => Value::Float(a as f32 % b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a % b),

            _ => panic!(),
        }
    }

    fn pow(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a.pow(b as u32)),
            (Value::Float(a), Value::Number(b)) => Value::Float(a.powf(b as f32)),
            (Value::Number(a), Value::Float(b)) => Value::Float((a as f32).powf(b)),
            (Value::Float(a), Value::Float(b)) => Value::Float(a.powf(b)),

            _ => panic!(),
        }
    }

    fn eq(self, other: Value) -> Value {
        if self == other {
            Value::True
        } else {
            Value::False
        }
    }

    fn ne(self, other: Value) -> Value {
        if self != other {
            Value::True
        } else {
            Value::False
        }
    }

    fn gt(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => bool2value(a > b),
            (Value::Float(a), Value::Number(b)) => bool2value(a > b as f32),
            (Value::Number(a), Value::Float(b)) => bool2value(a as f32 > b),
            (Value::Float(a), Value::Float(b)) => bool2value(a > b),

            _ => panic!(),
        }
    }

    fn st(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => bool2value(a < b),
            (Value::Float(a), Value::Number(b)) => bool2value(a < b as f32),
            (Value::Number(a), Value::Float(b)) => bool2value((a as f32) < b),
            (Value::Float(a), Value::Float(b)) => bool2value(a < b),

            _ => panic!(),
        }
    }

    fn ge(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => bool2value(a >= b),
            (Value::Float(a), Value::Number(b)) => bool2value(a >= b as f32),
            (Value::Number(a), Value::Float(b)) => bool2value((a as f32) >= b),
            (Value::Float(a), Value::Float(b)) => bool2value(a >= b),

            _ => panic!(),
        }
    }

    fn se(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => bool2value(a <= b),
            (Value::Float(a), Value::Number(b)) => bool2value(a <= b as f32),
            (Value::Number(a), Value::Float(b)) => bool2value((a as f32) <= b),
            (Value::Float(a), Value::Float(b)) => bool2value(a <= b),

            _ => panic!(),
        }
    }

    fn lsh(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a << b),
            _ => panic!(),
        }
    }

    fn rsh(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a >> b),
            _ => panic!(),
        }
    }

    fn negate(self) -> Value {
        match self {
            Value::Number(a) => Value::Number(-a),
            Value::Float(a) => Value::Float(-a),

            _ => panic!(),
        }
    }

    fn not(self) -> Value {
        bool2value(!self.istruthy())
    }

    fn tpyeof(self) -> Value {
        match self {
            Value::String(_) => Value::String("string".to_string()),
            Value::FormattedString(_) => Value::String("string".to_string()),
            Value::Function { .. } => Value::String("function".to_string()),
            Value::Byte(_) => Value::String("byte".to_string()),
            Value::Number(_) => Value::String("number".to_string()),
            Value::Float(_) => Value::String("float".to_string()),
            Value::True | Value::False => Value::String("bool".to_string()),
            Value::Array(_) => Value::String("array".to_string()),
            Value::None => Value::String("none".to_string()),
            _ => panic!("istruthy on private variant"),
        }
    }
}

/* Represents an instruction, and is used in the compiler and the VM. */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Instruction {
    LoadConstant(usize),
    LoadVariable(usize),
    AssignVariable(usize),
    CallFunction(usize),
    CallPrimitiveFunction(usize, usize),

    Return,
    ReturnNone,

    LogicalNot,
    Negate,
    Typeof,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    LeftShift,
    RightShift,

    BinaryOr,
    BinaryAnd,

    More,
    Less,
    MoreEq,
    LessEq,
    Eq,
    UnEq,

    Jump(usize),
    JumpT(usize), // Jump if last value in stack is truthy
    JumpF(usize), // Jump if last value ins tack is falsy
}
