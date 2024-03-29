use crate::environment::Enviornment;
use crate::parser::{Expr, Stmt};
use crate::tokens::{Token, TokenType};
use anyhow::Result;
use anyhow::{anyhow, Context};
use std::fmt;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use take_until::TakeUntilExt;

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Nil,
    Boolean(bool),
    Double(f64),
    String(String),
    Callable(LoxCallableWrapper),
    Return(Box<Object>),
}

// This wrapper is just here so I can get around being able to derive PartialEq on the enum while ignoring (always false) Callables
#[derive(Debug, Clone)]
pub struct LoxCallableWrapper {
    inner: Rc<dyn LoxCallable>,
}
impl LoxCallable for LoxCallableWrapper {
    fn call(&self, i: &mut Interpreter, args: Vec<Object>) -> Object {
        self.inner.call(i, args)
    }
}

pub trait LoxCallable: Debug {
    fn call(&self, i: &mut Interpreter, args: Vec<Object>) -> Object;
}
impl PartialEq for LoxCallableWrapper {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug)]
struct LoxFunction {
    params: Vec<Token>,
    body: Stmt,
}
fn identifier_name(t: &Token) -> Option<String> {
    match &t.token_type {
        TokenType::IDENTIFIER(name) => Some(name.clone()),
        _ => None,
    }
}
impl LoxCallable for LoxFunction {
    fn call(&self, i: &mut Interpreter, args: Vec<Object>) -> Object {
        i.env.push_scope();
        // FIXME: Verify params/args lengths match
        itertools::zip(&self.params, &args).for_each(|(p, a)| {
            i.env.define(
                identifier_name(p).unwrap_or("FIXME: Something has gone wrong :P ".to_string()),
                a.clone(),
            )
        });
        let res = i.execute(&self.body);
        i.env.pop_scope();

        match res {
            Ok(StmtResult::Return(r)) => r,
            _ => Object::Nil,
        }
    }
}

#[derive(Debug)]
struct LoxBuiltinClock {}
impl LoxCallable for LoxBuiltinClock {
    fn call(&self, _i: &mut Interpreter, _args: Vec<Object>) -> Object {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time");
        Object::Double(now.as_secs_f64())
    }
}

#[derive(Debug, Clone)]
enum StmtResult {
    Noop,
    Return(Object),
}

#[derive(Debug)]
pub struct LoxRuntimeError {
    t: Token,
    message: String,
}
impl Display for LoxRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.\n[line {}]", self.message, self.t.line)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Double(d) => write!(f, "{}", d),
            Self::String(s) => write!(f, "{}", s),
            Self::Callable(_s) => write!(f, "...calable..."),
            Self::Return(o) => write!(f, "...returning {}...", o),
            Self::Nil => write!(f, "nil"),
        }
    }
}

pub fn truthy(o: &Object) -> bool {
    match o {
        Object::Nil => false,
        Object::Boolean(x) => *x,
        _ => true,
    }
}

pub struct Interpreter<'a> {
    pub env: &'a mut Enviornment,
}

