use maplit::hashmap;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
}

#[derive(Debug, PartialEq)]
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

impl Clone for TokenType {
    // FIXME: Can we not derive this??
    fn clone(&self) -> TokenType {
        match self {
            TokenType::IDENTIFIER(s) => TokenType::IDENTIFIER(s.to_string()),
            TokenType::STRING(s) => TokenType::STRING(s.to_string()),
            TokenType::NUMBER(s) => TokenType::NUMBER(*s),
            TokenType::LEFT_PAREN => TokenType::LEFT_PAREN,
            TokenType::RIGHT_PAREN => TokenType::RIGHT_PAREN,
            TokenType::LEFT_BRACE => TokenType::LEFT_BRACE,
            TokenType::RIGHT_BRACE => TokenType::RIGHT_BRACE,
            TokenType::COMMA => TokenType::COMMA,
            TokenType::DOT => TokenType::DOT,
            TokenType::MINUS => TokenType::MINUS,
            TokenType::PLUS => TokenType::PLUS,
            TokenType::SEMICOLON => TokenType::SEMICOLON,
            TokenType::SLASH => TokenType::SLASH,
            TokenType::STAR => TokenType::STAR,
            TokenType::BANG => TokenType::BANG,
            TokenType::BANG_EQUAL => TokenType::BANG_EQUAL,
            TokenType::EQUAL => TokenType::EQUAL,
            TokenType::EQUAL_EQUAL => TokenType::EQUAL_EQUAL,
            TokenType::GREATER => TokenType::GREATER,
            TokenType::GREATER_EQUAL => TokenType::GREATER_EQUAL,
            TokenType::LESS => TokenType::LESS,
            TokenType::LESS_EQUAL => TokenType::LESS_EQUAL,
            TokenType::AND => TokenType::AND,
            TokenType::CLASS => TokenType::CLASS,
            TokenType::ELSE => TokenType::ELSE,
            TokenType::FALSE => TokenType::FALSE,
            TokenType::FUN => TokenType::FUN,
            TokenType::FOR => TokenType::FOR,
            TokenType::IF => TokenType::IF,
            TokenType::NIL => TokenType::NIL,
            TokenType::OR => TokenType::OR,
            TokenType::PRINT => TokenType::PRINT,
            TokenType::RETURN => TokenType::RETURN,
            TokenType::SUPER => TokenType::SUPER,
            TokenType::THIS => TokenType::THIS,
            TokenType::TRUE => TokenType::TRUE,
            TokenType::VAR => TokenType::VAR,
            TokenType::WHILE => TokenType::WHILE,
            TokenType::EOF => TokenType::EOF,
        }
    }
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
