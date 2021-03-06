use crate::scanner::Token;

use super::parser::*;
use hlvm::{
    hir::*,
    lir::{HlvmInstruction, HlvmValue},
};

fn op_token_to_instruction(op: Token) -> HlvmHirInstruction {
    match op {
        Token::OpAdd => HlvmHirInstruction::Add,
        Token::OpSub => HlvmHirInstruction::Subtract,
        Token::OpMul => HlvmHirInstruction::Multiply,
        Token::OpDiv => HlvmHirInstruction::Divide,
        Token::OpMod => todo!("Modulo not yet implemented"),
        Token::OpEq => HlvmHirInstruction::Equal,
        Token::OpBangEq => HlvmHirInstruction::NotEqual,
        Token::OpLess => HlvmHirInstruction::LessThan,
        Token::OpLessEq => HlvmHirInstruction::LessThanOrEqual,
        Token::OpMore => HlvmHirInstruction::GreaterThan,
        Token::OpMoreEq => HlvmHirInstruction::GreaterThanOrEqual,
        _ => panic!(),
    }
}

fn compile_value(value: NodeValue) -> Vec<HlvmHirInstruction> {
    let mut instructions = vec![];

    match value {
        NodeValue::StringValue(string) => {
            instructions.push(HlvmHirInstruction::Push(HlvmValue::String(string)))
        }
        NodeValue::NumberValue(number) => {
            instructions.push(HlvmHirInstruction::Push(HlvmValue::Number(number)))
        }
        NodeValue::BoolValue(bool) => {
            instructions.push(HlvmHirInstruction::Push(HlvmValue::Bool(bool)))
        }
        NodeValue::IdentifierValue(iden) => {
            instructions.push(HlvmHirInstruction::Get(iden));
        }
        NodeValue::FunctionCall(function, mut arguments) => {
            arguments.reverse();
            let mut arguemnts_hir = vec![];

            for argument in arguments {
                arguemnts_hir.append(&mut compile_value(argument));
            }

            instructions.append(&mut arguemnts_hir);
            instructions.push(HlvmHirInstruction::Get(function));
            instructions.push(HlvmHirInstruction::Call)
        }
        NodeValue::PrimitiveFunctionCall(index, mut arguments) => {
            arguments.reverse();
            let len = arguments.len();

            for argument in arguments {
                instructions.append(&mut compile_value(argument));
            }

            instructions.push(HlvmHirInstruction::CallPrimitive(index, len));
        }
        NodeValue::Binary(left, right, op) => {
            instructions.append(&mut compile_value(*left));
            instructions.append(&mut compile_value(*right));
            instructions.push(op_token_to_instruction(op));
        }
        NodeValue::Unary(value, modifier) => {
            instructions.append(&mut compile_value(*value));
            
            match modifier {
                Unary::Negate => instructions.push(HlvmHirInstruction::Negate),
                Unary::Not => instructions.push(HlvmHirInstruction::Not),
                Unary::Typeof => instructions.push(HlvmHirInstruction::Typeof)
            }
        }
        _ => panic!(),
    }

    instructions
}

pub fn compile(ast: Vec<Node>) -> Vec<HlvmHirInstruction> {
    let mut instructions = vec![];

    for node in ast {
        match node.inner {
            NodeValue::VariableDecleration(name, value, ..) => {
                instructions.append(&mut compile_value(*value));
                instructions.push(HlvmHirInstruction::SetLocal(name));
            }
            NodeValue::VariableAssignment(name, value) => {
                instructions.append(&mut compile_value(*value));
                instructions.push(HlvmHirInstruction::SetLocal(name));
            }
            NodeValue::If(ontrue, onelseif, onfalse) => {
                let ontrue_body = compile(ontrue.1);
                let mut onelseif_hir: Vec<(Vec<HlvmHirInstruction>, Vec<HlvmHirInstruction>)> =
                    Vec::with_capacity(onelseif.len());

                let onelseif_isempty = onelseif.is_empty();

                for elseif in onelseif {
                    onelseif_hir.push((compile_value(*elseif.0), compile(elseif.1)));
                }

                instructions.append(&mut compile_value(*ontrue.0));
                instructions.push(HlvmHirInstruction::IfStatement {
                    ontrue: ontrue_body,
                    onelseif: if onelseif_isempty {
                        None
                    } else {
                        Some(onelseif_hir)
                    },
                    onfalse: compile(onfalse.unwrap_or(vec![])),
                })
            }
            NodeValue::WhileStatement(condition, body) => {
                instructions.push(HlvmHirInstruction::WhileStatement(
                    compile_value(*condition),
                    compile(body),
                ));
            }
            NodeValue::Return(value) => {
                instructions.append(&mut compile_value(*value));
                instructions.push(HlvmHirInstruction::ReturnValue);
            }
            NodeValue::FunctionDecleration(name, body, params, ..) => {
                let mut parameters = vec![];

                for parameter in params {
                    parameters.push(parameter.name);
                }

                let function = HlvmValue::Function(from_hir(compile(body)), parameters, None);

                instructions.push(HlvmHirInstruction::Push(function));
                instructions.push(HlvmHirInstruction::SetLocal(name));
            }
            NodeValue::StructInit(name, arguments) => {
                for argument in arguments {
                    instructions.append(&mut compile_value(argument.1.inner));
                }

                instructions.push(HlvmHirInstruction::Get(name));
                instructions.push(HlvmHirInstruction::Instantiate);
            }
            // NodeValue::TypeDecleration(name, functions, variables) => {
            //     let mut attributes = variables;

            //     for function in functions {
            //         if let NodeValue::FunctionDecleration(name, ..) = function {
            //             
            //         }
            //     }
            // }

            NodeValue::StringValue(..)
            | NodeValue::NumberValue(..)
            | NodeValue::BoolValue(..)
            | NodeValue::IdentifierValue(..)
            | NodeValue::FunctionCall(..)
            | NodeValue::PrimitiveFunctionCall(..)
            | NodeValue::Binary(..)
            | NodeValue::Unary(..) => {
                instructions.append(&mut compile_value(node.inner));
            }

            _ => todo!(),
        }
    }

    instructions
}
