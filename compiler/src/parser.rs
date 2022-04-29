use crate::error::*;
use crate::scanner::Token;
use colored::*;
use logos::Lexer;
use std::mem::discriminant;

/// Represents a unary operation
#[derive(Debug, Clone)]
pub enum Unary {
    Negate,
    Typeof,
    Not,
}

pub type Public = bool;
pub type Mutable = bool;
pub type ConditionalBlock = (Box<NodeValue>, Vec<Node>);

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Number,
    Bool,
    Array(Box<Type>),
    Void,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub mutable: bool,
    pub datatype: Type,
}

#[derive(Debug, Clone)]
pub enum NodeValue {
    // Values are integrated into NodeValue so that it's easier to type
    StringValue(String),
    IdentifierValue(String),
    NumberValue(f64),
    BoolValue(bool),
    ArrayValue(Vec<NodeValue>),
    FunctionCall(String, Vec<NodeValue>),
    NoneValue,

    Unary(Box<NodeValue>, Unary),
    Binary(Box<NodeValue>, Box<NodeValue>, Token),

    GetAttribute(Box<NodeValue>, String),

    FunctionDecleration(String, Vec<Node>, Vec<Parameter>, Public, Type),
    VariableDecleration(String, Box<NodeValue>, Public, Mutable, Type),
    VariableAssignment(String, Box<NodeValue>),
    WhileStatement(Box<NodeValue>, Vec<Node>),
    ImportStatement(String, String),
    If(ConditionalBlock, Vec<ConditionalBlock>, Option<Vec<Node>>),
}

/// Contains a NodeValue along with additional metadata, like which line the node was on.
#[derive(Debug, Clone)]
pub struct Node {
    pub inner: NodeValue,
    pub line: usize,
}

impl Node {
    pub fn new(value: NodeValue, line: usize) -> Node {
        Node { inner: value, line }
    }
}

pub struct Parser<'a> {
    source: String,

    // Just used to determine the current line index
    line: usize,
    last: usize,

    pub ast: Vec<Node>,
    pub tokens: Lexer<'a, Token>,

    pub current: Token,
}

impl<'p> Parser<'p> {
    /// Creates a new Parser.
    /// Requires the lexer iterator to contain atleast one token, and will panic otherwise.
    pub fn new(mut tokens: Lexer<Token>, source: String) -> Parser {
        let first = tokens.next().unwrap();

        Parser {
            source,
            line: 0,
            last: 0,

            tokens,

            ast: vec![],
            current: first,
        }
    }

    /// Advance the `tokens` iterator
    fn advance(&mut self) -> Token {
        match self.tokens.next() {
            Some(token) => {
                self.current = token.clone();

                let mut nl = 0;

                for ch in self.source.chars().skip(self.last) {
                    if ch == '\n' {
                        nl += 1
                    }
                }

                self.line += nl;
                token
            }
            None => {
                self.current = Token::End;
                Token::End
            }
        }
    }

    fn get_error_data(&mut self) -> (String, String, String, usize, &str) {
        let span = self.tokens.span();
        let mut line = 0;
        let lines: Vec<&str> = self.source.split('\n').collect();
        let mut last_n = 0;

        // please forgive me
        for (i, character) in self.source.char_indices() {
            if i == span.start {
                break;
            } else if character == '\n' {
                line += 1;
                last_n = i + 1;
            }
        }

        let line_len = line.to_string().len();

        (
            " ".repeat(line_len),
            " ".repeat(span.start - last_n),
            "^".repeat(span.end - span.start),
            line + 1,
            lines[line],
        )
    }

    /// Raise an error
    fn error(&mut self, error: &str) -> ! {
        let (empty, spacing, pointer, line_idx, line_text) = self.get_error_data();
        ErrorHandler::error(empty, spacing, pointer, line_idx, line_text, error);
    }

    /// Raise an error, with a tip
    fn error_tip(&mut self, error: &str, tip: &str) -> ! {
        let (empty, spacing, pointer, line_idx, line_text) = self.get_error_data();
        ErrorHandler::error_tip(empty, spacing, pointer, line_idx, line_text, error, tip);
    }

    /// Print a warning to the console
    fn warn(&mut self, warning: &str) {
        println!("{}: {}", "Warning".bright_yellow(), warning);
    }

