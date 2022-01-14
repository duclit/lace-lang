pub mod arithmetic;
pub mod common;
pub mod r#macro;
pub mod opcode;

use std::collections::HashMap;

use crate::error::{raise_internal, Data};
use crate::vm::opcode::*;

pub fn run(
    function: CodeObject,
    variables: HashMap<&String, Value>,
    global_funcs: Option<&HashMap<String, CodeObject>>,
) -> Value {
    let global_functions: &HashMap<String, CodeObject>;

    match global_funcs {
        Option::Some(global_funcs) => {
            global_functions = global_funcs;
        }
        Option::None => {
            global_functions = &function.functions;
        }
    }

    let mut stack: Vec<Value> = vec![];
    let mut variables: HashMap<&String, Value> = variables;

    let mut macros: HashMap<&str, fn(Vec<Value>) -> Value> = HashMap::new();

    macros.insert("writeln", r#macro::writeln);

    for opcode in function.code {
        match opcode {
            OpCode::LoadConst(idx) => stack.push(function.constants[idx].clone()),
            OpCode::LoadVariable(idx) => {
                if let Value::String(name) = function.constants[idx].clone() {
                    match variables.get(&name) {
                        Option::Some(value) => stack.push(value.clone()),
                        Option::None => Data::new(0, function.file.clone())
                            .raise(format!("Variable `{}` does not exist", name)),
                    }
                }
            }
            OpCode::AssignVar(idx) => {
                if let Value::String(name) = &function.constants[idx] {
                    let elem = stack.pop().unwrap();
                    variables.insert(&name, elem);
                }
            }
            OpCode::LoadBuiltinValue(idx) => match idx {
                0 => stack.push(Value::None),
                1 => stack.push(Value::Bool(true)),
                2 => stack.push(Value::Bool(false)),
                _ => raise_internal("0015"),
            },
            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Pow
            | OpCode::RShift
            | OpCode::LShift
            | OpCode::Equal
            | OpCode::NotEqual
            | OpCode::More
            | OpCode::Less
            | OpCode::MoreOrEqual => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let context = Data::new(usize::MAX, function.file.clone());

                stack.push(arithmetic::operate(&a, &b, opcode.clone(), context));
            }
            OpCode::CallMacro(idx, arg_len) => {
                if let Value::String(name) = &function.constants[idx] {
                    let mut arguments: Vec<Value> = vec![];

                    for _ in 0..arg_len {
                        arguments.push(stack.pop().unwrap());
                    }

                    arguments.reverse();

                    if !macros.contains_key(name.as_str()) {
                        Data::new(0, function.file.clone())
                            .raise(format!("Macro {} not found.", name))
                    }
                    
                    // get the function from the macros hashmap and call it
                    let value = macros.get(name.as_str()).unwrap()(arguments);
                    stack.push(value);
                } else {
                    raise_internal("0009")
                }
            }
            OpCode::CallFunction(idx, arg_len) => {
                if let Value::String(name) = &function.constants[idx] {
                    let mut arguments: Vec<Value> = vec![];

                    for _ in 0..arg_len {
                        arguments.push(stack.pop().unwrap());
                    }

                    arguments.reverse();

                    let func = match function.functions.get(name) {
                        Option::Some(func) => func,
                        Option::None => {
                            // if function doesn't exist in the local functions,
                            // check for global functions with that name.
                            let func = global_functions.get(name);

                            match func {
                                Option::Some(func) => func,
                                Option::None => Data::new(0, function.file.clone())
                                    .raise(format!("Function {} not found.", name)),
                            }
                        }
                    };

                    if arguments.len() != func.parameters.len() {
                        Data::new(0, function.file.clone()).raise(format!(
                            "Function {} expected {} arguments, got {}.",
                            name,
                            func.parameters.len(),
                            arguments.len()
                        ))
                    }

                    let mut args_map: HashMap<&String, Value> = HashMap::new();

                    for (name, value) in func.parameters.iter().zip(arguments) {
                        args_map.insert(name, value);
                    }

                    let res = run(func.clone(), args_map, Option::Some(global_functions));
                    stack.push(res);
                } else {
                    raise_internal("0016")
                }
            }
            _ => {}
        }
    }

    println!("{:?}", variables);
    return Value::None;
}
