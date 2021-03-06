use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Brackets
    #[token("{")]
    LeftCurly,
    #[token("}")]
    RightCurly,

    #[token("[")]
    LeftSquare,
    #[token("]")]
    RightSquare,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,

    // Keywords
    #[token("as")]
    KwAs,
    #[token("let")]
    KwLet,
    #[token("mut")]
    KwMut,
    #[token("pub")]
    KwPub,
    #[token("type")]
    KwType,
    #[token("typeof")]
    KwTypeof,
    #[token("return")]
    KwReturn,
    #[token("fn")]
    KwFn,
    #[token("async")]
    KwAsync,
    #[token("and")]
    KwAnd,
    #[token("or")]
    KwOr,
    #[token("while")]
    KwWhile,
    #[token("use")]
    KwUse,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("new")]
    KwNew,

    // Builtin Values
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("none")]
    None,

    // Operators
    #[token("!=")]
    OpBangEq,
    #[token("==")]
    OpEq,
    #[token(">=")]
    OpMoreEq,
    #[token("<=")]
    OpLessEq,
    #[token(">>")]
    OpRightShift,
    #[token("<<")]
    OpLeftShift,
    #[token(">")]
    OpMore,
    #[token("<")]
    OpLess,
    #[token("+")]
    OpAdd,
    #[token("-")]
    OpSub,
    #[token("*")]
    OpMul,
    #[token("/")]
    OpDiv,
    #[token("%")]
    OpMod,
    #[token("**")]
    OpPow,
    #[token("^")]
    BitwiseXor,
    #[token("|")]
    BitwiseOr,
    #[token("&")]
    BitwiseAnd,
    #[token("=")]
    Assign,
    #[token("!")]
    OpBang,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("?")]
    Que,
    #[token(".")]
    Period,

    // Literals
    // #[regex(r#"0b([0-9]+)"#, |lex|lex .slice().parse())]
    // Byte(i8),
    #[regex(r#"([0-9]*\.[0-9]+)|([0-9]*)"#, |lex| lex.slice().parse())]
    Number(f64),
    #[regex("\"([^\"]*)\"", |lex| lex.slice().to_string())]
    #[regex("'([^\"]*)'", |lex| lex.slice().to_string())]
    String(String),
    #[regex("`([^\"]*)`", |lex| lex.slice().to_string())]
    FormattedString(String),
    #[regex("[a-zA-Z_]+!", |lex| lex.slice().to_string())]
    PrimitiveFnIdentifier(String),
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[error]
    #[regex(r"[\s\t\n\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    Error,

    End,
}