    /// Advances the tokens iterator and checks if the current token is the token specified.
    fn expect(&mut self, token: Token, exact: bool) -> bool {
        let next = self.advance();
        (exact && next == token) || (!exact && discriminant(&next) == discriminant(&token))
    }

    /// Raises an error if self.expect(token, exact) is false.
    fn expect_handle(&mut self, token: Token, exact: bool, error: &str) {
        if !self.expect(token, exact) {
            self.error(error);
        }
    }

    /// Parse a value, the smallest part of an expression
    fn value(&mut self) -> Node {
        let current = self.current.clone();
        self.advance();

        match current {
            Token::Number(num) => Node::new(NodeValue::NumberValue(num), self.line),
            Token::True => Node::new(NodeValue::BoolValue(true), self.line),
            Token::False => Node::new(NodeValue::BoolValue(false), self.line),
            Token::String(ref str) => Node::new(NodeValue::StringValue(str.to_string()), self.line),
            Token::Identifier(iden) => match self.current {
                Token::LeftParen => {
                    self.advance();
                    let mut arguments: Vec<NodeValue> = vec![];

                    if !(self.current == Token::RightParen) {
                        arguments.push(self.expression().inner);

                        while self.current == Token::Comma {
                            self.advance();

                            if !(self.current == Token::RightParen) {
                                arguments.push(self.expression().inner);
                            }
                        }
                    }

                    self.advance();
                    Node {
                        inner: NodeValue::FunctionCall(iden, arguments),
                        line: self.line,
                    }
                }
                _ => Node {
                    inner: NodeValue::IdentifierValue(iden),
                    line: self.line,
                },
            },
            Token::LeftSquare => {
                let mut elements: Vec<NodeValue> = vec![];

                if !(self.current == Token::RightSquare) {
                    elements.push(self.expression().inner);

                    while self.current == Token::Comma {
                        self.advance();

                        if !(self.current == Token::RightSquare) {
                            elements.push(self.expression().inner);
                        }
                    }
                }

                self.advance();
                Node {
                    inner: NodeValue::ArrayValue(elements),
                    line: self.line,
                }
            }
            Token::LeftParen => {
                let expression = self.expression();

                match self.current {
                    Token::RightParen => {}
                    _ => self.error("Expected ')' after expression."),
                }

                self.advance();
                expression
            }
            _ => todo!(),
        }
    }

    fn unary(&mut self) -> Node {
        match &self.current {
            Token::OpBang => {
                self.advance();
                Node::new(
                    NodeValue::Unary(Box::new(self.value().inner), Unary::Not),
                    self.line,
                )
            }
            Token::OpSub => {
                self.advance();
                Node::new(
                    NodeValue::Unary(Box::new(self.unary().inner), Unary::Negate),
                    self.line,
                )
            }
            Token::KwTypeof => {
                self.advance();
                Node::new(
                    NodeValue::Unary(Box::new(self.unary().inner), Unary::Typeof),
                    self.line,
                )
            }
            _ => self.value(),
        }
    }

    fn from_builder(&mut self, builder: &str) -> Node {
        match builder {
            "unary" => self.unary(),
            "additive" => self.additive_expression(),
            "comparison" => self.comparison_expression(),
            "multiplicative" => self.multiplicative_expression(),
            "bitwise_or" => self.bitwise_expression_1(),
            "bitwise_xor" => self.bitwise_expression_2(),
            "bitwise_and" => self.bitwise_expression_3(),
            _ => panic!("Unknown builder '{}'", builder),
        }
    }

    /* Helper function for parsing binary expression.
       `builder` -> the function you want to use to parse the left and right sides
       `operators` -> the operators you recognize on this precedence level
    */
    fn binary_expression(&mut self, builder: &str, operators: Vec<Token>) -> Node {
        let mut left = self.from_builder(builder);

        while operators.contains(&self.current) {
            let operator = self.current.clone();
            self.advance();

            let right = self.from_builder(builder);

            left = Node {
                inner: NodeValue::Binary(Box::new(left.inner), Box::new(right.inner), operator),
                line: 0,
            };
        }

        left
    }

    fn logical_expression(&mut self) -> Node {
        self.binary_expression("comparison", vec![Token::KwAnd, Token::KwOr])
    }

    fn comparison_expression(&mut self) -> Node {
        self.binary_expression(
            "additive",
            vec![
                Token::OpEq,
                Token::OpBangEq,
                Token::OpLess,
                Token::OpMore,
                Token::OpMoreEq,
                Token::OpLessEq,
            ],
        )
    }

