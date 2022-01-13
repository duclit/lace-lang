use std::collections::HashMap;

use crate::error::raise_internal;
use crate::lexer;
use crate::parser::{Function, Node};
use crate::vm::opcode::{Code, CodeObject, OpCode, Value};

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

fn get_operator_opcode(op: &String) -> Code {
    match op.as_str() {
        "+" => Code::OpCode(OpCode::Add),
        "-" => Code::OpCode(OpCode::Sub),
        "*" => Code::OpCode(OpCode::Mul),
        "/" => Code::OpCode(OpCode::Div),
        "%" => Code::OpCode(OpCode::Mod),
        "^" => Code::OpCode(OpCode::Pow),
        ">>" => Code::OpCode(OpCode::RShift),
        "<<" => Code::OpCode(OpCode::LShift),
        "==" => Code::OpCode(OpCode::Equal),
        "!=" => Code::OpCode(OpCode::NotEqual),
        "<=" => Code::OpCode(OpCode::LessOrEqual),
        ">=" => Code::OpCode(OpCode::MoreOrEqual),
        ">" => Code::OpCode(OpCode::More),
        "<" => Code::OpCode(OpCode::Less),
        _ => raise_internal("0002"),
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
                code.add_code(Code::OpCode(OpCode::LoadBuiltinValue));

                match value {
                    lexer::Value::None => code.add_code(Code::Number(0)),
                    lexer::Value::True => code.add_code(Code::Number(1)),
                    lexer::Value::False => code.add_code(Code::Number(2)),
                    _ => {}
                }
            }
            _ => {
                let index: usize = code.add_constant(to_literal(value));

                match value {
                    lexer::Value::Identifier(_) => {
                        code.add_code(Code::OpCode(OpCode::LoadVariable))
                    }
                    _ => code.add_code(Code::OpCode(OpCode::LoadConst)),
                }

                code.add_code(Code::Number(index));

                if let lexer::Value::FormattedString(_) = value {
                    code.add_code(Code::OpCode(OpCode::FormatString))
                }
            }
        },
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
            Node::VariableInit(name, value) => {
                compile_expression(&value, &mut code);

                let idx = code.add_constant(Value::String(name));
                code.add_code(Code::OpCode(OpCode::LoadConst));
                code.add_code(Code::Number(idx));

                code.add_code(Code::OpCode(OpCode::AssignVar));
            }
            Node::Unary(_) | Node::Binary(..) => {
                compile_expression(&node, &mut code);
            }
            Node::Return(value) => {
                compile_expression(&value, &mut code);
                code.add_code(Code::OpCode(OpCode::Return))
            }
        }
    }

    // compile all local functions
    for (name, function) in main.local_functions {
        code.functions.insert(name, compile(function));
    }

    code
}
