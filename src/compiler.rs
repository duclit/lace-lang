use crate::lexer::Token;
use std::process::exit;

pub fn raise(err: &str) -> ! {
    println!("{}", err);
    exit(0);
}

pub enum Byte {
    Instruction(u8),
    Address(u32),
}

#[derive(Debug)]
pub enum Node {
    Unary(Token),
    Function(FunctionCall),
    List(Vec<Node>),
    Binary(Box<BinaryNode>),
}

#[derive(Debug)]
pub struct BinaryNode {
    a: Node,
    b: Node,
    o: String,
}

#[derive(Debug)]
pub struct FunctionCall {
    name: String,
    args: Vec<Node>,
}

pub fn has_token(tokens: &Vec<&Token>, t: Token) -> bool {
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match token {
            Token::LParen | Token::LCurly | Token::LSquare => bracket_stack.push(*token),
            Token::RParen | Token::RCurly | Token::RSquare => {
                let last_token = bracket_stack.pop();

                match last_token {
                    Option::Some(last_token) => {
                        if !(*last_token == get_opposite(*token)) {
                            raise("Unmatched bracket.")
                        }
                    }
                    Option::None => raise("Unmatched bracket."),
                }
            }
            _ => {
                if *token == &t && bracket_stack.is_empty() {
                    return true;
                }
            }
        }
    }

    return false;
}

fn get_opposite(token: &Token) -> Token {
    match token {
        &Token::LCurly => Token::RCurly,
        &Token::LParen => Token::RParen,
        &Token::LSquare => Token::RSquare,

        &Token::RCurly => Token::LCurly,
        &Token::RParen => Token::LParen,
        &Token::RSquare => Token::LSquare,

        _ => raise("Err#0001 - An unexpected error has occured."),
    }
}

fn parse_function_arguments(tokens: &Vec<&Token>) -> Vec<Node> {
    return parse_list(tokens);
}

fn parse_list(tokens: &Vec<&Token>) -> Vec<Node> {
    let mut elements: Vec<Vec<&Token>> = vec![vec![]];
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match token {
            Token::LParen | Token::LCurly | Token::LSquare => {
                bracket_stack.push(*token);
            
                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
            },
            Token::RParen | Token::RCurly | Token::RSquare => {
                let last_token = bracket_stack.pop();

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);

                match last_token {
                    Option::Some(last_token) => {
                        if !(*last_token == get_opposite(*token)) {
                            raise("Unmatched bracket.")
                        }
                    }
                    Option::None => raise("Unmatched bracket."),
                }
            }
            Token::Operator(op) => {
                if op == "," && bracket_stack.is_empty() {
                    elements.push(vec![]);
                } else {
                    let arg_len = elements.len() - 1;
                    elements[arg_len].push(*token)
                }
            }
            _ => {
                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
            }
        }
    }

    let mut ret: Vec<Node> = vec![];

    for element in elements {
        ret.push(parse_expression(element));
    }

    return ret;
}

fn parse_function_call(tokens: &Vec<&Token>) -> Node {
    let mut function_name = String::from("NaN");

    if let Token::Identifier(identifier) = tokens[0] {
        function_name = identifier.to_string();
    }

    if tokens[1] == &Token::LParen {
        match tokens[tokens.len() - 1] {
            Token::RParen => match tokens.len() {
                3 => {
                    return Node::Function(FunctionCall {
                        name: function_name,
                        args: vec![],
                    })
                }
                _ => {
                    let arguments = parse_function_arguments(&tokens[2..tokens.len() - 1].to_vec());
                    return Node::Function(FunctionCall {
                        name: function_name,
                        args: arguments,
                    });
                }
            },
            _ => raise("Invalid syntax."),
        }
    } else {
        raise("Expected parenthesis after function name.");
    }
}