    // The highest level of a bitwise operation, scans only for bitwise OR
    fn bitwise_expression_1(&mut self) -> Node {
        self.binary_expression("bitwise_xor", vec![Token::BitwiseOr])
    }

    // The second highest level of a bitwise operation, scans only for bitwise XOR
    fn bitwise_expression_2(&mut self) -> Node {
        self.binary_expression("bitwise_and", vec![Token::BitwiseXor])
    }

    // The lowest level of a bitwise operation, scans only for bitwise AND
    fn bitwise_expression_3(&mut self) -> Node {
        self.binary_expression("comparison", vec![Token::BitwiseAnd])
    }

    fn additive_expression(&mut self) -> Node {
        self.binary_expression("multiplicative", vec![Token::OpAdd, Token::OpSub])
    }

    fn multiplicative_expression(&mut self) -> Node {
        self.binary_expression(
            "unary",
            vec![
                Token::OpMul,
                Token::OpDiv,
                Token::OpMod,
                Token::OpPow,
                Token::OpLeftShift,
                Token::OpRightShift,
            ],
        )
    }

    #[inline(always)]
    fn expression(&mut self) -> Node {
        self.logical_expression()
    }

    fn parse_type(&mut self) -> Type {
        if let Token::Identifier(_type) = &self.current {
            let datatype = match _type.clone().as_str() {
                "number" => Type::Number,
                "bool" => Type::Bool,
                "string" => Type::String,
                _ => self.error("Unknown type."),
            };

            self.advance();
            return datatype;
        } else {
            self.error("Expected Identifier");
        };
    }

    fn variable_decleration(&mut self, public: bool) -> Node {
        let (is_mutable, name) = match self.advance() {
            Token::KwMut => match self.advance() {
                Token::Identifier(iden) => (true, iden),
                _ => self.error("Expected Identifier after 'mut'"),
            },
            Token::Identifier(name_) => (false, name_),
            _ => self.error("Expected either 'mut' or Identifier."),
        };

        let datatype = match self.advance() {
            Token::Colon => {
                self.advance();
                let dt = self.parse_type();

                match self.current {
                    Token::Assign => {
                        self.advance();
                    }
                    _ => {
                        self.error("Expected '='");
                    }
                }

                dt
            }
            _ => self.error("Expected ':'"),
        };

        let value = self.expression();

        Node {
            inner: NodeValue::VariableDecleration(
                name,
                Box::new(value.inner),
                public,
                is_mutable,
                datatype,
            ),
            line: self.line,
        }
    }

    fn variable_assignment(&mut self, name: String) -> Node {
        self.advance();
        let value = self.expression();

        Node {
            inner: NodeValue::VariableAssignment(name, Box::new(value.inner)),
            line: self.line,
        }
    }

    fn function_decleration(&mut self, public: bool) -> Node {
        self.expect_handle(
            Token::Identifier("".to_string()),
            false,
            "Expected identifier",
        );

        let name = self.current.clone();
        let mut params: Vec<Parameter> = vec![];

        self.expect_handle(Token::LeftParen, true, "Expected '(' after function name.");
        self.advance();

        while self.current != Token::RightParen {
            match &self.current {
                Token::Identifier(_) | Token::KwMut => {
                    let (name, mutable) = match self.current.clone() {
                        Token::Identifier(iden) => {
                            self.advance();
                            (iden, false)
                        }
                        Token::KwMut => (
                            match self.advance() {
                                Token::Identifier(str) => {
                                    self.advance();
                                    str.clone()
                                }
                                _ => self.error("Expected identifier"),
                            },
                            true,
                        ),
                        _ => panic!(),
                    };

                    println!("{:?}", self.current);

                    if self.current != Token::Colon {
                        self.error("Expected ':' after parameter name.");
                    }

                    self.advance();
                    let datatype = self.parse_type();

                    let param = Parameter {
                        name,
                        mutable,
                        datatype,
                    };

                    params.push(param);
                }
                Token::RightParen => break,
                Token::Comma => {
                    self.advance();
                }
                _ => self.error("Expected either `mut` or identifier."),
            }
        }

        if let Token::Identifier(name) = name {
            self.advance();

            let return_type = match self.current {
                Token::LeftCurly => {
                    self.advance();
                    Type::Void
                }
                Token::Colon => {
                    self.advance();
                    let return_type = self.parse_type();

                    match self.current {
                        Token::LeftCurly => {
                            self.advance();
                        }
                        _ => self.error("Expected '{'"),
                    }

                    return_type
                }
                _ => self.error("Expected ':' or '{'"),
            };

            let mut body = Vec::new();

            while self.current != Token::RightCurly {
                body.push(self.statement());
            }

            self.advance();

            Node {
                inner: NodeValue::FunctionDecleration(name, body, params, public, return_type),
                line: self.line,
            }
        } else {
            self.error("Expected function name");
        }
    }

