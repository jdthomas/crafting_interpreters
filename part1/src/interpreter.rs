use std::fmt;
use std::fmt::{Debug, Display};

use crate::environment::Enviornment;
use crate::parser::{Expr, Stmt};
use crate::tokens::{Token, TokenType};
use anyhow::Result;
use anyhow::{anyhow, Context};

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    Double(f64),
    String(String),
}

#[derive(Debug)]
pub struct LoxRuntimeError {
    t: Token,
}
impl Display for LoxRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.t)
    }
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
    pub fn evaluate_unary(&mut self, t: &Token, e: &Expr) -> Result<Object> {
        let right = self.evaluate(e)?;
        match (&t.token_type, right) {
            (TokenType::MINUS, Object::Double(x)) => Ok(Object::Double(-x)),
            (TokenType::BANG, o) => Ok(Object::Boolean(!truthy(o))),
            _ => Err(anyhow!("oopsies, bad unary")).context(LoxRuntimeError { t: t.clone() }),
        }
    }

    pub fn evaluate_binary(&mut self, left: &Expr, t: &Token, right: &Expr) -> Result<Object> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match (left, &t.token_type, right) {
            (Object::String(l), TokenType::PLUS, Object::String(r)) => {
                Ok(Object::String(format!("{}{}", l, r)))
            }
            (Object::Double(l), TokenType::PLUS, Object::Double(r)) => Ok(Object::Double(l + r)),
            (Object::Double(l), TokenType::MINUS, Object::Double(r)) => Ok(Object::Double(l - r)),
            (Object::Double(l), TokenType::STAR, Object::Double(r)) => Ok(Object::Double(l * r)),
            (Object::Double(l), TokenType::SLASH, Object::Double(r)) => Ok(Object::Double(l / r)),

            (Object::Double(l), TokenType::LESS, Object::Double(r)) => Ok(Object::Boolean(l < r)),
            (Object::Double(l), TokenType::LESS_EQUAL, Object::Double(r)) => {
                Ok(Object::Boolean(l <= r))
            }
            (Object::Double(l), TokenType::GREATER, Object::Double(r)) => {
                Ok(Object::Boolean(l > r))
            }
            (Object::Double(l), TokenType::GREATER_EQUAL, Object::Double(r)) => {
                Ok(Object::Boolean(l >= r))
            }

            (l, TokenType::EQUAL_EQUAL, r) => Ok(Object::Boolean(l == r)),
            (l, TokenType::BANG_EQUAL, r) => Ok(Object::Boolean(l != r)),

            (l, tt, r) => Err(anyhow!("Bad binary expr '{:?}' '{}' '{:?}'", l, tt, r))
                .context(LoxRuntimeError { t: t.clone() }),
        }
    }

    pub fn evaluate_literal(&mut self, t: &Token) -> Result<Object> {
        match &t.token_type {
            TokenType::FALSE => Ok(Object::Boolean(false)),
            TokenType::TRUE => Ok(Object::Boolean(true)),
            TokenType::NUMBER(n) => Ok(Object::Double(*n)),
            TokenType::STRING(s) => Ok(Object::String(s.clone())),
            TokenType::NIL => Ok(Object::Nil),
            TokenType::EOF => Ok(Object::Nil), // ?
            _ => Err(anyhow!("oopsies, unexpected literal '{:?}'", t.token_type))
                .context(LoxRuntimeError { t: t.clone() }),
        }
    }

    pub fn evaluate_group(&mut self, e: &Expr) -> Result<Object> {
        self.evaluate(e)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Binary(left, t, right) => self.evaluate_binary(left, t, right),
            Expr::Unary(t, e) => self.evaluate_unary(t, e),
            Expr::Literal(t) => self.evaluate_literal(t),
            Expr::Grouping(s) => self.evaluate_group(s),
            Expr::Variable(n) => {
                if let TokenType::IDENTIFIER(name) = &n.token_type {
                    // FIXME: handle unseen symbol WRT unwarp
                    self.env.get(name).context(LoxRuntimeError { t: n.clone() })
                } else {
                    Ok(Object::Nil)
                }
            }
            Expr::Assign(n, v) => {
                let val = self.evaluate(v)?;
                if let TokenType::IDENTIFIER(name) = &n.token_type {
                    self.env
                        .assign(name.to_string(), val)
                        .context(LoxRuntimeError { t: n.clone() })?;
                    self.env.get(name)
                } else {
                    Ok(Object::Nil)
                }
            }
        }
    }

    pub fn execute(&mut self, ast: &Stmt) -> Result<()> {
        match ast {
            Stmt::Print(e) => println!("{}", self.evaluate(e)?),
            Stmt::Expr(e) => {
                let _ = self.evaluate(e)?;
            }
            Stmt::Var(name, e) => {
                if let Some(expr) = e {
                    let o = self.evaluate(expr)?;
                    self.env.define(name.clone(), o)
                } else {
                    self.env.define(name.clone(), Object::Nil)
                }
            }
            Stmt::Block(stmts) => {
                self.env.push_scope();
                stmts.iter().for_each(|s| {
                    self.execute(s);
                });
                self.env.pop_scope();
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        statements
            .iter()
            .map(|statement| self.execute(statement))
            .into_iter()
            .collect()
    }
}
