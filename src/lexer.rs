use crate::error::{raise, Context};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    FormattedStr(String),
    Keyword(String),

    Int(i64),
    Float(f64),
    Identifier(String),
    MacroIdentifier(String),

    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,

    Operator(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: Value,
    pub line: usize,
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

pub fn construct(value: Value, line: usize) -> Token {
    Token { value, line }
}

pub fn tokenize(mut source: String) -> Vec<Vec<Token>> {
    let mut tokens: Vec<Vec<Token>> = vec![vec![]];
    let mut line_tokens: &mut Vec<Token> = tokens.last_mut().unwrap();
    let mut line_idx: usize = 0;

    let keywords: Vec<&str> = vec!["let"];

    source.push(' ');
    let lines: Vec<&str> = source.split("\n").collect();
    let line: &str = lines[line_idx];
    let mut source_iter = source.char_indices();

    while let Some((i, ch)) = source_iter.next() {
        if ch == '\n' {
            line_idx += 1;
        } else if ch == ';' {
            tokens.push(Vec::new());
            line_tokens = tokens.last_mut().unwrap();
        } else if "()[]{}".contains(ch) {
            match ch {
                '(' => line_tokens.push(construct(Value::LParen, line_idx)),
                ')' => line_tokens.push(construct(Value::RParen, line_idx)),
                '[' => line_tokens.push(construct(Value::LSquare, line_idx)),
                ']' => line_tokens.push(construct(Value::RSquare, line_idx)),
                '{' => line_tokens.push(construct(Value::LCurly, line_idx)),
                '}' => line_tokens.push(construct(Value::RCurly, line_idx)),
                _ => {}
            }
        }
        // handle operators
        else if "+-*/><=!,^%".contains(ch) {
            let following = source.chars().nth(i + 1);

            match following {
                Option::Some(op) => {
                    if op == '=' {
                        line_tokens.push(construct(Value::Operator(format!("{}=", ch)), line_idx));
                        source_iter.next();
                    } else if op == '>' && ch == '-' {
                        line_tokens.push(construct(Value::Operator(format!("->")), line_idx));
                        source_iter.next();
                    } else if "<>".contains(op) && op == ch {
                        line_tokens.push(construct(
                            Value::Operator(format!("{}{}", op, ch)),
                            line_idx,
                        ));
                        source_iter.next();
                    } else if ch == '/' && op == '/' {
                        loop {
                            let ch = source_iter.next();

                            match ch {
                                Option::Some((_idx, ch)) => {
                                    if ch == '\n' {
                                        line_idx += 1;
                                        break;
                                    }
                                }

                                Option::None => break,
                            }
                        }
                    } else if ch == ',' {
                        line_tokens.push(construct(Value::Operator(String::from(",")), line_idx));
                    } else {
                        line_tokens.push(construct(Value::Operator(String::from(ch)), line_idx));
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

            // skip the amount of characters that were grabbed
            for _ in 0..i + 1 {
                source_iter.next();
            }

            match ch {
                '\'' => line_tokens.push(construct(Value::Str(string), line_idx)),
                '"' => line_tokens.push(construct(Value::Str(string), line_idx)),
                '`' => line_tokens.push(construct(Value::FormattedStr(string), line_idx)),
                _ => {}
            }
        }
        // handle identifiers
        else if ch.is_ascii_alphabetic() || "_!".contains(ch) {
            let (identifier, i) = grab(&source, i - 1, |fr| {
                fr.is_ascii_alphanumeric() || "_!".contains(fr)
            });

            // skip the amount of characters that were grabbed
            for _ in 0..i - 1 {
                source_iter.next();
            }

            match keywords.contains(&identifier.as_str()) {
                true => line_tokens.push(construct(Value::Keyword(identifier), line_idx)),
                false => {
                    if identifier.contains('!') {
                        if identifier.chars().filter(|&n| n == '!').count() == 1
                            && identifier.ends_with('!')
                        {
                            line_tokens
                                .push(construct(Value::MacroIdentifier(identifier), line_idx))
                        } else {
                            raise(
                                "Unexpected token '!'",
                                Context {
                                    line: line.to_string(),
                                    idx: line_idx,
                                    pointer: Option::Some(
                                        i + (identifier.len()
                                            - identifier.match_indices("!").nth(1).unwrap().0),
                                    ),
                                },
                            )
                        }
                    } else {
                        line_tokens.push(construct(Value::Identifier(identifier), line_idx))
                    }
                }
            }
        }
        // handle integers and floats
        else if "1234567890.".contains(ch) {
            let (int, r) = grab(&source, i - 1, |cu| "1234567890.".contains(cu));

            // skip the amount of characters that were grabbed
            for _ in 0..r - 1 {
                source_iter.next();
            }

            let count = int.matches(".").count();

            match count {
                0 => match int.parse::<i64>() {
                    Result::Ok(int) => line_tokens.push(construct(Value::Int(int), line_idx)),
                    Result::Err(_) => raise(
                        "Integer literal is too large.",
                        Context {
                            line: line.to_string(),
                            idx: line_idx,
                            pointer: Option::None,
                        },
                    ),
                },
                1 => match int.parse::<f64>() {
                    Result::Ok(float) => line_tokens.push(construct(Value::Float(float), line_idx)),
                    Result::Err(_) => raise(
                        "Float literal is too large.",
                        Context {
                            line: line.to_string(),
                            idx: line_idx,
                            pointer: Option::None,
                        },
                    ),
                },
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
