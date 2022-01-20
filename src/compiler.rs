use std::collections::HashMap;

use crate::error::raise_internal;
use crate::lexer;
use crate::parser::{Function, Node};
use crate::vm::opcode::{CodeObject, OpCode, Value};

fn to_literal(value: &lexer::Value) -> Value {
    match value.clone() {
        lexer::Value::String(str) => Value::String(str),
        lexer::Value::FormattedString(str) => Value::String(str),
        lexer::Value::Int(int) => Value::Integer(int),
        lexer::Value::Float(float) => Value::Float(float),
        lexer::Value::Identifier(iden) => Value::String(iden),
        _ => raise_internal("0001"),
    }
}

fn get_operator_opcode(op: &String) -> OpCode {
    match op.as_str() {
        "+" => OpCode::Add,
        "-" => OpCode::Sub,
        "*" => OpCode::Mul,
        "/" => OpCode::Div,
        "%" => OpCode::Mod,
        "^" => OpCode::Pow,
        ">>" => OpCode::RShift,
        "<<" => OpCode::LShift,
        "==" => OpCode::Equal,
        "!=" => OpCode::NotEqual,
        "<=" => OpCode::LessOrEqual,
        ">=" => OpCode::MoreOrEqual,
        ">" => OpCode::More,
        "<" => OpCode::Less,
        _ => raise_internal("02"),
    }
}

pub fn compile_expression(tree: &Node, code: &mut CodeObject) {
    match tree {
        Node::Binary(left, right, op) => {
            compile_expression(left, code);
            compile_expression(right, code);
            code.add_code(get_operator_opcode(&op));
        }
        Node::Unary(value) => match value {
            lexer::Value::False | lexer::Value::True | lexer::Value::None => {
                code.add_code(OpCode::LoadBuiltinValue(match value {
                    lexer::Value::None => 0,
                    lexer::Value::True => 1,
                    lexer::Value::False => 2,
                    _ => raise_internal("0025"),
                }));
            }
            _ => {
                let index: usize = code.add_constant(to_literal(value));

                match value {
                    lexer::Value::Identifier(_) => code.add_code(OpCode::LoadVariable(index)),
                    _ => code.add_code(OpCode::LoadConst(index)),
                }

                if let lexer::Value::FormattedString(_) = value {
                    code.add_code(OpCode::FormatString)
                }
            }
        },
        Node::Array(arr) => {
            for element in arr {
                compile_expression(element, code);
            }

            code.add_code(OpCode::BuildList(arr.len()));
        }
        Node::MacroCall(name, arguments) => {
            let args_len = arguments.len();
            let name_idx = code.add_constant(Value::String(name.to_string()));

            for argument in arguments {
                compile_expression(&argument, code);
            }

            code.add_code(OpCode::CallMacro(name_idx, args_len));
        }
        _ => raise_internal("0022"),
    }
}

pub fn compile(main: Function) -> CodeObject {
    println!("Compiling func: {}{:?}", main.name, main.args);

    let mut code = CodeObject {
        code: vec![],
        constants: vec![],
        functions: HashMap::new(),
        file: main.file,
        parameters: main.args,
    };

    for node in main.body {
        match node {
            Node::VariableInit(name, value, _) => {
                compile_expression(&value, &mut code);

                let idx = code.add_constant(Value::String(name));
                code.add_code(OpCode::AssignVar(idx));
            }
            Node::VariableAssign(name, value) => {
                compile_expression(&value, &mut code);

                let idx = code.add_constant(Value::String(name));
                code.add_code(OpCode::AssignVar(idx));
            }
            Node::Unary(_) | Node::Binary(..) | Node::MacroCall(..) => {
                compile_expression(&node, &mut code);
            }
            Node::Return(value) => {
                compile_expression(&value, &mut code);
                code.add_code(OpCode::Return);
            }
            _ => {}
        }
    }

    // compile all local functions
    for (name, function) in main.local_functions {
        code.functions.insert(name, compile(function));
    }

    code
}
