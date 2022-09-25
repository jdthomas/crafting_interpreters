use crate::environment::Enviornment;
use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::parser;
use crate::scanner;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use derive_more::Display;

pub struct Lox {
    pub has_error: bool,
    opts: LoxOptions,
}

#[derive(Debug, Display)]
pub struct LoxScanError {}
#[derive(Debug, Display)]
pub struct LoxParseError {}
pub use crate::interpreter::LoxRuntimeError;

#[derive(Parser, Debug)]
pub struct LoxOptions {
    #[clap(short, long)]
    debug_ast: bool,
}

impl Lox {
    pub fn new(opts: LoxOptions) -> Lox {
        Lox {
            has_error: false,
            opts,
        }
    }

    fn check_err(&self) -> Result<()> {
        match self.has_error {
            false => Ok(()),
            true => Err(anyhow!("Failure")),
        }
    }

    pub fn run(&mut self, source: String) -> Result<()> {
        self.run_with_env(source, &mut Enviornment::new())
    }

    pub fn run_with_env(&mut self, source: String, env: &mut Enviornment) -> Result<()> {
        let tokens = scanner::scan_tokens(self, &source);
        // println!("Tokens: {:#?}", tokens);
        if self.check_err().is_err() {
            return Err(anyhow!("failed to scan")).context(LoxScanError {});
        }

        let tok = tokens?;
        let mut tok = tok.iter().peekable();
        let mut parser = parser::Parser::new(&mut tok, self);

        let ast = parser.parse()?;
        if self.opts.debug_ast {
            println!("AST: {:#?}", ast);
        }
        if self.check_err().is_err() {
            return Err(anyhow!("failed to scan")).context(LoxParseError {});
        }
        let mut interpreter = Interpreter::new_with_env(env);
        let rte = interpreter.interpret(&ast);
        // println!("{:?}", rte);
        if let Err(err) = &rte {
            if let Some(e) = err.downcast_ref::<LoxRuntimeError>() {
                eprintln!("{}", e);
            }
            return rte;
        }

        self.check_err()
    }
}

impl Default for Lox {
    fn default() -> Self {
        Self::new(LoxOptions { debug_ast: false })
    }
}

impl LoxError for Lox {
    fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: i32, wh: &str, message: &str) {
        eprintln!(
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
