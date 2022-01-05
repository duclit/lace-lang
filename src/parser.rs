use crate::error::{raise, BaseContext, Context};
use crate::lexer::{Token, Value};

pub enum Byte {
    Instruction(u8),
    Address(u32),
}

#[derive(Debug)]
pub enum Node {
    Unary(Value),
    FunctionCall(FunctionCall),
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

pub fn has_operators(tokens: &Vec<&Token>, operators: Vec<&str>) -> bool {
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match token.value {
            Value::LParen | Value::LCurly | Value::LSquare => bracket_stack.push(*token),
            Value::RParen | Value::RCurly | Value::RSquare => {
                bracket_stack.pop();
            }
            _ => {
                for operator in &operators {
                    if token.value == Value::Operator(operator.to_string())
                        && bracket_stack.is_empty()
                    {
                        return true;
                    }
                }
            }
        }
    }

    return false;
}

fn parse_function_arguments(tokens: &Vec<&Token>, context: &BaseContext) -> Vec<Node> {
    return parse_list(tokens, context);
}

fn parse_list(tokens: &Vec<&Token>, context: &BaseContext) -> Vec<Node> {
    let mut elements: Vec<Vec<&Token>> = vec![vec![]];
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match &token.value {
            Value::LParen | Value::LCurly | Value::LSquare => {
                bracket_stack.push(*token);

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
            }
            Value::RParen | Value::RCurly | Value::RSquare => {
                bracket_stack.pop();

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token);
            }
            Value::Operator(op) => {
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
        ret.push(parse_expression(element, context));
    }

    return ret;
}

fn parse_function_call(tokens: &Vec<&Token>, context: &BaseContext) -> Node {
    let mut function_name = String::from("NaN");

    if let Value::Identifier(identifier) = &tokens[0].value {
        function_name = identifier.to_string();
    }

    if tokens[1].value == Value::LParen {
        match tokens[tokens.len() - 1].value {
            Value::RParen => match tokens.len() {
                3 => {
                    return Node::FunctionCall(FunctionCall {
                        name: function_name,
                        args: vec![],
                    })
                }
                _ => {
                    let arguments =
                        parse_function_arguments(&tokens[2..tokens.len() - 1].to_vec(), context);
                    return Node::FunctionCall(FunctionCall {
                        name: function_name,
                        args: arguments,
                    });
                }
            },
            _ => raise(
                "Invalid syntax. Perhaps you forgot a comma?",
                Context {
                    idx: tokens[tokens.len() - 1].line,
                    line: context.source[tokens[tokens.len() - 1].line].clone(),
                    pointer: Option::None,
                },
            ),
        }
    } else {
        raise(
            "Expected parenthesis after function name.",
            Context {
                idx: tokens[1].line,
                line: context.source[tokens[1].line].clone(),
                pointer: Option::None,
            },
        );
    }
}

fn parse_value(tokens: &Vec<&Token>, context: &BaseContext) -> Node {
    match tokens.len() {
        0 => raise(
            "Expected string, int, float, list or function call.",
            Context {
                idx: 0,
                line: context.source[0].clone(),
                pointer: Option::None,
            },
        ),
        _ => match tokens[0].value {
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::FormattedStr(_) => {
                match tokens.len() {
                    1 => return Node::Unary(tokens[0].clone().value),
                    _ => raise(
                        "Expected only one value. Perhaps you forgot a comma?",
                        Context {
                            idx: tokens[1].line,
                            line: context.source[tokens[1].line].clone(),
                            pointer: Option::None,
                        },
                    ),
                }
            }

            Value::Identifier(_) => match tokens.len() {
                1 => return Node::Unary(tokens[0].clone().value),
                2 => {
                    let message: &str;

                    match tokens[1].value {
                        Value::LParen => {
                            message =
                                "Unexpected opening parenthesis. Perhaps you forgot to close them?";
                        }
                        _ => message = "Expected only one value. Perhaps you forgot a comma?",
                    }

                    raise(
                        message,
                        Context {
                            idx: tokens[1].line,
                            line: context.source[tokens[1].line].clone(),
                            pointer: Option::None,
                        },
                    );
                }
                _ => return parse_function_call(tokens, context),
            },
            Value::LSquare => match tokens[tokens.len() - 1].value {
                Value::RSquare => {
                    let list: Vec<Node> =
                        parse_list(&tokens[1..tokens.len() - 1].to_vec(), context);
                    return Node::List(list);
                }
                _ => raise(
                    "Invalid syntax. Perhaps you forgot a comma?",
                    Context {
                        idx: tokens[tokens.len() - 1].line,
                        line: context.source[tokens[tokens.len() - 1].line].clone(),
                        pointer: Option::None,
                    },
                ),
            },
            _ => raise(
                "Expected string, int, float, list or function call.",
                Context {
                    idx: tokens[0].line,
                    line: context.source[tokens[0].line].clone(),
                    pointer: Option::None,
                },
            ),
        },
    }
}

