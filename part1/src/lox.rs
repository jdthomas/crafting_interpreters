use crate::environment::Enviornment;
use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::parser;
use crate::scanner;
use anyhow::anyhow;
use anyhow::Result;

pub struct Lox {
    pub has_error: bool,
}

impl Lox {
    pub fn new() -> Lox {
        Lox { has_error: false }
    }

    fn check_err(&self) -> Result<()> {
        match self.has_error {
            false => Ok(()),
            true => Err(anyhow!("Failure")),
        }
    }

    pub fn run(&mut self, source: String) -> Result<()> {
        let tokens = scanner::scan_tokens(self, &source);
        println!("Tokens: {:#?}", tokens);
        self.check_err()?;
        // parser::parse(&mut tokens?.iter().peekable())?;
        let tok = tokens?;
        let mut tok = tok.iter().peekable();
        let mut parser = parser::Parser::new(&mut tok, self);
        let ast = parser.parse();
        println!("AST: {:?}", ast);
        self.check_err()?;
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&ast);
        self.check_err()
    }
}

impl LoxError for Lox {
    fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: i32, wh: &str, message: &str) {
        println!(
            "[line {line}] Error{wh}: {message}",
            line = line,
            wh = wh,
            message = message
        );
        self.has_error = true;
    }

    fn has_error(&self) -> bool {
        self.has_error
    }
}
