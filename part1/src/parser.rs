// Lox Grammar
// program        → statement* EOF ;
// statement      → exprStmt
//                | printStmt ;
// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

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
use std::fmt;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Token),
    Grouping(Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(String, Option<Expr>),
    Block(Vec<Stmt>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(left, t, right) => {
                write!(f, "({} {} {})", t, left, right)
            }
            Self::Unary(t, e) => write!(f, "({} {})", t, e),
            Self::Literal(t) => write!(f, "{}", t),
            Self::Grouping(s) => write!(f, "({})", s),
            Self::Variable(n) => {
                write!(f, "{}", n)
            }
            Self::Assign(n, v) => {
                write!(f, "(= {} {})", n, v)
            }
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expr(e) => write!(f, "{}", e),
            Self::Print(e) => write!(f, "{}", e),
            Self::Var(n, Some(e)) => write!(f, "{} = {}", n, e),
            Self::Var(n, None) => write!(f, "{}", n),
            Self::Block(stmts) => write!(f, "{:?}", stmts),
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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        loop {
            let cur_token = self.tokens.peek();
            let cur_token = cur_token.unwrap_or(&&Token {
                token_type: TokenType::EOF,
                line: -1,
            });

            if cur_token.token_type == TokenType::EOF {
                break;
            }
            statements.push(self.declaration());
        }
        statements
    }

    fn token_match(&mut self, t: &[TokenType]) -> Option<&'a Token> {
        let cur_token = self.tokens.peek()?;
        if t.contains(&cur_token.token_type) {
            self.tokens.next()
        } else {
            None
        }
    }

    fn declaration(&mut self) -> Stmt {
        let cur_token = self.tokens.peek().unwrap();
        match cur_token.token_type {
            TokenType::VAR => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Stmt {
        self.token_match(&[TokenType::VAR]); // consume VAR
        let cur_token = self.tokens.peek().unwrap();
        if let TokenType::IDENTIFIER(name) = cur_token.token_type.clone() {
            self.tokens.next();

            let mut initializer: Option<Expr> = None;
            if self.token_match(&[TokenType::EQUAL]).is_some() {
                initializer = Some(self.expression());
            }

            if let Some(_t) = self.token_match(&[TokenType::SEMICOLON]) {
            } else {
                // FIXME: report "Expect ';' after expression."
            }

            Stmt::Var(name, initializer)
        } else {
            // Parse error?
            todo!()
        }
    }

    fn statement(&mut self) -> Stmt {
        let cur_token = self.tokens.peek().unwrap();
        match cur_token.token_type {
            TokenType::PRINT => self.print_statement(),
            TokenType::LEFT_BRACE => self.block(),
            _ => self.expression_statement(),
        }
    }

    fn block(&mut self) -> Stmt {
        self.tokens.next(); // consume LEFT_BRACE
        let mut statements: Vec<Stmt> = vec![];
        loop {
            let cur_token = self.tokens.peek().unwrap();
            if cur_token.token_type == TokenType::RIGHT_BRACE {
                break;
            }
            statements.push(self.declaration());
        }

        self.tokens.next(); // consume(RIGHT_BRACE, "Expect '}' after block.");
        Stmt::Block(statements)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        if let Some(_t) = self.token_match(&[TokenType::SEMICOLON]) {
        } else {
            // FIXME: report "Expect ';' after expression."
        }

        Stmt::Expr(expr)
    }

    fn print_statement(&mut self) -> Stmt {
        self.token_match(&[TokenType::PRINT]);
        let value = self.expression();

        if let Some(_t) = self.token_match(&[TokenType::SEMICOLON]) {
            // Ok
        } else {
            // FIXME: report problem
            // "Expect ';' after value.";
            // self.lox.report(cur_token.line, "", "");
        };
        Stmt::Print(value)
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.equality();

        if self.token_match(&[TokenType::EQUAL]).is_some() {
            // let equals = previous();
            let value = self.assignment();

            if let Expr::Variable(name) = expr {
                return Expr::Assign(name, Box::new(value));
            }

            // error(equals, "Invalid assignment target.");
            panic!("Invalid assignment target.");
        }
        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr: Expr = self.comparison();
        while let Some(operator) =
            self.token_match(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL])
        {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right));
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
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right));
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr: Expr = self.factor();
        while let Some(operator) = self.token_match(&[TokenType::PLUS, TokenType::MINUS]) {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right));
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr: Expr = self.unary();
        while let Some(operator) = self.token_match(&[TokenType::STAR, TokenType::SLASH]) {
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some(operator) = self.token_match(&[TokenType::BANG, TokenType::MINUS]) {
            let right = self.unary();
            Expr::Unary(operator.clone(), Box::new(right))
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
            TokenType::EOF | TokenType::FALSE | TokenType::TRUE | TokenType::NIL => {
                Expr::Literal(cur_token.clone())
            }
            TokenType::STRING(_lit_str_val) => Expr::Literal(cur_token.clone()),
            TokenType::NUMBER(_lit_num_val) => Expr::Literal(cur_token.clone()),

            TokenType::LEFT_PAREN => {
                let expr: Expr = self.expression();
                self.token_match(&[TokenType::RIGHT_PAREN]).or_else(|| {
                    self.lox.report(cur_token.line, "", "");
                    todo!() /*set parse error*/
                });
                Expr::Grouping(Box::new(expr))
            }

            TokenType::IDENTIFIER(_name) => Expr::Variable(cur_token.clone()),

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
