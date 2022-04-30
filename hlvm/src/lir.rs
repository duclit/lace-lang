use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

type Address = usize;

/// The amount of arguments a function takes, along with their names.
pub(crate) type Arguments = Vec<String>;

/// The amount of space that needs to be allocated for a function's locals.
pub(crate) type LocalPreAlloc = Option<usize>;

/// Values supported by the high level virtual machine.
/// * Number - 64 bit float
/// * String - String
/// * Function - Functions are values that can be called.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HlvmValue {
    Number(f64),
    Bool(bool),
    String(String),

    StructInstance(HashMap<String, HlvmValue>),
    StructBlueprint(Vec<String>),

    Function(Vec<HlvmInstruction>, Arguments, LocalPreAlloc),
    BuiltInFunction(usize, usize),
}

impl HlvmValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            HlvmValue::Number(val) => *val != 0.0,
            HlvmValue::String(val) => !val.is_empty(),
            HlvmValue::Bool(val) => *val,
            HlvmValue::Function(..)
            | HlvmValue::StructInstance(..)
            | HlvmValue::StructBlueprint(..)
            | HlvmValue::BuiltInFunction(..) => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HlvmInstruction {
    Push(HlvmValue),

    /// Pops the value from the stack and calls it.
    /// Top of the stack must be a function, otherwise HLVM will panic.
    /// The arguments passed into the function must be in reversed order.
    Call,
    CallPrimitive(usize, usize),

    /// Returns HlvmValue::Int32(0)
    Return,
    /// Returns the value at the top of the stack
    ReturnValue,

    GetLocal(String),
    GetGlobal(String),
    SetLocal(String),
    SetGlobal(String),

    /// Gets the value of attribute `n` of the value at the top of the stack.
    GetAttribute(String),

    /// This is a wierd one because in order to win against the borrow checker,
    /// the value you want to set the attribute on must be loaded before the desired
    /// value of the attribute itself.
    SetAttribute(String),

    /// Instantiate a value. Calls the `instantiate` method defined in the Instantiable trait.
    /// (look at `dev.rs` if you want to view the default implementation or change it)
    ///
    /// Unlike SetAttribute, the object you want to initialize must be on top of the stack,
    /// followed by any attributes.
    Instantiate,

    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,
    BinaryAnd,
    BinaryOr,
    Not,
    Negate,
    Typeof,

    Jump(Address),
    JumpIf(Address),
}

#[derive(Clone, Debug)]
pub struct HlvmCallFrame {
    pub locals: HashMap<String, HlvmValue>,
}
