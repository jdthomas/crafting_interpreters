use crate::parser::Expr;
use crate::tokens::TokenType;

#[derive(PartialEq, Debug)]
pub struct LoxObj {}

#[derive(PartialEq, Debug)]
pub enum Object {
    Object(LoxObj),
    Nil,
    Boolean(bool),
    Double(f64),
    String(String),
}

pub fn truthy(o: Object) -> bool {
    match o {
        Object::Nil => false,
        Object::Boolean(x) => x,
        _ => true,
    }
}

pub fn equal(l: Object, r: Object) -> bool {
    match (l, r) {
        (Object::Nil, Object::Nil) => true,
        (Object::Nil, _) => false,
        (_, Object::Nil) => false,
        (a, b) => a == b,
    }
}

pub fn evaluate_unary(t: &TokenType, e: &Expr) -> Object {
    let right = interpreter(e);
    match (t, right) {
        (TokenType::MINUS, Object::Double(x)) => Object::Double(-x),
        (TokenType::BANG, o) => Object::Boolean(!truthy(o)),
        _ => panic!("oopsies, bad unary"),
    }
}

pub fn evaluate_binary(left: &Expr, t: &TokenType, right: &Expr) -> Object {
    let left = interpreter(left);
    let right = interpreter(right);
    match (left, t, right) {
        (Object::String(l), TokenType::PLUS, Object::String(r)) => {
            Object::String(format!("{}{}", l, r))
        }
        (Object::Double(l), TokenType::PLUS, Object::Double(r)) => Object::Double(l + r),
        (Object::Double(l), TokenType::MINUS, Object::Double(r)) => Object::Double(l - r),
        (Object::Double(l), TokenType::STAR, Object::Double(r)) => Object::Double(l * r),
        (Object::Double(l), TokenType::SLASH, Object::Double(r)) => Object::Double(l / r),

        (Object::Double(l), TokenType::LESS, Object::Double(r)) => Object::Boolean(l < r),
        (Object::Double(l), TokenType::LESS_EQUAL, Object::Double(r)) => Object::Boolean(l <= r),
        (Object::Double(l), TokenType::GREATER, Object::Double(r)) => Object::Boolean(l > r),
        (Object::Double(l), TokenType::GREATER_EQUAL, Object::Double(r)) => Object::Boolean(l >= r),

        (l, TokenType::EQUAL_EQUAL, r) => Object::Boolean(equal(l, r)),
        (l, TokenType::BANG_EQUAL, r) => Object::Boolean(!equal(l, r)),

        (l, t, r) => panic!("Bad binary expr '{:?}' '{}' '{:?}'", l, t, r),
    }
}

pub fn evaluate_literal(t: &TokenType) -> Object {
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

pub fn evaluate_group(e: &Expr) -> Object {
    interpreter(e)
}

pub fn interpreter(ast: &Expr) -> Object {
    match ast {
        Expr::Binary(left, t, right) => evaluate_binary(left, t, right),
        Expr::Unary(t, e) => evaluate_unary(t, e),
        Expr::Literal(t) => evaluate_literal(t),
        Expr::Grouping(s) => evaluate_group(s),
    }
}
