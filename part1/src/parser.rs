use crate::lox_error::LoxError;
use crate::tokens::{Token, TokenType};
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use itertools::Itertools;
use std::fmt;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Token),
    Grouping(Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(String, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(String, Vec<Token>, Box<Stmt>),
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
            Self::Logical(l, o, r) => {
                write!(f, "{} {} {}", l, o.token_type, r)
            }
            Self::Call(callee, args) => {
                write!(f, "{} {:?}", callee, args)
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
            Self::If(c, t, e) => write!(f, "{} {} {:?}", c, t, e),
            Self::While(c, s) => write!(f, "{} {}", c, s),
            Self::Function(n, p, b) => write!(f, "{} {:?} {} ", n, p, b),
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
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
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn token_match(&mut self, t: &[TokenType]) -> Option<&'a Token> {
        let cur_token = self.tokens.peek()?;
        if t.contains(&cur_token.token_type) {
            self.tokens.next()
        } else {
            None
        }
    }

    fn consume(&mut self, t: TokenType, msg: &str) -> Result<&Token> {
        self.token_match(&[t]).ok_or_else(|| {
            self.lox.report(self.tokens.peek().unwrap().line, "", msg);
            anyhow!("{}", msg)
        })
    }
    fn consume_identifier(&mut self, msg: &str) -> Result<String> {
        let cur_token = self.tokens.peek().unwrap();
        if let TokenType::IDENTIFIER(name) = cur_token.token_type.clone() {
            self.tokens.next();
            Ok(name)
        } else {
            self.lox.report(cur_token.line, "", msg);
            Err(anyhow!(""))
        }
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let cur_token = self.tokens.peek().unwrap();
        match cur_token.token_type {
            TokenType::VAR => self.var_declaration(),
            TokenType::FUN => self.fun_declaration(),
            _ => self.statement(),
        }
    }

    fn fun_declaration(&mut self) -> Result<Stmt> {
        let kind = "function";
        self.tokens.next(); // skip FUN
        let name = self.consume_identifier(&format!("Expect {} name.", kind))?;
        let _ = self.consume(
            TokenType::LEFT_PAREN,
            &format!("Expect '(' after {} name", kind),
        );
        let parameters: Result<Vec<Token>> = self
            .tokens
            .take_while(|token| token.token_type != TokenType::RIGHT_PAREN)
            .chain(&[Token {
                token_type: TokenType::COMMA,
                line: 0,
            }])
            .tuples::<(_, _)>()
            .map(|(name, comma)| -> Result<Token> {
                // println!("T1: {} t2: {}", name, comma);
                match (&name.token_type, &comma.token_type) {
                    (TokenType::IDENTIFIER(_), TokenType::COMMA) => Ok(name.clone()),
                    (_, _) => Err(anyhow!("Expected identifier, comma pairs")),
                }
            })
            .into_iter()
            .collect();
        let parameters = parameters?;
        // FIXME: the take_while ate our paren, should find a way to report that error
        // let _ = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after paramaters");
        let cur_token = self.tokens.peek().unwrap();
        let body = match cur_token.token_type {
            TokenType::LEFT_BRACE => self.block(),
            _ => todo!(),
        }?;

        Ok(Stmt::Function(name, parameters, Box::new(body)))
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
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

            Ok(Stmt::Var(name, initializer))
        } else {
            // Parse error?
            todo!()
        }
    }

    fn statement(&mut self) -> Result<Stmt> {
        let cur_token = self.tokens.peek().unwrap();
        match cur_token.token_type {
            TokenType::PRINT => self.print_statement(),
            TokenType::WHILE => self.while_statement(),
            TokenType::FOR => self.for_statement(),
            TokenType::IF => self.if_statement(),
            TokenType::LEFT_BRACE => self.block(),
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.tokens.next(); // consume FOR
        if self.token_match(&[TokenType::LEFT_PAREN]).is_none() {
            // FIXME:
            return Err(anyhow!("expected lparen"));
        }
        let cur_token = self.tokens.peek().unwrap();
        let initilizer = if cur_token.token_type == TokenType::SEMICOLON {
            self.tokens.next();
            None
        } else if cur_token.token_type == TokenType::VAR {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let cur_token = self.tokens.peek().unwrap();

        let condition = if cur_token.token_type == TokenType::SEMICOLON {
            None
        } else {
            Some(self.expression())
        };

        if self.token_match(&[TokenType::SEMICOLON]).is_none() {
            // FIXME:
            return Err(anyhow!("Expect ';' after loop condition."));
        }

        let cur_token = self.tokens.peek().unwrap();
        let increment = if cur_token.token_type == TokenType::RIGHT_PAREN {
            None
        } else {
            Some(self.expression())
        };

        if self.token_match(&[TokenType::RIGHT_PAREN]).is_none() {
            // FIXME:
            return Err(anyhow!("Expect ')' after for clauses."));
        }

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(increment)])
        }
        if let Some(condition) = condition {
            body = Stmt::While(condition, Box::new(body));
        }
        if let Some(initilizer) = initilizer {
            body = Stmt::Block(vec![initilizer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.tokens.next(); // consume WHILE
        if self.token_match(&[TokenType::LEFT_PAREN]).is_none() {
            // FIXME:
            return Err(anyhow!("expected lparen"));
        }
        let condition = self.expression();
        if self.token_match(&[TokenType::RIGHT_PAREN]).is_none() {
            // FIXME:
            return Err(anyhow!("expected rparen"));
        }
        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.tokens.next(); // consume IF

        let cur_token = self.tokens.peek().unwrap();
        if cur_token.token_type == TokenType::LEFT_PAREN {
            self.tokens.next(); // consume '('

            let condition = self.expression();

            let cur_token = self.tokens.peek().unwrap();
            if cur_token.token_type != TokenType::RIGHT_PAREN {
                return Err(anyhow!("expected right paren"));
            }
            self.tokens.next(); // consume ')'

            let then_branch = self.statement()?;
            let else_branch = if let Some(cur_token) = self.tokens.peek() {
                if cur_token.token_type == TokenType::ELSE {
                    self.tokens.next(); // consume ELSE
                    Some(Box::new(self.statement()?))
                } else {
                    None
                }
            } else {
                None
            };
            return Ok(Stmt::If(condition, Box::new(then_branch), else_branch));
        }
        // FIXME: Parse error
        todo!()
    }

    fn block(&mut self) -> Result<Stmt> {
        self.tokens.next(); // consume LEFT_BRACE
        let mut statements: Vec<Stmt> = vec![];
        loop {
            let cur_token = self.tokens.peek().unwrap();
            if cur_token.token_type == TokenType::RIGHT_BRACE {
                break;
            }
            statements.push(self.declaration()?);
        }

        self.tokens.next(); // consume(RIGHT_BRACE, "Expect '}' after block.");
        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression();
        if let Some(_t) = self.token_match(&[TokenType::SEMICOLON]) {
        } else {
            // FIXME: report "Expect ';' after expression."
        }

        Ok(Stmt::Expr(expr))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        self.token_match(&[TokenType::PRINT]);
        let value = self.expression();

        if let Some(_t) = self.token_match(&[TokenType::SEMICOLON]) {
            // Ok
        } else {
            // FIXME: report problem
            // "Expect ';' after value.";
            // self.lox.report(cur_token.line, "", "");
        };
        Ok(Stmt::Print(value))
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn or_expr(&mut self) -> Expr {
        let mut expr = self.and_expr();

        while let Some(operator) = self.token_match(&[TokenType::OR]) {
            let right = self.and_expr();
            expr = Expr::Logical(Box::new(expr), operator.clone(), Box::new(right));
        }
        expr
    }

    fn and_expr(&mut self) -> Expr {
        let mut expr = self.equality();

        while let Some(operator) = self.token_match(&[TokenType::AND]) {
            let right = self.equality();
            expr = Expr::Logical(Box::new(expr), operator.clone(), Box::new(right));
        }
        expr
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or_expr();

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
            self.call()
        }
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        while let Some(_operator) = self.token_match(&[TokenType::LEFT_PAREN]) {
            expr = self.finish_call(expr);
        }
        expr
    }
    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments: Vec<Expr> = vec![];
        if let Some(_operator) = self.token_match(&[TokenType::RIGHT_PAREN]) {
        } else {
            loop {
                arguments.push(self.expression());
                if let Some(_operator) = self.token_match(&[TokenType::COMMA]) {
                } else {
                    if self.token_match(&[TokenType::RIGHT_PAREN]).is_none() {
                        // "Expect ')' after arguments."
                        todo!();
                    }
                    break;
                }
            }
        }

        Expr::Call(Box::new(callee), arguments)
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