    fn while_statement(&mut self) -> Node {
        self.advance();
        let condition = self.expression();

        if self.current != Token::LeftCurly {
            self.error("Expected '{' after while statement.");
        }

        self.advance();
        let mut body: Vec<Node> = vec![];

        while self.current != Token::RightCurly {
            body.push(self.statement());
        }

        self.advance();

        Node {
            inner: NodeValue::WhileStatement(Box::new(condition.inner), body),
            line: self.line,
        }
    }

    fn import_statement(&mut self) -> Node {
        self.expect(Token::String(String::new()), false);

        if let Token::String(path) = self.current.clone() {
            self.expect(Token::KwAs, true);
            self.expect(Token::Identifier(String::new()), false);

            if let Token::Identifier(name) = self.current.clone() {
                self.advance();
                Node {
                    inner: NodeValue::ImportStatement(path, name),
                    line: self.line,
                }
            } else {
                self.error("Expected path to file.");
            }
        } else {
            self.error("Expected path to file.");
        }
    }

    fn if_statement(&mut self) -> Node {
        self.advance();
        let condition = self.expression();

        if self.current != Token::LeftCurly {
            self.error("Expected '{' after if statement.");
        }

        self.advance();
        let mut body: Vec<Node> = vec![];

        while self.current != Token::RightCurly {
            body.push(self.statement());
        }

        self.advance();

        let mut else_body: Vec<Node> = vec![];
        let mut else_if_bodies: Vec<ConditionalBlock> = vec![];

        if self.current == Token::KwElse {
            self.advance();

            match self.current {
                Token::LeftCurly => {
                    self.advance();

                    while self.current != Token::RightCurly {
                        else_body.push(self.statement());
                    }

                    self.advance();
                }
                Token::KwIf => {
                    let statement = self.if_statement();

                    if let NodeValue::If(_if, elseif, _else) = statement.inner {
                        else_if_bodies.push(_if);

                        for body in elseif {
                            else_if_bodies.push(body);
                        }

                        if let Option::Some(code) = _else {
                            else_body = code;
                        }
                    }
                }
                _ => self.error("Expected '{' or 'if'."),
            }
        }

        let else_body = if else_body.is_empty() {
            None
        } else {
            Some(else_body)
        };

        Node {
            inner: NodeValue::If((Box::new(condition.inner), body), else_if_bodies, else_body),
            line: self.line,
        }
    }

    fn statement(&mut self) -> Node {
        match self.current {
            Token::KwLet => self.variable_decleration(false),
            Token::KwFn => self.function_decleration(false),
            Token::KwPub => match self.advance() {
                Token::KwLet => self.variable_decleration(true),
                Token::KwFn => self.function_decleration(true),
                _ => self.error("Expected 'let' or 'fn' after 'pub'"),
            },
            Token::KwWhile => self.while_statement(),
            Token::KwUse => self.import_statement(),
            Token::KwIf => self.if_statement(),
            Token::Identifier(_) => {
                /*  Lines that start with identifiers can either be assignments or expressions.
                    Therefore, we parse an expression, and if expression is a sole identifier and
                        the next token is a '=', it's an assignment.
                */
                let node = self.expression();

                if let NodeValue::IdentifierValue(iden) = &node.inner {
                    if self.current == Token::Assign {
                        self.variable_assignment(iden.clone())
                    } else {
                        node
                    }
                } else {
                    node
                }
            }
            Token::Number(_)
            | Token::String(_)
            | Token::FormattedString(_)
            | Token::PrimitiveFnIdentifier(_)
            | Token::LeftSquare
            | Token::True
            | Token::False
            | Token::None
            | Token::KwTypeof
            | Token::OpBang
            | Token::OpSub => self.expression(),
            _ => todo!(),
        }
    }

    pub fn parse(&mut self) {
        while self.current != Token::End {
            let statement = self.statement();
            self.ast.push(statement);
        }
    }
}
