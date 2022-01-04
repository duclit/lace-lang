use crate::error::{raise, raise_internal, BaseContext, Context};
use crate::lexer::Token;

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

// compute the line of a token relative to the raw source code, as lines are actually seperated
// by semicolons instead of newlines internally.
pub fn compute_token_line(tokens: &Vec<Token>, base: usize, index: usize) -> usize {
    let mut newlines = 0;

    for (idx, token) in tokens.iter().enumerate() {
        if idx == index {
            return base + newlines;
        } else {
            match token {
                Token::Newline => newlines += 1,
                _ => {}
            }
        }
    }

    return base;
}

pub fn has_operators(tokens: &Vec<&Token>, operators: Vec<&str>) -> bool {
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match token {
            Token::LParen | Token::LCurly | Token::LSquare => bracket_stack.push(*token),
            Token::RParen | Token::RCurly | Token::RSquare => {
                bracket_stack.pop();
            }
            _ => {
                for operator in &operators {
                    if *token == &Token::Operator(operator.to_string()) && bracket_stack.is_empty()
                    {
                        return true;
                    }
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

        _ => raise_internal("Err 0001 - An unexpected error has occured."),
    }
}

fn parse_function_arguments(
    tokens: &Vec<&Token>,
    start_idx: usize,
    context: &BaseContext,
) -> Vec<Node> {
    return parse_list(tokens, start_idx, context);
}

fn parse_list(tokens: &Vec<&Token>, start_idx: usize, context: &BaseContext) -> Vec<Node> {
    let mut elements: Vec<Vec<&Token>> = vec![vec![]];
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match token {
            Token::LParen | Token::LCurly | Token::LSquare => {
                bracket_stack.push(*token);

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
            }
            Token::RParen | Token::RCurly | Token::RSquare => {
                bracket_stack.pop();

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
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
        ret.push(parse_expression(element, start_idx, context));
    }

    return ret;
}

fn parse_function_call(tokens: &Vec<&Token>, start_idx: usize, context: &BaseContext) -> Node {
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
                    let arguments = parse_function_arguments(
                        &tokens[2..tokens.len() - 1].to_vec(),
                        start_idx,
                        context,
                    );
                    return Node::Function(FunctionCall {
                        name: function_name,
                        args: arguments,
                    });
                }
            },
            _ => {
                let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
                raise(
                    "Invalid syntax. Perhaps you forgot a comma?",
                    Context {
                        idx: line_idx,
                        line: context.source[line_idx].clone(),
                        pointer: Option::None,
                    },
                )
            }
        }
    } else {
        let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
        raise(
            "Expected parenthesis after function name.",
            Context {
                idx: line_idx,
                line: context.source[line_idx].clone(),
                pointer: Option::None,
            },
        );
    }
}

fn parse_value(tokens: &Vec<&Token>, start_idx: usize, context: &BaseContext) -> Node {
    match tokens.len() {
        0 => {
            let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
            raise(
                "Expected value",
                Context {
                    idx: line_idx,
                    line: context.source[line_idx].clone(),
                    pointer: Option::None,
                },
            )
        }
        _ => match tokens[0] {
            Token::Str(_) | Token::Int(_) | Token::Float(_) | Token::FormattedStr(_) => {
                match tokens.len() {
                    1 => return Node::Unary((*(*tokens)[0]).clone()),
                    _ => {
                        let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
                        raise(
                            "Expected only one value",
                            Context {
                                idx: line_idx,
                                line: context.source[line_idx].clone(),
                                pointer: Option::None,
                            },
                        )
                    }
                }
            }

            Token::Identifier(_) => match tokens.len() {
                1 => return Node::Unary((*(*tokens)[0]).clone()),
                2 => {
                    let message: &str;

                    match tokens[1] {
                        Token::LParen => {
                            message =
                                "Unexpected opening parenthesis. Perhaps you forgot to close them?";
                        }
                        _ => message = "Expected only one token. Perhaps you forgot a comma?",
                    }

                    let line_idx = compute_token_line(&context.tokens, context.base, start_idx);

                    raise(
                        message,
                        Context {
                            idx: line_idx,
                            line: context.source[line_idx].clone(),
                            pointer: Option::None,
                        },
                    );
                }
                _ => return parse_function_call(tokens, start_idx, context),
            },
            Token::LSquare => match tokens[tokens.len() - 1] {
                Token::RSquare => {
                    let list: Vec<Node> =
                        parse_list(&tokens[1..tokens.len() - 1].to_vec(), start_idx, context);
                    return Node::List(list);
                }
                _ => {
                    let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
                    raise(
                        "Invalid syntax. Perhaps you forgot a comma?",
                        Context {
                            idx: line_idx,
                            line: context.source[line_idx].clone(),
                            pointer: Option::None,
                        },
                    )
                }
            },
            _ => {
                let line_idx = compute_token_line(&context.tokens, context.base, start_idx);
                raise(
                    "Unexpected token.",
                    Context {
                        idx: line_idx,
                        line: context.source[line_idx].clone(),
                        pointer: Option::None,
                    },
                )
            }
        },
    }
}