fn parse_value(tokens: &Vec<&Token>) -> Node {
    match tokens.len() {
        0 => raise("Expected value"),
        _ => match tokens[0] {
            Token::Str(_) | Token::Int(_) | Token::Float(_) | Token::FormattedStr(_) => {
                match tokens.len() {
                    1 => return Node::Unary((*(*tokens)[0]).clone()),
                    _ => raise("Invalid syntax."),
                }
            }

            Token::Identifier(_) => match tokens.len() {
                1 => return Node::Unary((*(*tokens)[0]).clone()),
                2 => raise("Invalid syntax."),
                _ => return parse_function_call(tokens),
            },
            Token::LSquare => {
                match tokens[tokens.len() - 1] {
                    Token::RSquare => {
                        let list: Vec<Node> = parse_list(&tokens[1..tokens.len() - 1].to_vec());
                        return Node::List(list)
                    },
                    _ => raise("Invalid syntax.")
                }
             },
            _ => raise("Unexpected token."),
        },
    }
}

fn parse_mul_div(tokens: Vec<&Token>) -> Node {
    let mut rnode: Node = Node::Unary(Token::Int(0));
    let mut lnode: Node = Node::Unary(Token::Int(0));

    let mut operator: String = String::from(" ");

    for (idx, token) in (&tokens).into_iter().enumerate() {
        match token {
            Token::Operator(op) => {
                if *op == String::from("*") || *op == String::from("/") {
                    lnode = parse_value(&tokens[0..idx].to_vec());

                    operator = op.to_string();

                    let right = tokens[idx + 1..tokens.len()].to_vec();

                    if has_token(&right, Token::Operator("*".to_string()))
                        | has_token(&right, Token::Operator("/".to_string()))
                    {
                        rnode = parse_mul_div(right);
                    } else {
                        rnode = parse_value(&right)
                    }
                }
            }
            _ => {}
        }
    }

    return Node::Binary(Box::new(BinaryNode {
        a: lnode,
        b: rnode,
        o: operator,
    }));
}

pub fn parse_expression(tokens: Vec<&Token>) -> Node {
    if has_token(&tokens, Token::Operator("+".to_string()))
        | has_token(&tokens, Token::Operator("-".to_string()))
    {
        let mut left_tokens: Vec<&Token> = vec![];
        let mut bracket_stack: Vec<&Token> = vec![];

        let mut operator: String = " ".to_string();

        for token in (&tokens).into_iter() {
            if let Token::Operator(ref op) = *token {
                if (op == "+" || op == "-") && bracket_stack.is_empty() {
                    operator = op.to_string();
                    break;
                } else {
                    left_tokens.push(token);
                }
            } else {
                if *token == &Token::LCurly || *token == &Token::LParen || *token == &Token::LSquare
                {
                    bracket_stack.push(*token);
                    left_tokens.push(*token);
                } else if *token == &Token::RCurly
                    || *token == &Token::RParen
                    || *token == &Token::RSquare
                {
                    let last_token = bracket_stack.pop();
                    left_tokens.push(*token);

                    match last_token {
                        Option::Some(last_token) => {
                            if !(*last_token == get_opposite(*token)) {
                                raise("Unmatched bracket.")
                            }
                        }
                        Option::None => raise("Unmatched bracket."),
                    }
                }

                left_tokens.push(token);
            }
        }

        let right_tokens = tokens[left_tokens.len() + 1..tokens.len()].to_vec();

        let right_node: Node;
        let left_node: Node;

        if has_token(&left_tokens, Token::Operator("*".to_string()))
            | has_token(&left_tokens, Token::Operator("/".to_string()))
        {
            left_node = parse_mul_div(left_tokens);
        } else {
            left_node = parse_value(&left_tokens);
        }

        if has_token(&right_tokens, Token::Operator("+".to_string()))
            | has_token(&right_tokens, Token::Operator("-".to_string()))
        {
            right_node = parse_expression(right_tokens);
        } else {
            if has_token(&right_tokens, Token::Operator("*".to_string()))
                | has_token(&right_tokens, Token::Operator("/".to_string()))
            {
                right_node = parse_mul_div(right_tokens);
            } else {
                right_node = parse_value(&right_tokens);
            }
        }

        return Node::Binary(Box::new(BinaryNode {
            a: left_node,
            b: right_node,
            o: operator,
        }));
    } else if has_token(&tokens, Token::Operator("*".to_string()))
        | has_token(&tokens, Token::Operator("/".to_string()))
    {
        return parse_mul_div(tokens);
    } else {
        return parse_value(&tokens);
    }
}

//fn compile_expression(tree: Node) -> Vec<u8> { }

//pub fn compile(tokens: Vec<Token>) -> Vec<u8> { }
