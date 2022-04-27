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
    use Type::*;

    match (op, left, right) {
        (_, NumberType, NumberType) => Ok(NumberType),
        (_, NumberType, BoolType) => Ok(NumberType),
        (_, BoolType, NumberType) => Ok(NumberType),
        ("+", StringType, StringType) => Ok(StringType),
        ("*", StringType, NumberType) => Ok(StringType),
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
            NodeValue::NumberValue(_) => Type::NumberType,
            NodeValue::BoolValue(_) => Type::BoolType,
            NodeValue::StringValue(_) => Type::StringType,
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

    fn initialise(&mut self, program: &Vec<Node>) {
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
                    }
                }
                _ => {}
            }
        }
    }
}
