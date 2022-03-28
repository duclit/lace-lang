use logos::Logos;

pub fn lace_pipeline_init(source: &str) {
    let scanner = crate::scanner::Token::lexer(source);
    let mut parser = crate::parser::Parser::new(scanner, source.to_string());
    parser.parse();

    println!("{:?}", parser.ast)
}
