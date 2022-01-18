use crate::error::{raise_internal, raise_rng, Context};
use crate::lexer::{Extract, Token, Value};
use crate::vm::opcode::Type;

use std::collections::HashMap;
use std::iter::Peekable;
use std::mem::discriminant;

#[derive(Debug, Clone)]
pub enum Node {
    Unary(Value),
    Binary(Box<Node>, Box<Node>, String),
    VariableInit(String, Box<Node>),
    Return(Box<Node>),
    FunctionCall(String, Vec<Node>),
    MacroCall(String, Vec<Node>),
}

#[derive(Debug, Clone)]
pub struct BinaryNode {
    pub a: Node,
    pub b: Node,
    pub o: String,
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
    tokens_iter: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    // create a new parser instance.
    pub fn new(tokens: Vec<Token>, source: Vec<String>) -> Parser {
        Parser {
            tokens: tokens.clone(),
            source,
            current: Token::new(Value::None, 0, 0, 0),
            current_idx: 0,
            tokens_iter: tokens.into_iter().peekable(),
        }
    }

    // raise an error.
    fn raise(&self, error: &str) -> ! {
        raise_rng(
            error,
            Context::new(
                self.current.line,
                &self.source,
                Option::Some(self.current.start),
            ),
            self.current.end - self.current.start,
        );
    }

    // expect a token with a certain value, gives a result telling whether the token was found or not.
    fn expect_token(&mut self, value: Value, exact: bool) -> Result<Value, ()> {
        if (!exact && discriminant(&self.current.value) == discriminant(&value))
            || (exact && self.current.value == value)
        {
            let res = Ok(self.current.value.clone());
            return res;
        } else {
            return Err(());
        }
    }

    // expect a token with any value from a list of values. Useful in expecting assignment operators. (*=, /=, +=, etc)
    fn expect_exact_tokens(&mut self, values: Vec<Value>) -> Result<Value, ()> {
        for value in values {
            if self.current.value == value {
                return Ok(self.current.value.clone());
            }
        }

        return Err(());
    }

    // same as expect_exact_tokens, but raises an error if the token is not found.
    fn expect_exact(&mut self, values: Vec<Value>, err: &str) -> Value {
        self.advance();

        match self.expect_exact_tokens(values) {
            Ok(value) => value,
            Err(_) => self.raise(err),
        }
    }

    // same as expect_token, but raises an error if the token is not found.
    fn expect(&mut self, value: Value, exact: bool, err: &str) -> Value {
        self.advance();

        match self.expect_token(value, exact) {
            Ok(value) => value,
            Err(_) => self.raise(err),
        }
    }

    // consume the next token in the tokens iterator.
    // MUST USE this as it sets a few attributes on this struct, and the parser might break otherwise.
    fn advance(&mut self) -> Option<Token> {
        let current = self.tokens_iter.next();

        match current.clone() {
            Some(token) => {
                self.current = token;
                self.current_idx += 1;
                current
            }
            None => None,
        }
    }

    fn consume_token(&mut self, value: Value) -> Result<(), ()> {
        if self.current.value != value {
            return Err(());
        }

        return Ok(());
    }

    fn consume(&mut self, value: Value, err: &str) {
        match self.consume_token(value) {
            Ok(_) => {
                self.advance();
            }
            Err(_) => self.raise(err),
        }
    }

    // convert a token value representing an operator to a string.
    fn operator_to_string(&mut self, operator: Value) -> String {
        match operator {
            Value::OpMul => "*".to_string(),
            Value::OpDiv => "/".to_string(),
            Value::OpPow => "^".to_string(),
            Value::OpAdd => "+".to_string(),
            Value::OpSub => "-".to_string(),
            _ => raise_internal("00"),
        }
    }

    // convert a token value representing a type to a type for the compiler/vm
    fn to_type(&mut self, value: Value) -> Type {
        match value {
            Value::TypeBool => Type::Bool,
            Value::TypeFloat => Type::Float,
            Value::TypeInt => Type::Integer,
            Value::TypeString => Type::String,
            _ => raise_internal("01"),
        }
    }

