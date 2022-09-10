use maplit::hashmap;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token_type: TokenType::UNKNOWN_TOKEN,
            line: -1,
        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} on line {}", &self.token_type, &self.line)
    }
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
    UNKNOWN_TOKEN,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LEFT_PAREN => write!(f, "(",),
            Self::RIGHT_PAREN => write!(f, ")",),
            Self::LEFT_BRACE => write!(f, "{{",),
            Self::RIGHT_BRACE => write!(f, "}}",),
            Self::COMMA => write!(f, ",",),
            Self::DOT => write!(f, ".",),
            Self::MINUS => write!(f, "-",),
            Self::PLUS => write!(f, "+",),
            Self::SEMICOLON => write!(f, ";",),
            Self::SLASH => write!(f, "/",),
            Self::STAR => write!(f, "*",),
            Self::BANG => write!(f, "!",),
            Self::BANG_EQUAL => write!(f, "!=",),
            Self::EQUAL => write!(f, "=",),
            Self::EQUAL_EQUAL => write!(f, "==",),
            Self::GREATER => write!(f, ">",),
            Self::GREATER_EQUAL => write!(f, ">=",),
            Self::LESS => write!(f, "<",),
            Self::LESS_EQUAL => write!(f, "<=",),
            Self::IDENTIFIER(name) => write!(f, "{}", name),
            Self::STRING(val) => write!(f, "{}", val),
            Self::NUMBER(val) => write!(f, "{}", val),
            Self::AND => write!(f, "&&",),
            Self::CLASS => write!(f, "class",),
            Self::ELSE => write!(f, "else",),
            Self::FALSE => write!(f, "false",),
            Self::FUN => write!(f, "fun",),
            Self::FOR => write!(f, "for",),
            Self::IF => write!(f, "if",),
            Self::NIL => write!(f, "nil",),
            Self::OR => write!(f, "||",),
            Self::PRINT => write!(f, "print",),
            Self::RETURN => write!(f, "return",),
            Self::SUPER => write!(f, "super",),
            Self::THIS => write!(f, "this",),
            Self::TRUE => write!(f, "true",),
            Self::VAR => write!(f, "var",),
            Self::WHILE => write!(f, "while",),
            Self::EOF => write!(f, "<EOF>",),
            Self::UNKNOWN_TOKEN => write!(f, "#######UNKNOWN#######",),
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
