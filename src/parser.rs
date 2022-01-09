use crate::error::{raise, raise_internal, BaseContext, Context};
use crate::lexer::{Token, Value};

use std::collections::HashMap;
use std::mem::discriminant;

#[derive(Debug, Clone)]
pub enum Node {
    Unary(Value),
    FunctionCall(FunctionCall),
    List(Vec<Node>),
    Binary(Box<BinaryNode>),
    Assignment(VariableAssignment),
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
        ret.push(parse_equation(element, context))
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
                Context::new(tokens[tokens.len() - 1].line, &context.source, Option::None),
            ),
        }
    } else {
        raise(
            "Expected parenthesis after function name.",
            Context::new(tokens[1].line, &context.source, Option::None),
        )
    }
}

fn parse_value(tokens: &Vec<&Token>, context: &BaseContext) -> Node {
    match tokens.len() {
        0 => raise(
            "Expected string, int, float, list or function call.",
            Context::new(context.base, &context.source, Option::None),
        ),
        _ => match tokens[0].value {
            Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::FormattedStr(_) => {
                match tokens.len() {
                    1 => Node::Unary(tokens[0].clone().value),
                    _ => raise(
                        "Expected only one value. Perhaps you forgot a comma?",
                        Context::new(tokens[1].line, &context.source, Option::None),
                    ),
                }
            }

            Value::True | Value::False | Value::None => match tokens.len() {
                1 => Node::Unary(tokens[0].clone().value),
                _ => raise(
                    "Expected only one value. Perhaps you forgot a comma?",
                    Context::new(tokens[1].line, &context.source, Option::None),
                ),
            },

            Value::Identifier(_) | Value::MacroIdentifier(_) => match tokens.len() {
                1 => match tokens[0].value {
                    Value::Identifier(_) => Node::Unary(tokens[0].clone().value),
                    Value::MacroIdentifier(_) => raise(
                        "Expected macro call.",
                        Context::new(tokens[0].line, &context.source, Option::None),
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
                        Context::new(tokens[1].line, &context.source, Option::None),
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
                    Context::new(tokens[tokens.len() - 1].line, &context.source, Option::None),
                ),
            },
            _ => raise(
                "Expected string, int, float, list or function call.",
                Context::new(tokens[0].line, &context.source, Option::None),
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

fn parse_equation(tokens: Vec<&Token>, context: &BaseContext) -> Node {
    if has_operators(&tokens, vec!["==", "!=", ">", "<", ">=", "<="]) {
        let mut left_tokens: Vec<&Token> = vec![];
        let mut bracket_stack: Vec<&Token> = vec![];

        let mut operator: String = " ".to_string();

        for (_idx, token) in (&tokens).into_iter().enumerate() {
            if let Value::Operator(ref op) = token.value {
                if (vec!["==", "!=", ">", "<", ">=", "<="].contains(&op.as_str()))
                    && bracket_stack.is_empty()
                {
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

        let right_node: Node;
        let left_node: Node;

        let right_tokens = tokens[left_tokens.len() + 1..tokens.len()].to_vec();

        if has_operators(&left_tokens, vec!["==", "!=", ">", "<", ">=", "<="]) {
            left_node = parse_equation(left_tokens, context);
        } else {
            left_node = parse_expression(left_tokens, context);
        }

        right_node = parse_expression(right_tokens, context);

        Node::Binary(Box::new(BinaryNode {
            a: left_node,
            b: right_node,
            o: operator,
        }))
    } else {
        parse_expression(tokens, context)
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
        return Result::Err(Context::new(
            tokens[tokens.len() - 1].line,
            &context.source,
            Option::None,
        ));
    }

    let token = &tokens[idx + 1];

    if (!exact && discriminant(&token.value) == discriminant(&value))
        || (exact && token.value == value)
    {
        Result::Ok(())
    } else {
        return Result::Err(Context::new(token.line, &context.source, Option::None));
    }
}

fn parse_assignment(tokens: &Vec<Token>, context: &BaseContext) -> Node {
    match expect(tokens, 0, Value::Identifier(String::new()), context, false) {
        Ok(_) => {}
        Err(context) => raise("Expected identifier.", context),
    }

    match expect(tokens, 1, Value::Operator(String::from("=")), context, true) {
        Ok(_) => {}
        Err(context) => match tokens.len() {
            2 => raise("Uninitialized variables are not allowed.", context),
            _ => raise("Expected assignment operator (`=`).", context),
        },
    }

    if tokens.len() >= 4 {
        if let Value::Identifier(identifier) = &tokens[1].value {
            let value = parse_equation(tokens.iter().skip(3).collect::<Vec<&Token>>(), context);

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
            Context::new(tokens[2].line, &context.source, Option::None),
        )
    }
}

fn parse_function_parameters(tokens: &Vec<Token>, context: &BaseContext) -> Vec<String> {
    let mut arguments: Vec<String> = vec![];
    let mut current_arguments: Vec<String> = vec![];

    for token in tokens {
        match &token.value {
            Value::Identifier(identifier) => current_arguments.push(identifier.to_string()),
            Value::Operator(op) => match op.as_str() {
                "," => match current_arguments.len() {
                    0 => raise(
                        "Expected identifier.",
                        Context::new(tokens[1].line, &context.source, Option::None),
                    ),
                    1 => {
                        arguments.push(current_arguments[0].clone());
                        current_arguments = vec![];
                    }
                    _ => raise(
                        "Expected only one identifier.",
                        Context::new(tokens[2].line, &context.source, Option::None),
                    ),
                },
                _ => raise(
                    "Expected identifier or comma.",
                    Context::new(token.line, &context.source, Option::None),
                ),
            },
            _ => raise(
                "Expected identifier or comma.",
                Context::new(token.line, &context.source, Option::None),
            ),
        }
    }

    return arguments;
}

fn parse_function(
    tokens: &Vec<Token>,
    context: &BaseContext,
    source: String,
    file: String,
) -> (Function, usize) {
    match expect(tokens, 0, Value::Identifier(String::new()), context, false) {
        Ok(_) => {}
        Err(context) => raise("Expected function name.", context),
    }

    match expect(tokens, 1, Value::LParen, context, true) {
        Ok(_) => {}
        Err(context) => raise("Expected parameter list.", context),
    }

    let mut parameters: Vec<String> = vec![];
    let body: (Vec<Token>, usize);

    //println!("{:?} {:?}", tokens, tokens[3]);

    match expect(tokens, 2, Value::RParen, context, true) {
        Ok(_) => {
            match expect(tokens, 4, Value::LCurly, context, true) {
                Ok(_) => {}
                Err(context) => raise("Expected opening curly braces", context),
            }

            body = get_block(&tokens, 5, source.to_string());
        }
        Err(_context) => {
            let mut argument_tokens: Vec<Token> = vec![];

            for token in tokens.into_iter().skip(4).collect::<Vec<&Token>>() {
                match token.value {
                    Value::RParen => break,
                    _ => argument_tokens.push(token.clone()),
                }
            }

            parameters = parse_function_parameters(&argument_tokens, context);

            match expect(tokens, 4 + parameters.len(), Value::LCurly, context, true) {
                Ok(_) => {}
                Err(context) => raise("Expected opening curly braces", context),
            }

            body = get_block(&tokens, 6 + parameters.len(), source.to_string());
        }
    }

    if let Value::Identifier(name) = &tokens[1].value {
        let mut function = Function {
            name: name.to_string(),
            args: parameters,
            body: vec![],
            file,
            local_functions: HashMap::new(),
        };

        parse(body.0, source.to_string(), &mut function);

        return (function, body.1 + 3);
    } else {
        raise_internal("0005");
    }
}

fn get_block(tokens: &Vec<Token>, start_i: usize, _source: String) -> (Vec<Token>, usize) {
    let mut block: Vec<Token> = Vec::new();
    let tokens: Vec<&Token> = tokens.iter().skip(start_i).collect();

    //println!("{:?}", tokens);

    let mut bracket_stack: Vec<Value> = Vec::new();

    for token in tokens {
        match token.value {
            Value::LCurly => {
                bracket_stack.push(Value::LCurly);
                block.push(token.clone())
            }
            Value::RCurly => {
                if bracket_stack.is_empty() {
                    break;
                } else {
                    block.push(token.clone());
                    bracket_stack.pop();
                }
            }
            _ => block.push(token.clone()),
        }
    }

    return (block.clone(), block.len());
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

pub fn parse(tokens: Vec<Token>, source: String, chunk: &mut Function) {
    let temp_lines: Vec<&str> = source.split("\n").collect();
    let mut lines: Vec<String> = vec![];

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

                    chunk.body.push(parse_assignment(
                        &line,
                        &BaseContext {
                            base: token.line,
                            source: lines.clone(),
                            tokens: tokens.clone(),
                        },
                    ));
                }
                "fn" => {
                    let (function, skip) = parse_function(
                        &tokens[idx..tokens.len()].to_vec(),
                        &BaseContext {
                            base: token.line,
                            source: lines.clone(),
                            tokens: tokens.clone(),
                        },
                        source.to_string(),
                        chunk.file.to_string(),
                    );

                    for _ in 0..skip {
                        tokens_iter.next();
                    }

                    chunk
                        .local_functions
                        .insert((&function.name).to_string(), function);
                }
                _ => {}
            },
            Value::LSquare
            | Value::Identifier(_)
            | Value::MacroIdentifier(_)
            | Value::True
            | Value::False
            | Value::None
            | Value::Str(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::FormattedStr(_) => {
                let line = get_line(&tokens, idx);
                let line: Vec<&Token> = line.iter().collect();

                for _ in 0..line.len() {
                    tokens_iter.next();
                }

                chunk.body.push(parse_equation(
                    line,
                    &BaseContext {
                        base: token.line,
                        source: lines.clone(),
                        tokens: tokens.clone(),
                    },
                ))
            }
            _ => {}
        }
    }
}
