use crate::error::{raise, Context};
use std::process::exit;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Str(String),
    FormattedStr(String),
    Keyword(String),

    Int(i32),
    Float(f32),
    Identifier(String),

    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Newline,

    Operator(String),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Scanning {
    None,
    Str(char),
    Int,
    Identifier,
}

fn grab<F: Fn(char) -> bool>(source: &String, i: usize, check: F) -> (String, usize) {
    let source: String = (*source).chars().skip(i + 1).collect::<String>();

    let mut idx: usize = 0;
    let mut captured: String = String::new();

    for ch in source.chars() {
        if check(ch) {
            captured.push(ch);
            idx += 1;
        } else {
            break;
        }
    }

    return (captured, idx);
}

pub fn tokenize(mut source: String) -> Vec<Vec<Token>> {
    let mut tokens: Vec<Vec<Token>> = vec![vec![]];
    let mut line_tokens: &mut Vec<Token> = tokens.last_mut().unwrap();
    let mut line_idx: usize = 0;

    let identifiers: Vec<&str> = vec!["let"];

    source.push(' ');
    let lines: Vec<&str> = source.split("\n").collect();
    let line: &str = lines[line_idx];
    let mut source_iter = source.char_indices();

    while let Some((i, ch)) = source_iter.next() {
        if ch == '\n' {
            line_idx += 1;
            line_tokens.push(Token::Newline)
        } else if ch == ';' {
            tokens.push(Vec::new());
            line_tokens = tokens.last_mut().unwrap();
        } else if "()[]{}".contains(ch) {
            match ch {
                '(' => line_tokens.push(Token::LParen),
                ')' => line_tokens.push(Token::RParen),
                '[' => line_tokens.push(Token::LSquare),
                ']' => line_tokens.push(Token::RSquare),
                '{' => line_tokens.push(Token::LCurly),
                '}' => line_tokens.push(Token::RCurly),
                _ => {}
            }
        }
        // handle operators
        else if "+-*/><=!,^%".contains(ch) {
            let following = source.chars().nth(i + 1);

            match following {
                Option::Some(op) => {
                    if op == '=' {
                        line_tokens.push(Token::Operator(format!("{}=", ch)));
                        source_iter.next();
                    } else if op == '>' && ch == '-' {
                        line_tokens.push(Token::Operator(format!("->")));
                        source_iter.next();
                    } else if "<>".contains(op) && op == ch {
                        line_tokens.push(Token::Operator(format!("{}{}", op, ch)));
                        source_iter.next();
                    } else if ch == '/' && op == '/' {
                        loop {
                            let ch = source_iter.next();

                            match ch {
                                Option::Some((_idx, ch)) => {
                                    if ch == '\n' {
                                        break;
                                    }
                                }

                                Option::None => break,
                            }
                        }
                    } else if ch == ',' {
                        line_tokens.push(Token::Operator(String::from(",")));
                    } else {
                        line_tokens.push(Token::Operator(String::from(ch)));
                    }
                }
                Option::None => raise(
                    "Expected value",
                    Context {
                        line: line.to_string(),
                        idx: line_idx,
                        pointer: Option::None,
                    },
                ),
            }
        }
        // handle strings
        else if "\"'`".contains(ch) {
            let (string, i) = grab(&source, i, move |de| de != ch);

            for _ in 0..i + 1 {
                source_iter.next();
            }

            match ch {
                '\'' => line_tokens.push(Token::Str(string)),
                '"' => line_tokens.push(Token::Str(string)),
                '`' => line_tokens.push(Token::FormattedStr(string)),
                _ => {}
            }
        }
        // handle identifiers
        else if ch.is_ascii_alphabetic() || ch == '_' {
            let (identifier, i) =
                grab(&source, i - 1, |fr| fr.is_ascii_alphanumeric() || fr == '_');

            for _ in 0..i - 1 {
                source_iter.next();
            }
            
            match identifiers.contains(&identifier.as_str()) {
                true => line_tokens.push(Token::Keyword(identifier)),
                false => line_tokens.push(Token::Identifier(identifier))
            }
        }
        // handle integers and floats
        else if "1234567890.".contains(ch) {
            let (int, r) = grab(&source, i - 1, |cu| "1234567890.".contains(cu));

            for _ in 0..r - 1 {
                source_iter.next();
            }

            let count = int.matches(".").count();

            match count {
                0 => line_tokens.push(Token::Int(int.parse::<i32>().unwrap())),
                1 => line_tokens.push(Token::Float(int.parse::<f32>().unwrap())),
                _ => raise(
                    "Float can only have one decimal point",
                    Context {
                        line: line.to_string(),
                        idx: line_idx,
                        pointer: Option::Some(
                            i + (int.len() - int.match_indices(".").nth(1).unwrap().0),
                        ),
                    },
                ),
            }
        }
    }

    return tokens;
}
