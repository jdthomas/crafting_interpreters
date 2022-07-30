use maplit::hashmap;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER(String),
    STRING(String),
    NUMBER(f64),

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,

    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

pub fn keywords() -> HashMap<String, TokenType> {
    hashmap! {
        "and".to_owned() => TokenType::AND,
        "class".to_owned() => TokenType::CLASS,
        "else".to_owned() => TokenType::ELSE,
        "false".to_owned() => TokenType::FALSE,
        "fun".to_owned() => TokenType::FUN,
        "for".to_owned() => TokenType::FOR,
        "if".to_owned() => TokenType::IF,
        "nil".to_owned() => TokenType::NIL,
        "or".to_owned() => TokenType::OR,

        "print".to_owned() => TokenType::PRINT,
        "return".to_owned() => TokenType::RETURN,
        "super".to_owned() => TokenType::SUPER,
        "this".to_owned() => TokenType::THIS,
        "true".to_owned() => TokenType::TRUE,
        "var".to_owned() => TokenType::VAR,
        "while".to_owned() => TokenType::WHILE,
    }
}
