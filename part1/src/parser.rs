// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

use crate::lox_error::LoxError;
use crate::tokens::{Token, TokenType};
use itertools::Itertools;
use std::fmt;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Unary(TokenType, Box<Expr>),
    Literal(TokenType),
    Grouping(Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(left, t, right) => {
                write!(f, "({} {} {})", t, left.to_string(), right.to_string())
            }
            Self::Unary(t, e) => write!(f, "({} {})", t, e.to_string()),
            Self::Literal(t) => write!(f, "({})", t),
            Self::Grouping(s) => write!(f, "({})", s.iter().map(|x| x.to_string()).join(",")),
        }
    }
}

type Tokenz<'a> = &'a mut Peekable<Iter<'a, Token>>;
pub struct Parser<'a> {
    tokens: Tokenz<'a>,
    lox: &'a mut dyn LoxError,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Tokenz<'a>, lox: &'a mut dyn LoxError) -> Self {
        Self { tokens, lox }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    fn token_match(&mut self, t: &[TokenType]) -> Option<&'a Token> {
        let cur_token = self.tokens.peek()?;
        if t.contains(&cur_token.token_type) {
            self.tokens.next()
        } else {
            None
        }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr: Expr = self.comparison();
        while let Some(operator) =
            self.token_match(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL])
        {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.token_type.clone(), Box::new(right));
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr: Expr = self.term();
        while let Some(operator) = self.token_match(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.token_type.clone(), Box::new(right));
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr: Expr = self.factor();
        while let Some(operator) = self.token_match(&[TokenType::PLUS, TokenType::MINUS]) {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.token_type.clone(), Box::new(right));
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr: Expr = self.unary();
        while let Some(operator) = self.token_match(&[TokenType::STAR, TokenType::SLASH]) {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.token_type.clone(), Box::new(right));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some(operator) = self.token_match(&[TokenType::BANG, TokenType::MINUS]) {
            let right = self.unary();
            Expr::Unary(operator.token_type.clone(), Box::new(right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        let cur_token = self.tokens.next();
        let cur_token = cur_token.unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: -1,
        });
        match &cur_token.token_type {
            TokenType::FALSE => Expr::Literal(TokenType::FALSE),
            TokenType::TRUE => Expr::Literal(TokenType::TRUE),
            TokenType::NIL => Expr::Literal(TokenType::NIL),

            TokenType::STRING(lit_val) => Expr::Literal(TokenType::STRING(lit_val.clone())),
            TokenType::NUMBER(lit_val) => Expr::Literal(TokenType::NUMBER(*lit_val)),

            TokenType::LEFT_PAREN => {
                let expr: Expr = self.expression();
                self.token_match(&[TokenType::RIGHT_PAREN]).or_else(|| {
                    self.lox.report(cur_token.line, "", "");
                    todo!() /*set parse error*/
                });
                expr
            }

            TokenType::EOF => Expr::Literal(TokenType::EOF),

            _ => {
                // TODO: Report error
                //panic!("whoopsie, unexpected {:?}", cur_token),
                self.synchronize();
                self.expression()
            }
        }
    }

    fn synchronize(&mut self) {
        while let Some(cur_token) = self.tokens.next() {
            match cur_token.token_type {
                TokenType::SEMICOLON => {
                    self.tokens.next();
                    return;
                }
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,

                _ => {}
            }
        }
    }
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
        let tokens = vec![];
        let tokz = &mut tokens.iter().peekable();
        let mut parser = Parser::new(tokz, &mut lox);
        let ast = parser.parse();
        println!("{:#?}", ast);
    }
    #[test]
    fn test_simple() {
        let mut lox = TestLox { has_error: false };
        let tokens = vec![
            Token {
                token_type: TokenType::LEFT_PAREN,
                line: 1,
            },
            Token {
                token_type: TokenType::NUMBER(42.0),
                line: 1,
            },
            Token {
                token_type: TokenType::RIGHT_PAREN,
                line: 1,
            },
        ];
        let tokz = &mut tokens.iter().peekable();

        let mut parser = Parser::new(tokz, &mut lox);
        let ast = parser.parse();
        println!("{:#?}", ast);
    }
}
