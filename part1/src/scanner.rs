use crate::lox_error::LoxError;
use crate::tokens::{keywords, Token, TokenType};
use anyhow::Result;
use itertools::peek_nth;

pub fn scan_tokens(lox: &mut dyn LoxError, source: &str) -> Result<Vec<Token>> {
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
                    chars.next();
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                },
                line,
            }),
            '=' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    chars.next();
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                },
                line,
            }),
            '<' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    chars.next();
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                },
                line,
            }),
            '>' => tokens.push(Token {
                token_type: if chars.peek() == Some(&'=') {
                    chars.next();
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
                let value: String = value.into_iter().collect();
                let kw = keywords();
                if let Some(token_type) = kw.get(&value) {
                    tokens.push(Token {
                        token_type: (*token_type).clone(),
                        line,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::IDENTIFIER(value),
                        line,
                    });
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    struct TestLox {
        pub has_error: bool,
    }

    impl LoxError for TestLox {
        fn error(&mut self, line: i32, message: &str) {
            self.report(line, "", message);
        }

        fn report(&mut self, _line: i32, _wh: &str, _message: &str) {
            self.has_error = true;
        }

        fn has_error(&self) -> bool {
            self.has_error
        }
    }

    #[test]
    fn test_empty() {
        let mut lox = TestLox { has_error: false };
        let input = "";
        let expected = vec![Token {
            token_type: TokenType::EOF,
            line: 1,
        }];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_identifier() {
        let mut lox = TestLox { has_error: false };
        let input = "asdf";
        let expected = vec![
            Token {
                token_type: TokenType::IDENTIFIER("asdf".to_string()),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_digit() {
        let mut lox = TestLox { has_error: false };
        let input = "1";
        let expected = vec![
            Token {
                token_type: TokenType::NUMBER(1.0),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_number() {
        let mut lox = TestLox { has_error: false };
        let input = "123.123 321";
        let expected = vec![
            Token {
                token_type: TokenType::NUMBER(123.123),
                line: 1,
            },
            Token {
                token_type: TokenType::NUMBER(321.0),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_simple_string() {
        let mut lox = TestLox { has_error: false };
        let input = "\"asdf\"";
        let expected = vec![
            Token {
                token_type: TokenType::STRING(input[1..input.len() - 1].to_string()),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_string() {
        let mut lox = TestLox { has_error: false };
        let input = "\" asdf\n\t\"";
        let expected = vec![
            Token {
                token_type: TokenType::STRING(input[1..input.len() - 1].to_string()),
                line: 2, // FIXME: Is this what we expect?
            },
            Token {
                token_type: TokenType::EOF,
                line: 2,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_punct() {
        let mut lox = TestLox { has_error: false };
        let input = "(){},.-+;/*";
        let expected = vec![
            Token {
                token_type: TokenType::LEFT_PAREN,
                line: 1,
            },
            Token {
                token_type: TokenType::RIGHT_PAREN,
                line: 1,
            },
            Token {
                token_type: TokenType::LEFT_BRACE,
                line: 1,
            },
            Token {
                token_type: TokenType::RIGHT_BRACE,
                line: 1,
            },
            Token {
                token_type: TokenType::COMMA,
                line: 1,
            },
            Token {
                token_type: TokenType::DOT,
                line: 1,
            },
            Token {
                token_type: TokenType::MINUS,
                line: 1,
            },
            Token {
                token_type: TokenType::PLUS,
                line: 1,
            },
            Token {
                token_type: TokenType::SEMICOLON,
                line: 1,
            },
            Token {
                token_type: TokenType::SLASH,
                line: 1,
            },
            Token {
                token_type: TokenType::STAR,
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_punct2() {
        let mut lox = TestLox { has_error: false };
        let input = "! != = == > >= < <= ";
        let expected = vec![
            Token {
                token_type: TokenType::BANG,
                line: 1,
            },
            Token {
                token_type: TokenType::BANG_EQUAL,
                line: 1,
            },
            Token {
                token_type: TokenType::EQUAL,
                line: 1,
            },
            Token {
                token_type: TokenType::EQUAL_EQUAL,
                line: 1,
            },
            Token {
                token_type: TokenType::GREATER,
                line: 1,
            },
            Token {
                token_type: TokenType::GREATER_EQUAL,
                line: 1,
            },
            Token {
                token_type: TokenType::LESS,
                line: 1,
            },
            Token {
                token_type: TokenType::LESS_EQUAL,
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_keywords() {
        let mut lox = TestLox { has_error: false };
        let input = "and class else false fun for if nil or print return super this true var while";
        let expected = vec![
            Token {
                token_type: TokenType::AND,
                line: 1,
            },
            Token {
                token_type: TokenType::CLASS,
                line: 1,
            },
            Token {
                token_type: TokenType::ELSE,
                line: 1,
            },
            Token {
                token_type: TokenType::FALSE,
                line: 1,
            },
            Token {
                token_type: TokenType::FUN,
                line: 1,
            },
            Token {
                token_type: TokenType::FOR,
                line: 1,
            },
            Token {
                token_type: TokenType::IF,
                line: 1,
            },
            Token {
                token_type: TokenType::NIL,
                line: 1,
            },
            Token {
                token_type: TokenType::OR,
                line: 1,
            },
            Token {
                token_type: TokenType::PRINT,
                line: 1,
            },
            Token {
                token_type: TokenType::RETURN,
                line: 1,
            },
            Token {
                token_type: TokenType::SUPER,
                line: 1,
            },
            Token {
                token_type: TokenType::THIS,
                line: 1,
            },
            Token {
                token_type: TokenType::TRUE,
                line: 1,
            },
            Token {
                token_type: TokenType::VAR,
                line: 1,
            },
            Token {
                token_type: TokenType::WHILE,
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_comment() {
        let mut lox = TestLox { has_error: false };
        let input = "something // comment";
        let expected = vec![
            Token {
                token_type: TokenType::IDENTIFIER("something".to_string()),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                line: 1,
            },
        ];
        let tokens = scan_tokens(&mut lox, input).unwrap();
        assert_eq!(tokens, expected);
        assert_eq!(lox.has_error(), false);
    }

    #[test]
    fn test_unexp_chr() {
        let mut lox = TestLox { has_error: false };
        let input = "[]";
        let tokens = scan_tokens(&mut lox, input).unwrap();
        let expected = vec![Token {
            token_type: TokenType::EOF,
            line: 1,
        }];
        // FIXME: SHould this be an error return?
        assert_eq!(&tokens, &expected);
        assert_eq!(lox.has_error(), true);
    }

    #[test]
    fn test_unterm_string() {
        let mut lox = TestLox { has_error: false };
        let input = "\"asdfa";
        let tokens = scan_tokens(&mut lox, input);
        assert!(tokens.is_err());
        assert_eq!(lox.has_error(), true);
    }
}