impl<'a> Interpreter<'a> {
    // pub fn new() -> Self {
    //     Interpreter {
    //         env: &mut Enviornment::new(),
    //     }
    // }
    pub fn new_with_env(env: &'a mut Enviornment) -> Self {
        env.define(
            "clock".to_owned(),
            Object::Callable(LoxCallableWrapper {
                inner: Rc::new(LoxBuiltinClock {}),
            }),
        );
        Interpreter { env }
    }
    pub fn evaluate_unary(&mut self, t: &Token, e: &Expr) -> Result<Object> {
        let right = self.evaluate(e)?;
        match (&t.token_type, right) {
            (TokenType::MINUS, Object::Double(x)) => Ok(Object::Double(-x)),
            (TokenType::BANG, o) => Ok(Object::Boolean(!truthy(&o))),
            _ => Err(anyhow!("oopsies, bad unary")).context(LoxRuntimeError {
                t: t.clone(),
                message: "".to_owned(),
            }),
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

            (l, tt, r) => Err(anyhow!("Bad binary expr '{:?}' '{}' '{:?}'", l, tt, r)).context(
                LoxRuntimeError {
                    t: t.clone(),
                    message: "Bad binary expr".to_owned(),
                },
            ),
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
            _ => Err(anyhow!("oopsies, unexpected literal '{:?}'", t.token_type)).context(
                LoxRuntimeError {
                    t: t.clone(),
                    message: format!("unexpected literal '{:?}'", t.token_type),
                },
            ),
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
                    self.env.get(name).context(LoxRuntimeError {
                        t: n.clone(),
                        message: format!("Undefined variable '{}'", name),
                    })
                } else {
                    Ok(Object::Nil)
                }
            }
            Expr::Logical(l, o, r) => {
                let left = self.evaluate(l)?;
                if o.token_type == TokenType::OR {
                    if truthy(&left) {
                        return Ok(left);
                    }
                } else if !truthy(&left) {
                    return Ok(left);
                }
                self.evaluate(r)
            }
            Expr::Assign(n, v) => {
                let val = self.evaluate(v)?;
                if let TokenType::IDENTIFIER(name) = &n.token_type {
                    self.env
                        .assign(name.to_string(), val)
                        .context(LoxRuntimeError {
                            t: n.clone(),
                            message: format!("Undefined variable '{}'", name),
                        })?;
                    self.env.get(name)
                } else {
                    Ok(Object::Nil)
                }
            }
            Expr::Call(callee, args) => {
                let callee = self.evaluate(callee)?;
                let arguments: Result<Vec<Object>> = args
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .into_iter()
                    .collect();
                let arguments = arguments?;

                match callee {
                    Object::Callable(c) => Ok(c.call(self, arguments)),
                    _ => todo!(), /*Runtime error */
                }
            }
        }
    }

    fn execute(&mut self, ast: &Stmt) -> Result<StmtResult> {
        match ast {
            Stmt::Print(e) => {
                println!("{}", self.evaluate(e)?);
                Ok(StmtResult::Noop)
            }
            Stmt::Expr(e) => {
                self.evaluate(e)?;
                Ok(StmtResult::Noop)
            }
            Stmt::Var(name, e) => {
                if let Some(expr) = e {
                    let o = self.evaluate(expr)?;
                    self.env.define(name.clone(), o)
                } else {
                    self.env.define(name.clone(), Object::Nil)
                }
                Ok(StmtResult::Noop)
            }
            Stmt::Block(stmts) => {
                self.env.push_scope();
                let mut result: Vec<Result<StmtResult>> = stmts
                    .iter()
                    .map(|s| -> Result<StmtResult> { self.execute(s) })
                    .take_until(|r| !matches!(r, Ok(StmtResult::Noop)))
                    .into_iter()
                    .collect();
                // println!("jt: {:?}", result);
                self.env.pop_scope();
                result.pop().unwrap_or(Ok(StmtResult::Noop))
            }
            Stmt::If(c, t, e) => {
                if truthy(&self.evaluate(c)?) {
                    self.execute(t)
                } else if let Some(e) = e {
                    self.execute(e)
                } else {
                    Ok(StmtResult::Noop)
                }
            }
            Stmt::While(c, s) => {
                while truthy(&self.evaluate(c)?) {
                    let r = self.execute(s);
                    if !matches!(r, Ok(StmtResult::Noop)) {
                        return r;
                    };
                }
                Ok(StmtResult::Noop)
            }
            Stmt::Function(name, params, body) => {
                self.env.define(
                    name.clone(),
                    Object::Callable(LoxCallableWrapper {
                        inner: Rc::new(LoxFunction {
                            params: params.clone(),
                            body: *body.clone(),
                        }),
                    }),
                );
                Ok(StmtResult::Noop)
            }
            Stmt::Return(_kw, v) => {
                // TODO: Find a way to handle the unwind here ... we don't have throw like java, could use the Error short circuting, but that feels ... gross.
                // FIXME: Maybe something like this guy did: https://github.com/franeklubi/luxya/blob/c38bd0a3e3063241f0e7517778adab6040ddf08a/src/interpreter/types.rs#L144-L149
                // which has a StmtResult (Continue, Break, Return, Noop) to propagate those statement actions up 🤔

                let rv = if let Some(v) = v {
                    self.evaluate(v)?
                } else {
                    Object::Nil
                };
                Ok(StmtResult::Return(rv))
            }
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        statements
            .iter()
            .map(|statement| self.execute(statement))
            .map(|r| -> Result<()> {
                match r {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            })
            .into_iter()
            .collect()
    }
}