fn parse_mul_div(tokens: Vec<&Token>, context: &BaseContext) -> Node {
    let mut rnode: Node = Node::Unary(Value::Int(0));
    let mut lnode: Node = Node::Unary(Value::Int(0));

    let mut operator: String = String::from(" ");

    for (idx, token) in (&tokens).into_iter().enumerate() {
        match &token.value {
            Value::Operator(op) => {
                if vec!["*", "/", "%", "^", ">>", "<<"].contains(&op.as_str()) {
                    lnode = parse_value(&tokens[0..idx].to_vec(), context);

                    operator = op.to_string();

                    let right = tokens[idx + 1..tokens.len()].to_vec();

                    if has_operators(&right, vec!["*", "/", "%", "^", ">>", "<<"]) {
                        rnode = parse_mul_div(right, context);
                    } else {
                        rnode = parse_value(&right, context)
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

pub fn parse_expression(tokens: Vec<&Token>, context: &BaseContext) -> Node {
    if has_operators(&tokens, vec!["+", "-"]) {
        let mut left_tokens: Vec<&Token> = vec![];
        let mut bracket_stack: Vec<&Token> = vec![];

        let mut operator: String = " ".to_string();

        for (_idx, token) in (&tokens).into_iter().enumerate() {
            if let Value::Operator(ref op) = token.value {
                if (op == "+" || op == "-") && bracket_stack.is_empty() {
                    operator = op.to_string();
                    break;
                } else {
                    left_tokens.push(token);
                }
            } else {
                match &token.value {
                    Value::LCurly | Value::LParen | Value::LSquare => bracket_stack.push(*token),
                    Value::RCurly | Value::RParen | Value::RSquare => {
                        bracket_stack.pop();
                    }
                    _ => {}
                }

                left_tokens.push(token);
            }
        }

        let right_tokens = tokens[left_tokens.len() + 1..tokens.len()].to_vec();
        let left_tokens_len = left_tokens.len();

        let right_node: Node;
        let left_node: Node;

        if has_operators(&left_tokens, vec!["*", "/", "%", "^", "<<", ">>"]) {
            left_node = parse_mul_div(left_tokens, context);
        } else {
            left_node = parse_value(&left_tokens, context);
        }

        if has_operators(&right_tokens, vec!["+", "-"]) {
            right_node = parse_expression(right_tokens, context);
        } else {
            if has_operators(&right_tokens, vec!["*", "/", "%", "^", "<<", ">>"]) {
                right_node = parse_mul_div(right_tokens, context);
            } else {
                right_node = parse_value(&right_tokens, context);
            }
        }

        return Node::Binary(Box::new(BinaryNode {
            a: left_node,
            b: right_node,
            o: operator,
        }));
    } else if has_operators(&tokens, vec!["*", "/", "%", "^", "<<", ">>"]) {
        return parse_mul_div(tokens, context);
    } else {
        return parse_value(&tokens, context);
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
        if line.is_empty() {
            continue;
        }
        let mut ref_line: Vec<&Token> = vec![];

        for token in line {
            ref_line.push(&token)
        }

        let node = parse_expression(
            ref_line,
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
