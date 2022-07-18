use crate::lox::Lox;
use crate::tokens::{Token, TokenType};
use anyhow::Result;
use itertools::peek_nth;

pub fn scan_tokens(lox: &mut Lox, source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut chars = peek_nth(source.chars());

    while let Some(c) = chars.next() {
        match c {
            // Ignore white space
            ' ' | '\t' | '\r' => {}
            '\n' => line += 1,
            // Single-character tokens.
            '(' => tokens.push(Token {
                token_type: TokenType::LEFT_PAREN,
                line,
            }),
            ')' => tokens.push(Token {
                token_type: TokenType::RIGHT_PAREN,
                line,
            }),
            '{' => tokens.push(Token {
                token_type: TokenType::LEFT_BRACE,
                line,
            }),
            '}' => tokens.push(Token {
                token_type: TokenType::RIGHT_BRACE,
                line,
            }),
            ',' => tokens.push(Token {
                token_type: TokenType::COMMA,
                line,
            }),
            '.' => tokens.push(Token {
                token_type: TokenType::DOT,
                line,
            }),
            '-' => tokens.push(Token {
                token_type: TokenType::MINUS,
                line,
            }),
            '+' => tokens.push(Token {
                token_type: TokenType::PLUS,
                line,
            }),
            ';' => tokens.push(Token {
                token_type: TokenType::SEMICOLON,
                line,
            }),
            '*' => tokens.push(Token {
                token_type: TokenType::STAR,
                line,
            }),
            // One or two character tokens.
            '!' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                },
                line,
            }),
            '=' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                },
                line,
            }),
            '<' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                },
                line,
            }),
            '>' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                },
                line,
            }),
            // SLASH or comment
            '/' => {
                if chars.peek() == Some(&'/') {
                    while chars.peek() != Some(&'\n') && chars.peek().is_some() {
                        let _ = chars.next();
                    }
                } else {
                    tokens.push(Token {
                        token_type: TokenType::SLASH,
                        line,
                    });
                }
            }
            // String Literal
            '"' => {
                let mut value = Vec::new();
                while chars.peek().is_some() && chars.peek() != Some(&'"') {
                    let x = chars.next();
                    value.push(x.unwrap());
                    if x == Some('\n') {
                        line += 1;
                    }
                }
                let x = chars.next();
                if x.is_none() {
                    lox.error(line, "Unterminated string.");
                    return Err(anyhow::anyhow!("Unterminated string."));
                }
                tokens.push(Token {
                    token_type: TokenType::STRING(value.into_iter().collect()),
                    line,
                });
            }
            // Number literal
            '0'..='9' => {
                let mut value = Vec::new();
                value.push(c);
                while chars.peek().is_some() && chars.peek().unwrap().is_ascii_digit() {
                    let x = chars.next().unwrap();
                    value.push(x);
                }
                if chars.peek() == Some(&'.')
                    && chars.peek_nth(1).is_some()
                    && chars.peek_nth(1).unwrap().is_ascii_digit()
                {
                    let x = chars.next().unwrap();
                    value.push(x);
                    while chars.peek().is_some() && chars.peek().unwrap().is_ascii_digit() {
                        let x = chars.next().unwrap();
                        value.push(x);
                    }
                }
                let string_value: String = value.into_iter().collect();
                let value: f64 = string_value.parse::<f64>().unwrap();
                tokens.push(Token {
                    token_type: TokenType::NUMBER(value),
                    line,
                });
            }
            // Idnetifier
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut value = Vec::new();
                value.push(c);
                while chars.peek().is_some()
                    && (chars.peek().unwrap().is_ascii_alphabetic() || chars.peek() == Some(&'_'))
                {
                    value.push(chars.next().unwrap());
                }
                tokens.push(Token {
                    token_type: TokenType::IDENTIFIER(value.into_iter().collect()),
                    line,
                });
            }
            c => {
                lox.error(line, &format!("Unexpected character {:?}.", c));
                // return Err(anyhow::anyhow!("oops"));
            }
        }
    }

    tokens.push(Token {
        token_type: TokenType::EOF,
        line,
    });
    Ok(tokens)
}
