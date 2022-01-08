use crate::error::{raise, raise_internal, BaseContext, Context};
use crate::lexer::{Token, Value};

use std::mem::discriminant;
use std::ops::RangeInclusive;
use std::process::exit;

#[derive(Debug, Clone)]
pub enum Node {
    Unary(Value),
    FunctionCall(FunctionCall),
    List(Vec<Node>),
    Binary(Box<BinaryNode>),
    Assignment(VariableAssignment),
    Function(Function),
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
    pub lfunctions: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct VariableAssignment {
    pub name: String,
    pub value: Box<Node>,
}

fn has_operators(tokens: &Vec<&Token>, operators: Vec<&str>) -> bool {
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
    parse_list(tokens, context)
}

fn parse_list(tokens: &Vec<&Token>, context: &BaseContext) -> Vec<Node> {
    let mut elements: Vec<Vec<&Token>> = vec![vec![]];
    let mut bracket_stack: Vec<&Token> = vec![];

    for token in tokens {
        match &token.value {
            Value::LParen | Value::LCurly | Value::LSquare => {
                bracket_stack.push(*token);

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token)
            }
            Value::RParen | Value::RCurly | Value::RSquare => {
                bracket_stack.pop();

                let arg_len = elements.len() - 1;
                elements[arg_len].push(*token)
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
                elements[arg_len].push(*token)
            }
        }
    }

    let mut ret: Vec<Node> = vec![];

    for element in elements {
        ret.push(parse_expression(element, context))
    }

    ret
}

