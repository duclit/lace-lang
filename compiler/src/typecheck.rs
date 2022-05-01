use crate::{
    parser::{Node, NodeValue, Type},
    scanner::Token,
};
use std::collections::HashMap;

pub struct Typechecker {
    functions: HashMap<String, Type>,
    variables: HashMap<String, Type>,
}

fn binary_return_type(op: &str, left: Type, right: Type) -> Result<Type, ()> {
    match (op, left, right) {
        ("==", Type::Number, Type::Number) => Ok(Type::Bool),
        ("==", Type::String, Type::String) => Ok(Type::Bool),
        ("==", Type::Bool, Type::Bool) => Ok(Type::Bool),
        ("!=", Type::Number, Type::Number) => Ok(Type::Bool),
        ("!=", Type::String, Type::String) => Ok(Type::Bool),
        ("!=", Type::Bool, Type::Bool) => Ok(Type::Bool),
        (_, Type::Number, Type::Number) => Ok(Type::Number),
        (_, Type::Number, Type::Bool) => Ok(Type::Number),
        (_, Type::Bool, Type::Number) => Ok(Type::Number),
        ("+", Type::String, Type::String) => Ok(Type::String),
        ("*", Type::String, Type::Number) => Ok(Type::String),
        _ => Err(()),
    }
}

fn token_to_op(t: Token) -> &'static str {
    match t {
        Token::OpAdd => "+",
        Token::OpSub => "-",
        Token::OpMul => "*",
        Token::OpDiv => "/",
        Token::OpMod => "%",
        Token::OpEq => "==",
        Token::OpBangEq => "!=",
        Token::OpLess => "<",
        Token::OpLessEq => "<=",
        Token::OpMore => ">",
        Token::OpMoreEq => ">=",
        _ => panic!(),
    }
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    fn get_value_type(&self, value: NodeValue) -> Type {
        match value {
            NodeValue::NumberValue(_) => Type::Number,
            NodeValue::BoolValue(_) => Type::Bool,
            NodeValue::StringValue(_) => Type::String,
            NodeValue::IdentifierValue(iden) => {
                let var = self.variables.get(&iden);

                match var {
                    Some(t) => t.clone(),
                    None => panic!("Variable {} not found", iden),
                }
            }
            NodeValue::FunctionCall(name, _) => {
                let fun = self.functions.get(&name);

                match fun {
                    Some(t) => t.clone(),
                    None => panic!("Function {} not found", name),
                }
            }
            _ => panic!(),
        }
    }

    fn eval_binary_expression(&self, value: NodeValue) -> Result<Type, ()> {
        match value {
            NodeValue::Binary(left, right, op) => binary_return_type(
                token_to_op(op),
                self.eval_binary_expression(*left)?,
                self.eval_binary_expression(*right)?,
            ),
            _ => Ok(self.get_value_type(value)),
        }
    }

    fn initialise(&mut self, program: &[Node]) {
        for node in program.iter() {
            match node.inner.clone() {
                NodeValue::FunctionDecleration(name, _, _, _, return_type) => {
                    self.functions.insert(name.clone(), return_type);
                }
                _ => {}
            }
        }
    }

    pub fn check(&mut self, program: Vec<Node>) {
        self.initialise(&program);

        for node in program.iter() {
            match node.inner.clone() {
                NodeValue::VariableDecleration(name, value, _, _, annotation) => {
                    if let Ok(return_type) = self.eval_binary_expression(*value) {
                        if annotation == return_type {
                            self.variables.insert(name.clone(), return_type);
                        } else {
                            panic!("Expected type {:?}, got {:?}", annotation, return_type);
                        }
                    } else {
                        panic!("Error in variable decleration: Invalid Types.")
                    }
                }
                NodeValue::If(_if, _elseif, _else) => {
                    let if_type = self.eval_binary_expression((*_if.0).clone());

                    if let Err(_) = if_type {
                        panic!("Error in if statement (IF): Invalid types")
                    }

                    for (condition, _) in _elseif {
                        let if_type = self.eval_binary_expression((*condition).clone());

                        if let Err(_) = if_type {
                            panic!("Error in if statement (ELSEIF): Invalid types")
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
