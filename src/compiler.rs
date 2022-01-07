use crate::error::raise_internal;
use crate::lexer;
use crate::opcode::{Code, CodeObject, OpCode, Value};
use crate::parser::Node;

fn to_literal(value: &lexer::Value) -> Value {
    match value.clone() {
        lexer::Value::Str(str) => Value::String(str),
        lexer::Value::FormattedStr(str) => Value::String(str),
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
        _ => raise_internal("0002"),
    }
}

pub fn compile_expression(tree: &Node, code: &mut CodeObject) {
    match tree {
        Node::Unary(value) => {
            let index: usize = code.add_constant(to_literal(value));

            match value {
                lexer::Value::Identifier(_) => code.add_code(Code::OpCode(OpCode::LoadVariable)),
                _ => code.add_code(Code::OpCode(OpCode::LoadConst)),
            }

            code.add_code(Code::Number(index));

            match value {
                lexer::Value::FormattedStr(_) => code.add_code(Code::OpCode(OpCode::FormatString)),
                _ => {}
            }
        }
        Node::Binary(binary) => {
            compile_expression(&binary.a, code);
            compile_expression(&binary.b, code);
            code.add_code(get_operator_opcode(&binary.o))
        }
        Node::List(list) => {
            for expression in list {
                compile_expression(&expression, code);
            }

            code.add_code(Code::OpCode(OpCode::BuildList));
            code.add_code(Code::Number(list.len()))
        }
        _ => {}
    }
}

pub fn compile(ast: Vec<Node>) -> CodeObject {
    let mut code = CodeObject {
        code: vec![],
        constants: vec![],
    };

    for node in ast {
        match node {
            Node::Assignment(assignment) => {
                compile_expression(&assignment.value, &mut code);

                let name = assignment.name;
                let idx = code.add_constant(Value::String(name));
                code.add_code(Code::OpCode(OpCode::LoadConst));
                code.add_code(Code::Number(idx));

                code.add_code(Code::OpCode(OpCode::AssignVar));
            }
            _ => {}
        }
    }

    code
}
