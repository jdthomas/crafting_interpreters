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

    pub fn run(&mut self, source: String) -> Result<()> {
        let tokens = scanner::scan_tokens(self, &source);
        println!("Tokens: {:#?}", tokens);
        if self.has_error {
            Err(anyhow!("Failure"))
        } else {
            Ok(())
        }
    }

    pub fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: i32, wh: &str, message: &str) {
        println!(
            "[line {line}] Error{wh}: {message}",
            line = line,
            wh = wh,
            message = message
        );
        self.has_error = true;
    }
}