    // get a block of code. Raises an error if self.current.value is not Value::LCurly.
    fn get_block(&mut self) -> Vec<Token> {
        let mut block: Vec<Token> = vec![];
        let mut bracket_stack: Vec<Value> = vec![];

        match self.current.value {
            Value::LCurly => loop {
                let option = self.advance();

                match option {
                    Some(current) => match current.value {
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
                self.raise("Expected '{'.");
            }
        }

        return block;
    }

    pub fn literal(&mut self) -> Node {
        match self.current.value.clone() {
            Value::Int(_) | Value::String(_) | Value::FormattedString(_) | Value::Float(_) => {
                let val = Node::Unary(self.current.value.clone());
                println!("Scanning literal, current is: {:?}", self.current);
                self.advance();
                return val;
            }
            Value::MacroIdentifier(name) => {
                self.expect(
                    Value::LParen,
                    true,
                    "Expected opening parenthesis after macro name",
                );

                let mut arguments: Vec<Node> = vec![];
                self.advance();

                match self.expect_token(Value::RParen, true) {
                    Ok(_) => {}
                    Err(_) => {
                        arguments.push(self.expression());

                        while let Ok(_) = self.consume_token(Value::Comma) {
                            self.advance();
                            arguments.push(self.expression());
                        }
                    }
                }

                self.consume(Value::RParen, "Expected ')' after function call.");
                self.advance();

                Node::MacroCall(name.to_string(), arguments)
            }
            Value::Identifier(name) => match self.tokens_iter.peek() {
                Some(token) => match token.value {
                    Value::LParen => {
                        self.advance();
                        let mut arguments: Vec<Node> = vec![];
                        self.advance();

                        match self.expect_token(Value::RParen, true) {
                            Ok(_) => {}
                            Err(_) => {
                                arguments.push(self.expression());
        
                                while let Ok(_) = self.consume_token(Value::Comma) {
                                    self.advance();
                                    arguments.push(self.expression());
                                }
                            }
                        }

                        self.consume(Value::RParen, "Expected ')' after function call.");
                        self.advance();

                        Node::FunctionCall(name.to_string(), arguments)
                    }
                    _ => {let val = Node::Unary(self.current.value.clone()); self.advance(); val},
                },
                None => self.raise("Expected expression."),
            },
            _ => self.raise("Unexpected token."),
        }
    }

    // helper function for parsing binary expression.
    // builder -> the function you want to use to parse the left and right sides
    // operators -> the operators you recognize on this precedence level
    pub fn binary_expression(&mut self, builder: &str, operators: Vec<Value>) -> Node {
        let mut left = match builder {
            "literal" => self.literal(),
            "additive" => self.additive_expr(),
            "multiplicative" => self.multiplicative_expr(),
            _ => raise_internal("0024"),
        };

        while operators.contains(&self.current.value) {
            let operator = self.current.value.clone();
            self.advance();

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

    #[inline(always)]
    pub fn expression(&mut self) -> Node {
        self.additive_expr()
    }

    // main parse function
    pub fn parse(&mut self, chunk: &mut Function) {
        while let Some(current) = self.advance() {
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
                    self.advance();

                    let expression = self.expression();

                    println!("{:?}", self.current);

                    self.expect(
                        Value::Semicolon,
                        true,
                        "Unexpected token. Perhaps you missed a semicolon?",
                    );

                    chunk
                        .body
                        .push(Node::VariableInit(name, Box::new(expression)));
                }
                Value::KeywordReturn => {
                    let expression = self.expression();

                    self.expect(
                        Value::Semicolon,
                        true,
                        "Unexpected token. Perhaps you missed a semicolon?",
                    );

                    chunk.body.push(Node::Return(Box::new(expression)));
                }
                Value::MacroIdentifier(_)
                | Value::Identifier(_)
                | Value::Int(_)
                | Value::Float(_)
                | Value::String(_)
                | Value::FormattedString(_) => chunk.body.push(self.expression()),
                _ => {}
            }
        }

        println!("{:?}", chunk.body); // for debugging
    }
}