fn parse_mul_div(tokens: Vec<&Token>, start_idx: usize, context: &BaseContext) -> Node {
    let mut rnode: Node = Node::Unary(Token::Int(0));
    let mut lnode: Node = Node::Unary(Token::Int(0));

    let mut operator: String = String::from(" ");

    for (idx, token) in (&tokens).into_iter().enumerate() {
        match token {
            Token::Operator(op) => {
                if *op == String::from("*") || *op == String::from("/") {
                    lnode = parse_value(&tokens[0..idx].to_vec(), start_idx, context);

                    operator = op.to_string();

                    let right = tokens[idx + 1..tokens.len()].to_vec();

                    if has_operators(&right, vec!["*", "/"]) {
                        rnode = parse_mul_div(right, start_idx, context);
                    } else {
                        rnode = parse_value(&right, start_idx, context)
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

pub fn parse_expression(tokens: Vec<&Token>, start_idx: usize, context: &BaseContext) -> Node {
    if has_operators(&tokens, vec!["+", "-"]) {
        let mut left_tokens: Vec<&Token> = vec![];
        let mut bracket_stack: Vec<&Token> = vec![];

        let mut operator: String = " ".to_string();

        for (idx, token) in (&tokens).into_iter().enumerate() {
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
                    bracket_stack.pop();
                    left_tokens.push(*token);
                }

                left_tokens.push(token);
            }
        }

        let right_tokens = tokens[left_tokens.len() + 1..tokens.len()].to_vec();

        let right_node: Node;
        let left_node: Node;

        if has_operators(&left_tokens, vec!["*", "/"]) {
            left_node = parse_mul_div(left_tokens, start_idx, context);
        } else {
            left_node = parse_value(&left_tokens, start_idx, context);
        }

        if has_operators(&right_tokens, vec!["+", "-"]) {
            right_node = parse_expression(right_tokens, start_idx, context);
        } else {
            if has_operators(&right_tokens, vec!["*", "/"]) {
                right_node = parse_mul_div(right_tokens, start_idx, context);
            } else {
                right_node = parse_value(&right_tokens, start_idx, context);
            }
        }

        return Node::Binary(Box::new(BinaryNode {
            a: left_node,
            b: right_node,
            o: operator,
        }));
    } else if has_operators(&tokens, vec!["*", "/"]) {
        return parse_mul_div(tokens, start_idx, context);
    } else {
        return parse_value(&tokens, start_idx, context);
    }
}

//fn compile_expression(tree: Node) -> Vec<u8> { }

pub fn parse(tokens: Vec<Vec<Token>>, source: String) -> Vec<Node> {
    let temp_lines: Vec<&str> = source.split("\n").collect();
    let mut lines: Vec<String> = vec![];

    let mut nodes: Vec<Node> = vec![];

    for line in temp_lines {
        lines.push(line.to_string());
    }

    for (idx, line) in tokens.iter().enumerate() {
        if line.is_empty() { continue; }
        let mut ref_line: Vec<&Token> = vec![];

        for token in line {
            ref_line.push(&token)
        }

        let node = parse_expression(
            ref_line,
            0,
            &BaseContext {
                tokens: (*line).clone(),
                base: idx,
                source: lines.clone(),
            },
        );

        nodes.push(node)
    }

    return nodes;
}

//pub fn compile(tokens: Vec<Token>) -> Vec<u8> { }
