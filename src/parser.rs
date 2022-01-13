use crate::error::{raise, raise_internal, Context};
use crate::lexer::{Extract, Token, Value};
use crate::vm::opcode;
use crate::vm::opcode::Type;

use std::collections::HashMap;
use std::iter::Peekable;
use std::mem::discriminant;

#[derive(Debug, Clone)]
pub enum Node {
    Unary(Value),
    Binary(Box<Node>, Box<Node>, String),
    VariableInit(String, Box<Node>),
    Return(Box<Node>)
}

#[derive(Debug, Clone)]
pub struct BinaryNode {
    pub a: Node,
    pub b: Node,
    pub o: String,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Node>,
    pub ismacro: bool,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Node>,
    pub file: String,
    pub local_functions: HashMap<String, Function>,
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub source: Vec<String>,
    pub current: Token,
    current_idx: usize,
    tokens_iter: Peekable<std::iter::Enumerate<std::vec::IntoIter<Token>>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: Vec<String>) -> Parser {
        Parser {
            tokens: tokens.clone(),
            source,
            current: Token::new(Value::None, 0, 0, 0),
            current_idx: 0,
            tokens_iter: tokens.into_iter().enumerate().peekable(),
        }
    }

    fn raise(&self, error: &str) -> ! {
        raise(
            error,
            Context::new(
                self.current.line,
                &self.source,
                Option::Some(self.current.start),
            ),
        );
    }

    fn expect_token(&mut self, value: Value, exact: bool) -> Result<Value, ()> {
        let token: Option<&(usize, Token)> = self.tokens_iter.peek();

        match token {
            Some((_, token)) => {
                if (!exact && discriminant(&token.value) == discriminant(&value))
                    || (exact && token.value == value)
                {
                    let res = Ok(token.value.clone());
                    self.advance();

                    return res;
                } else {
                    return Err(());
                }
            }
            None => {
                return Err(());
            }
        }
    }

    fn expect_exact_tokens(&mut self, values: Vec<Value>) -> Result<Value, ()> {
        let token: Option<(usize, Token)> = self.tokens_iter.next();

        for value in values {
            match token {
                Some((idx, ref token)) => {
                    if token.value == value {
                        self.current = token.clone();
                        self.current_idx = idx;

                        return Ok(token.value.clone());
                    }
                }
                None => {
                    return Err(());
                }
            }
        }

        return Err(());
    }

    fn expect_exact(&mut self, values: Vec<Value>, err: &str) -> Value {
        match self.expect_exact_tokens(values) {
            Ok(value) => value,
            Err(_) => self.raise(err),
        }
    }

    fn expect(&mut self, value: Value, exact: bool, err: &str) -> Value {
        match self.expect_token(value, exact) {
            Ok(value) => value,
            Err(_) => self.raise(err),
        }
    }

    fn advance(&mut self) -> Option<(usize, Token)> {
        let current = self.tokens_iter.next();
        println!("{:?}", current);

        match current.clone() {
            Some((idx, token)) => {
                self.current = token;
                self.current_idx = idx;
                current
            }
            None => None,
        }
    }

    fn operator_to_string(&mut self, operator: Value) -> String {
        match operator {
            Value::OpMul => "*".to_string(),
            Value::OpDiv => "/".to_string(),
            Value::OpPow => "^".to_string(),
            Value::OpAdd => "+".to_string(),
            Value::OpSub => "-".to_string(),
            _ => raise_internal("0023"),
        }
    }

    fn to_type(&mut self, value: Value) -> Type {
        match value {
            Value::TypeBool => Type::Bool,
            Value::TypeFloat => Type::Float,
            Value::TypeInt => Type::Integer,
            Value::TypeString => Type::String,
            _ => raise_internal("0019"),
        }
    }

    fn get_block(&mut self) -> Vec<Token> {
        let mut block: Vec<Token> = vec![];
        let mut bracket_stack: Vec<Value> = vec![];

        match self.current.value {
            Value::LCurly => loop {
                let option = self.advance();

                match option {
                    Some((_, current)) => match current.value {
                        Value::RCurly => {
                            if bracket_stack.is_empty() {
                                break;
                            } else {
                                block.push(current);
                            }
                        }
                        Value::LCurly => {
                            bracket_stack.push(Value::LCurly);
                            block.push(current);
                        }
                        _ => block.push(current),
                    },
                    None => break,
                }
            },
            _ => {
                self.raise("Expected opening bracket");
            }
        }

        return block;
    }

    fn get_line(&mut self) -> Vec<Token> {
        let mut line: Vec<Token> = vec![self.current.clone()];

        while let Some((idx, current)) = self.advance() {
            self.current = current.clone();
            self.current_idx = idx;

            match current.value {
                Value::Semicolon => break,
                _ => line.push(current),
            }
        }

        return line;
    }

    pub fn literal(&mut self) -> Node {
        let token = self.advance();

        match token {
            Some((_, token)) => match token.value {
                Value::Int(_) | Value::String(_) | Value::FormattedString(_) | Value::Identifier(_) | Value::Float(_) => return Node::Unary(token.value),
                _ => self.raise("Unexpected token.")
            }
            None => self.raise("Expected value.") 
        }
    }

    pub fn binary_expression(&mut self, builder: &str, operators: Vec<Value>) -> Node {
        let mut left = match builder {
            "literal" => self.literal(),
            "additive" => self.additive_expr(),
            "multiplicative" => self.multiplicative_expr(),
            _ => raise_internal("0024"),
        };

        while operators.contains(&self.tokens_iter.peek().unwrap().1.value) {
            self.advance();
            let operator = self.current.value.clone();

            let right = match builder {
                "literal" => self.literal(),
                "additive" => self.additive_expr(),
                "multiplicative" => self.multiplicative_expr(),
                _ => raise_internal("0025"),
            };

            left = Node::Binary(
                Box::new(left.clone()),
                Box::new(right),
                self.operator_to_string(operator),
            );
        }

        return left;
    }

    pub fn multiplicative_expr(&mut self) -> Node {
        self.binary_expression("literal", vec![Value::OpMul, Value::OpDiv, Value::OpPow])
    }

    pub fn additive_expr(&mut self) -> Node {
        self.binary_expression("multiplicative", vec![Value::OpAdd, Value::OpSub])
    }

    pub fn primary_expr(&mut self) -> Node {
        self.additive_expr()
    }

    pub fn parse(&mut self, chunk: &mut Function) {
        while let Some((_, current)) = self.advance() {
            match current.value {
                Value::KeywordFn => {
                    let name = self.expect(
                        Value::Identifier(String::new()),
                        false,
                        "Expected identifier",
                    );

                    let name: String = name.extract().unwrap();

                    self.expect(Value::LCurly, true, "Expected opening brackets.");
                    let block: Vec<Token> = self.get_block();

                    let mut function: Function = Function {
                        name: name.clone(),
                        args: vec![],
                        body: vec![],
                        file: String::from("main.lc"),
                        local_functions: HashMap::new(),
                    };

                    let mut parser: Parser = Parser::new(block, self.source.clone());
                    parser.parse(&mut function);

                    chunk.local_functions.insert(name, function);
                }
                Value::KeywordLet => {
                    let name = self.expect(
                        Value::Identifier(String::new()),
                        false,
                        "Expected identifier",
                    );

                    let name: String = name.extract().unwrap();

                    self.expect_exact(vec![Value::Assign], "Expected assignment operator.");
                    let expression = self.primary_expr();
                    self.expect(Value::Semicolon, true, "Unexpected token. Perhaps you missed a semicolon?");

                    chunk.body.push(Node::VariableInit(name, Box::new(expression)));
                }
                Value::KeywordReturn => {
                    let expression = self.primary_expr();
                    self.expect(Value::Semicolon, true, "Unexpected token. Perhaps you missed a semicolon?");

                    chunk.body.push(Node::Return(Box::new(expression)));
                }
                _ => {}
            }
        }

        println!("{:?}", chunk.body);
    }
}