fn parse_function_call(tokens: &Vec<&Token>, context: &BaseContext, ismacro: bool) -> Node {
    let mut function_name = String::from("NaN");

    if let Value::Identifier(identifier) = &tokens[0].value {
        function_name = identifier.to_string();
    } else if let Value::MacroIdentifier(identifier) = &tokens[0].value {
        function_name = identifier.to_string();
    } else {
        raise_internal("0004");
    }

    if tokens[1].value == Value::LParen {
        match tokens[tokens.len() - 1].value {
            Value::RParen => match tokens.len() {
                3 => Node::FunctionCall(FunctionCall {
                    name: function_name,
                    args: vec![],
                    ismacro,
                }),
                _ => {
                    let arguments =
                        parse_function_arguments(&tokens[2..tokens.len() - 1].to_vec(), context);
                    Node::FunctionCall(FunctionCall {
                        name: function_name,
                        args: arguments,
                        ismacro,
                    })
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
        )
    }
}

fn parse_value(tokens: &Vec<&Token>, context: &BaseContext) -> Node {
    match tokens.len() {
        0 => raise(
            "Expected string, int, float, list or function call.",
            Context {
                idx: context.base,
                line: context.source[context.base].clone(),
                pointer: Option::None,
            },
        ),
        _ => match tokens[0].value {
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::FormattedStr(_) => {
                match tokens.len() {
                    1 => Node::Unary(tokens[0].clone().value),
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

            Value::True | Value::False | Value::None => match tokens.len() {
                1 => Node::Unary(tokens[0].clone().value),
                _ => raise(
                    "Expected only one value. Perhaps you forgot a comma?",
                    Context {
                        idx: tokens[1].line,
                        line: context.source[tokens[1].line].clone(),
                        pointer: Option::None,
                    },
                ),
            },

            Value::Identifier(_) | Value::MacroIdentifier(_) => match tokens.len() {
                1 => match tokens[0].value {
                    Value::Identifier(_) => Node::Unary(tokens[0].clone().value),
                    Value::MacroIdentifier(_) => raise(
                        "Expected macro call.",
                        Context {
                            idx: tokens[0].line,
                            line: context.source[tokens[0].line].clone(),
                            pointer: Option::None,
                        },
                    ),
                    _ => raise_internal("0003"),
                },
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
                _ => parse_function_call(
                    tokens,
                    context,
                    discriminant(&tokens[0].value)
                        == discriminant(&Value::MacroIdentifier("".to_string())),
                ),
            },
            Value::LSquare => match tokens[tokens.len() - 1].value {
                Value::RSquare => {
                    let list: Vec<Node> =
                        parse_list(&tokens[1..tokens.len() - 1].to_vec(), context);
                    Node::List(list)
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

    Node::Binary(Box::new(BinaryNode {
        a: lnode,
        b: rnode,
        o: operator,
    }))
}

fn parse_expression(tokens: Vec<&Token>, context: &BaseContext) -> Node {
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

        Node::Binary(Box::new(BinaryNode {
            a: left_node,
            b: right_node,
            o: operator,
        }))
    } else if has_operators(&tokens, vec!["*", "/", "%", "^", "<<", ">>"]) {
        parse_mul_div(tokens, context)
    } else {
        parse_value(&tokens, context)
    }
}

fn expect(
    tokens: &Vec<Token>,
    idx: usize,
    value: Value,
    context: &BaseContext,
    exact: bool,
) -> Result<(), Context> {
    if tokens.len() < idx + 2 {
        return Result::Err(Context {
            idx: tokens[tokens.len() - 1].line,
            line: context.source[tokens[tokens.len() - 1].line].clone(),
            pointer: Option::None,
        });
    }

    let token = &tokens[idx + 1];

    if (!exact && discriminant(&token.value) == discriminant(&value))
        || (exact && token.value == value)
    {
        Result::Ok(())
    } else {
        return Result::Err(Context {
            idx: token.line,
            line: context.source[token.line].clone(),
            pointer: Option::None,
        });
    }
}

fn parse_assignment(tokens: &Vec<Token>, base_context: &BaseContext) -> Node {
    match expect(
        tokens,
        0,
        Value::Identifier(String::new()),
        base_context,
        false,
    ) {
        Ok(_) => {}
        Err(context) => raise("Expected identifier.", context),
    }

    match expect(
        tokens,
        1,
        Value::Operator(String::from("=")),
        base_context,
        true,
    ) {
        Ok(_) => {}
        Err(context) => match tokens.len() {
            2 => raise("Uninitialized variables are not allowed.", context),
            _ => raise("Expected assignment operator (`=`).", context),
        },
    }

    if tokens.len() >= 4 {
        if let Value::Identifier(identifier) = &tokens[1].value {
            let value =
                parse_expression(tokens.iter().skip(3).collect::<Vec<&Token>>(), base_context);

            Node::Assignment(VariableAssignment {
                name: (*identifier).clone(),
                value: Box::new(value),
            })
        } else {
            raise_internal("0006");
        }
    } else {
        raise(
            "Expected expression.",
            Context {
                idx: tokens[2].line,
                line: base_context.source[tokens[2].line].clone(),
                pointer: Option::None,
            },
        )
    }
}

fn parse_function_parameters(tokens: &Vec<Token>) -> Vec<String> {
    return vec!["".to_string()];
}

fn parse_function(
    tokens: &Vec<Token>,
    base_context: &BaseContext,
    source: String,
) -> (Node, usize) {
    match expect(
        tokens,
        0,
        Value::Identifier(String::new()),
        base_context,
        false,
    ) {
        Ok(_) => {}
        Err(context) => raise("Expected function name.", context),
    }

    match expect(tokens, 1, Value::LParen, base_context, true) {
        Ok(_) => {}
        Err(context) => raise("Expected parameter list.", context),
    }

    match expect(tokens, 2, Value::RParen, base_context, true) {
        Ok(_) => {}
        Err(context) => raise("Expected closing parenthesis.", context),
    }

    match expect(tokens, 3, Value::LCurly, base_context, true) {
        Ok(_) => {}
        Err(context) => raise("Expected opening curly bracket.", context),
    }

    let body = get_block(&tokens, 5, source);

    if let Value::Identifier(name) = &tokens[1].value {
        return (
            Node::Function(Function {
                name: name.to_string(),
                args: vec![],
                body: body.0,
                lfunctions: vec![],
            }),
            body.1 + 3,
        );
    } else {
        raise_internal("0005");
    }
}

fn get_block(tokens: &Vec<Token>, start_i: usize, source: String) -> (Vec<Node>, usize) {
    let mut block: Vec<Token> = Vec::new();
    let tokens = tokens.iter().skip(start_i);

    let mut bracket_stack: Vec<Value> = Vec::new();

    for token in tokens {
        match token.value {
            Value::LCurly => {
                bracket_stack.push(Value::LCurly);
                block.push(token.clone())
            }
            Value::RCurly => {
                if bracket_stack.is_empty() {
                    bracket_stack.push(Value::RCurly);
                    break;
                } else {
                    bracket_stack.push(Value::RCurly);
                    bracket_stack.pop();
                }
            }
            _ => block.push(token.clone()),
        }
    }

    return (parse(block.clone(), source), block.len());
}

fn get_line(tokens: &Vec<Token>, start_i: usize) -> Vec<Token> {
    let mut line: Vec<Token> = Vec::new();

    for token in tokens.iter().skip(start_i) {
        match token.value {
            Value::Semicolon => break,
            _ => line.push(token.clone()),
        }
    }

    line
}

pub fn parse(tokens: Vec<Token>, source: String) -> Vec<Node> {
    let temp_lines: Vec<&str> = source.split("\n").collect();
    let mut lines: Vec<String> = vec![];

    let mut nodes: Vec<Node> = vec![];

    for line in temp_lines {
        lines.push(line.to_string());
    }

    let mut tokens_iter = tokens.iter().enumerate();

    while let Some((idx, token)) = tokens_iter.next() {
        match &token.value {
            Value::Keyword(keyword) => match keyword.as_str() {
                "let" => {
                    let line = get_line(&tokens, idx);

                    for _ in 0..line.len() {
                        tokens_iter.next();
                    }

                    nodes.push(parse_assignment(
                        &line,
                        &BaseContext {
                            base: token.line,
                            source: lines.clone(),
                            tokens: tokens.clone(),
                        },
                    ));
                }
                "fn" => {


                    let (node, skip) = parse_function(
                        &tokens[idx..tokens.len()].to_vec(),
                        &BaseContext {
                            base: token.line,
                            source: lines.clone(),
                            tokens: tokens.clone(),
                        },
                        source.to_string(),
                    );

                    for _ in 0..skip {
                        tokens_iter.next();
                    }

                    nodes.push(node)
                }
                _ => {}
            },
            _ => {}
        }
    }

    println!("{:#?}", nodes);
    nodes
}
