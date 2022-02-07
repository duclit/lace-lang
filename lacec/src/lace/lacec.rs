use {
    lacec::common::{Instruction, Node, UnaryOp, Value},
    lacec::scanner::Token,
};

pub struct Constants(pub Vec<Value>);

impl Constants {
    pub fn add_constant(&mut self, value: Value) -> usize {
        match self.0.iter().position(|x| x.clone() == value) {
            Option::Some(index) => index,
            Option::None => {
                self.0.push(value);
                self.0.len() - 1
            }
        }
    }
}

pub struct Compiler {
    source: std::vec::IntoIter<Node>,
    pub constants: Constants,
}

impl Compiler {
    pub fn new(source: Vec<Node>) -> Compiler {
        Compiler {
            source: source.into_iter(),
            constants: Constants(vec![]),
        }
    }

    fn compile_expression(&mut self, node: &Node, chunk: &mut Vec<Instruction>) {
        match node {
            Node::Value(value) => match value {
                Value::Identifier(_) => {
                    let index = self.constants.add_constant(value.clone());
                    chunk.push(Instruction::LoadVariable(index))
                }
                Value::NodeArray(_) => {}
                _ => {
                    let index = self.constants.add_constant(value.clone());
                    chunk.push(Instruction::LoadConstant(index))
                }
            },
            Node::Unary(node, operation) => {
                self.compile_expression(node, chunk);

                chunk.push(match operation {
                    UnaryOp::LogicalNot => Instruction::LogicalNot,
                    UnaryOp::Negate => Instruction::Negate,
                    UnaryOp::Typeof => Instruction::Typeof,
                });
            }
            Node::Binary {
                left,
                right,
                operator,
            } => {
                self.compile_expression(left, chunk);
                self.compile_expression(right, chunk);

                chunk.push(match operator {
                    Token::OpAdd => Instruction::Add,
                    Token::OpSub => Instruction::Sub,
                    Token::OpMul => Instruction::Mul,
                    Token::OpDiv => Instruction::Div,
                    Token::OpMod => Instruction::Mod,
                    Token::OpPow => Instruction::Pow,
                    Token::OpBangEq => Instruction::UnEq,
                    Token::OpEq => Instruction::Eq,
                    Token::OpLessEq => Instruction::LessEq,
                    Token::OpMoreEq => Instruction::MoreEq,
                    Token::OpLeftShift => Instruction::LeftShift,
                    Token::OpRightShift => Instruction::RightShift,
                    Token::OpLess => Instruction::Less,
                    Token::OpMore => Instruction::More,
                    Token::KwAnd => Instruction::BinaryAnd,
                    Token::KwOr => Instruction::BinaryOr,
                    _ => panic!(),
                });
            }
            Node::FunctionCall { name, arguments } => {
                let len = arguments.len();
                let index = self
                    .constants
                    .add_constant(Value::Identifier(name.to_string()));

                for argument in arguments {
                    self.compile_expression(argument, chunk);
                }

                chunk.push(Instruction::LoadVariable(index));
                chunk.push(Instruction::CallFunction(len))
            }
            Node::PrimitiveFunctionCall { name, arguments } => {
                let len = arguments.len();
                let index = self
                    .constants
                    .add_constant(Value::Identifier(name.to_string()));

                for argument in arguments {
                    self.compile_expression(argument, chunk);
                }

                chunk.push(Instruction::CallPrimitiveFunction(len, index))
            }
            _ => panic!(),
        }
    }

    fn compile_node(&mut self, node: Node, chunk: &mut Vec<Instruction>) {
        match node {
            Node::Unary(..)
            | Node::Value(_)
            | Node::Binary { .. }
            | Node::FunctionCall { .. }
            | Node::PrimitiveFunctionCall { .. } => self.compile_expression(&node, chunk),

            Node::Return(value) => {
                self.compile_expression(&*value, chunk);
                chunk.push(Instruction::Return);
            }
            Node::ReturnNone => chunk.push(Instruction::ReturnNone),

            Node::VariableDeclr { name, value, .. }
            | Node::VariableAssignment { name, value, .. } => {
                self.compile_expression(&value, chunk);

                let name_index = self.constants.add_constant(Value::Identifier(name));
                chunk.push(Instruction::AssignVariable(name_index))
            }

            Node::Function {
                name,
                params,
                function,
                coroutine,
                ..
            } => {
                let mut code: Vec<Instruction> = vec![];

                for node in function {
                    self.compile_node(node, &mut code);
                }

                let mut parameters: Vec<String> = vec![];

                for param in params {
                    parameters.push(param.name);
                }

                let function = Value::Function {
                    code,
                    parameters,
                    coroutine,
                };

                let function_index = self.constants.add_constant(function);
                chunk.push(Instruction::LoadConstant(function_index));

                let name_index = self.constants.add_constant(Value::Identifier(name));
                chunk.push(Instruction::AssignVariable(name_index))
            }

            Node::WhileLoop { condition, body } => {
                let jump_idx = chunk.len();

                self.compile_expression(&*condition, chunk);
                chunk.push(Instruction::JumpF(0)); // The index will be modified later
                let jumpf_idx = chunk.len() - 1;

                for node in body {
                    self.compile_node(node, chunk);
                }

                chunk.push(Instruction::Jump(jump_idx));
                chunk[jumpf_idx] = Instruction::JumpF(chunk.len());
            }
        }
    }

    pub fn compile(&mut self, chunk: &mut Vec<Instruction>) {
        while let Some(node) = self.source.next() {
            self.compile_node(node, chunk);
        }
    }
}
