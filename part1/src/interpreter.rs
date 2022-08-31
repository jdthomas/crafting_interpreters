use std::fmt;

use crate::environment::Enviornment;
use crate::parser::{Expr, Stmt};
use crate::tokens::TokenType;

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    Double(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Double(d) => write!(f, "{}", d),
            Self::String(s) => write!(f, "{}", s),
            Self::Nil => write!(f, "Nil"),
        }
    }
}

pub fn truthy(o: Object) -> bool {
    match o {
        Object::Nil => false,
        Object::Boolean(x) => x,
        _ => true,
    }
}

pub struct Interpreter {
    pub env: Enviornment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Enviornment::new(),
        }
    }
    pub fn evaluate_unary(&mut self, t: &TokenType, e: &Expr) -> Object {
        let right = self.evaluate(e);
        match (t, right) {
            (TokenType::MINUS, Object::Double(x)) => Object::Double(-x),
            (TokenType::BANG, o) => Object::Boolean(!truthy(o)),
            _ => panic!("oopsies, bad unary"),
        }
    }

    pub fn evaluate_binary(&mut self, left: &Expr, t: &TokenType, right: &Expr) -> Object {
        let left = self.evaluate(left);
        let right = self.evaluate(right);
        match (left, t, right) {
            (Object::String(l), TokenType::PLUS, Object::String(r)) => {
                Object::String(format!("{}{}", l, r))
            }
            (Object::Double(l), TokenType::PLUS, Object::Double(r)) => Object::Double(l + r),
            (Object::Double(l), TokenType::MINUS, Object::Double(r)) => Object::Double(l - r),
            (Object::Double(l), TokenType::STAR, Object::Double(r)) => Object::Double(l * r),
            (Object::Double(l), TokenType::SLASH, Object::Double(r)) => Object::Double(l / r),

            (Object::Double(l), TokenType::LESS, Object::Double(r)) => Object::Boolean(l < r),
            (Object::Double(l), TokenType::LESS_EQUAL, Object::Double(r)) => {
                Object::Boolean(l <= r)
            }
            (Object::Double(l), TokenType::GREATER, Object::Double(r)) => Object::Boolean(l > r),
            (Object::Double(l), TokenType::GREATER_EQUAL, Object::Double(r)) => {
                Object::Boolean(l >= r)
            }

            (l, TokenType::EQUAL_EQUAL, r) => Object::Boolean(l == r),
            (l, TokenType::BANG_EQUAL, r) => Object::Boolean(l != r),

            (l, t, r) => panic!("Bad binary expr '{:?}' '{}' '{:?}'", l, t, r),
        }
    }

    pub fn evaluate_literal(&mut self, t: &TokenType) -> Object {
        match t {
            TokenType::FALSE => Object::Boolean(false),
            TokenType::TRUE => Object::Boolean(true),
            TokenType::NUMBER(n) => Object::Double(*n),
            TokenType::STRING(s) => Object::String(s.clone()),
            TokenType::NIL => Object::Nil,
            TokenType::EOF => Object::Nil, // ?
            _ => panic!("oopsies, unexpected literal '{:?}'", t),
        }
    }

    pub fn evaluate_group(&mut self, e: &Expr) -> Object {
        self.evaluate(e)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Object {
        match expr {
            Expr::Binary(left, t, right) => self.evaluate_binary(left, t, right),
            Expr::Unary(t, e) => self.evaluate_unary(t, e),
            Expr::Literal(t) => self.evaluate_literal(t),
            Expr::Grouping(s) => self.evaluate_group(s),
            Expr::Variable(n) => {
                if let TokenType::IDENTIFIER(name) = n {
                    self.env.get(name).unwrap()
                } else {
                    Object::Nil
                }
            }
            Expr::Assign(n, v) => {
                let val = self.evaluate(v);
                if let TokenType::IDENTIFIER(name) = n {
                    self.env.assign(name.to_string(), val).unwrap();
                    self.env.get(name).unwrap()
                } else {
                    Object::Nil
                }
            }
        }
    }

    pub fn execute(&mut self, ast: &Stmt) {
        match ast {
            Stmt::Print(e) => println!("{}", self.evaluate(e)),
            Stmt::Expr(e) => {
                let _ = self.evaluate(e);
            }
            Stmt::Var(name, e) => {
                if let Some(expr) = e {
                    let o = self.evaluate(expr);
                    self.env.define(name.clone(), o)
                } else {
                    self.env.define(name.clone(), Object::Nil)
                }
            }
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        statements
            .iter()
            .for_each(|statement| self.execute(statement))
    }
}
