use std::iter::Peekable;

use crate::error::{raise, Context};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    FormattedString(String),

    KeywordLet,
    KeywordFn,
    KeywordReturn,
    KeywordStruct,
    KeywordAs,
    KeywordMut,

    TypeInt,
    TypeFloat,
    TypeString,
    TypeBool,
    TypeArray,

    Int(i32),
    Float(f32),
    Identifier(String),
    MacroIdentifier(String),

    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,

    Comma,
    Assign,
    Bang,

    Colon,
    Semicolon,

    True,
    False,
    None,

    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpPow,
    OpMod,
    OpEq,
    OpUnEq,
    OpMore,
    OpLess,
    OpMoreEq,
    OpLessEq,
    OpLShift,
    OpRShift,
    OpAddAssign,
    OpSubAssign,
    OpMulAssign,
    OpDivAssign,
    OpPowAssign,
    OpModAssign,
}

pub type Identifier = String;

pub trait Extract<T> {
    fn extract(self) -> Option<T>;
}

impl Extract<Identifier> for Value {
    fn extract(self) -> Option<String> {
        if let Value::Identifier(i) = self {
            Some(i)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: Value,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(value: Value, line: usize, start: usize, end: usize) -> Token {
        Token {
            value,
            line,
            start,
            end,
        }
    }
}

pub struct Tokenizer {
    pub current: char,
    pub current_i: usize,
    pub line_i: usize,
    pub source: String,
    pub source_iter: Peekable<owned_chars::OwnedChars>,
    pub tokens: Vec<Token>,
    pub source_lines: Vec<String>,
}

impl Tokenizer {
    pub fn new(source: String) -> Tokenizer {
        let source_chars = owned_chars::OwnedChars::from_string(source.clone()).peekable();

        Tokenizer {
            current: ' ',
            source: source.clone(),
            source_iter: source_chars,
            current_i: 0,
            line_i: 0,
            tokens: vec![],
            source_lines: source.split('\n').map(|word| word.to_string()).collect(),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let current = self.source_iter.next();

        match current {
            Some(ch) => {
                self.current = ch;

                if ch == '\n' {
                    self.line_i += 1;
                    self.current_i = 0;
                } else {
                    self.current_i += 1;
                }

                Option::Some(ch)
            }

            None => Option::None,
        }
    }

    fn add_token(&mut self, value: Value, start: usize) {
        self.tokens
            .push(Token::new(value, self.line_i, start, self.current_i))
    }

    fn raise(&mut self, error: &str) -> ! {
        raise(
            error,
            Context {
                line: self.source_lines[self.line_i].to_string(),
                idx: self.line_i,
                pointer: Option::None,
            },
        )
    }

    fn string(&mut self, delimiter: char) {
        let mut string = String::new();
        let start_i = self.current_i - 1;

        while let Some(ch) = self.advance() {
            if ch == delimiter {
                break;
            } else {
                string.push(ch)
            }
        }

        match delimiter {
            '\'' | '"' => self.add_token(Value::String(string), start_i),
            '`' => self.add_token(Value::FormattedString(string), start_i),
            _ => {}
        }
    }

    fn identifier(&mut self) {
        let mut string = String::new();
        let start_i = self.current_i - 1;

        string.push(self.current);

        while let Some(ch) = self.source_iter.peek() {
            if ch.is_alphanumeric() || *ch == '_' {
                string.push(*ch);
                self.advance();
            } else {
                break;
            }
        }

        match string.as_str() {
            "let" => self.add_token(Value::KeywordLet, start_i),
            "fn" => self.add_token(Value::KeywordFn, start_i),
            "return" => self.add_token(Value::KeywordReturn, start_i),
            "as" => self.add_token(Value::KeywordAs, start_i),
            "mut" => self.add_token(Value::KeywordMut, start_i),
            "struct" => self.add_token(Value::KeywordStruct, start_i),

            "none" => self.add_token(Value::None, start_i),
            "true" => self.add_token(Value::True, start_i),
            "false" => self.add_token(Value::False, start_i),

            "Float" => self.add_token(Value::TypeFloat, start_i),
            "Int" => self.add_token(Value::TypeInt, start_i),
            "String" => self.add_token(Value::TypeString, start_i),
            "Bool" => self.add_token(Value::TypeBool, start_i),
            "Array" => self.add_token(Value::TypeArray, start_i),

            _ => {
                if let Some(ch) = self.source_iter.peek() {
                    if *ch == '!' {
                        self.advance();
                        self.add_token(Value::MacroIdentifier(string), start_i);
                    } else {
                        self.add_token(Value::Identifier(string), start_i)
                    }
                }
            }
        }
    }

    fn integer(&mut self) {
        let mut int = String::new();
        let start_i = self.current_i - 1;

        int.push(self.current);

        while let Some(ch) = self.source_iter.peek() {
            if "0.123456789".contains(*ch) {
                int.push(*ch);
                self.advance();
            } else {
                break;
            }
        }

        let count = int.matches('.').count();

        match count {
            0 => match int.parse::<i32>() {
                Result::Ok(int) => self.add_token(Value::Int(int), start_i),
                Result::Err(_) => self.raise("Integer literal is too large."),
            },
            1 => match int.parse::<f32>() {
                Result::Ok(float) => self.add_token(Value::Float(float), start_i),
                Result::Err(_) => self.raise("Float literal is too large."),
            },
            _ => self.raise("Float can have only one decimal point."),
        }
    }

    pub fn tokenize(&mut self) {
        let whitespace = regex::Regex::new(r"\s").unwrap();

        while let Some(ch) = self.advance() {
            if "'`\"".contains(ch) {
                self.string(ch);
            } else if "123456789.0".contains(ch) {
                self.integer();
            } else if ch.is_alphabetic() || ch == '_' {
                self.identifier();
            } else {
                match ch {
                    '{' => self.add_token(Value::LCurly, self.current_i - 1),
                    '}' => self.add_token(Value::RCurly, self.current_i - 1),
                    '(' => self.add_token(Value::LParen, self.current_i - 1),
                    ')' => self.add_token(Value::RParen, self.current_i - 1),
                    '[' => self.add_token(Value::LSquare, self.current_i - 1),
                    ']' => self.add_token(Value::RSquare, self.current_i - 1),
                    ':' => self.add_token(Value::Colon, self.current_i - 1),
                    ';' => self.add_token(Value::Semicolon, self.current_i - 1),
                    ',' => self.add_token(Value::Comma, self.current_i - 1),
                    _ => {
                        if let Some(&following) = self.source_iter.peek() {
                            if (following == '=' || following == '>' || following == '<')
                                & !whitespace.is_match(ch.to_string().as_str())
                            {
                                self.advance();
                            }

                            match (ch, following) {
                                ('=', '=') => self.add_token(Value::OpEq, self.current_i),
                                ('!', '=') => self.add_token(Value::OpUnEq, self.current_i),
                                ('>', '>') => self.add_token(Value::OpRShift, self.current_i),
                                ('<', '<') => self.add_token(Value::OpLShift, self.current_i),
                                ('>', '=') => self.add_token(Value::OpMoreEq, self.current_i),
                                ('<', '=') => self.add_token(Value::OpLessEq, self.current_i),
                                ('+', '=') => self.add_token(Value::OpAddAssign, self.current_i),
                                ('-', '=') => self.add_token(Value::OpSubAssign, self.current_i),
                                ('*', '=') => self.add_token(Value::OpMulAssign, self.current_i),
                                ('/', '=') => self.add_token(Value::OpDivAssign, self.current_i),
                                ('^', '=') => self.add_token(Value::OpPowAssign, self.current_i),
                                ('%', '=') => self.add_token(Value::OpModAssign, self.current_i),
                                ('>', _) => self.add_token(Value::OpMore, self.current_i - 1),
                                ('<', _) => self.add_token(Value::OpLess, self.current_i - 1),
                                ('!', _) => self.add_token(Value::Bang, self.current_i - 1),
                                ('+', _) => self.add_token(Value::OpAdd, self.current_i - 1),
                                ('-', _) => self.add_token(Value::OpSub, self.current_i - 1),
                                ('*', _) => self.add_token(Value::OpMul, self.current_i - 1),
                                ('/', _) => self.add_token(Value::OpDiv, self.current_i - 1),
                                ('^', _) => self.add_token(Value::OpPow, self.current_i - 1),
                                ('%', _) => self.add_token(Value::OpMod, self.current_i - 1),
                                ('=', _) => self.add_token(Value::Assign, self.current_i - 1),
                                _ => {
                                    if !whitespace.is_match(ch.to_string().as_str()) {
                                        self.raise(format!("Unknown character '{}'", ch).as_str())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("{:?}", self.tokens);
    }
}
