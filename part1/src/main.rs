use anyhow::Result;
use clap::Parser;
use lib::environment::Enviornment;
use lib::lox::Lox;
use lib::lox::LoxParseError;
use lib::lox::LoxRuntimeError;
use lib::lox::LoxScanError;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Script to run
    #[clap()]
    script: Option<String>,
}

fn run_file(script_path: &str) -> Result<()> {
    let mut l = Lox::new();
    let data = fs::read_to_string(script_path)?;
    l.run(data)
}

fn run_prompt() -> Result<()> {
    let mut l = Lox::new();
    let mut env = Enviornment::new();
    const HISTORY_FILE: &str = "history.txt";

    let mut rl = Editor::<()>::new()?;
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No privious history");
    }

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                l.run_with_env(line, &mut env)?;
                // ...
                // FIXME: Don;t bail on bad line and reset l.has_error between
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(HISTORY_FILE)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    // println!("Hello, world! {:?}", args);
    let rv = match args.script {
        None => run_prompt(),
        Some(script) => run_file(&script),
    };
    if let Err(e) = &rv {
        if e.downcast_ref::<LoxScanError>().is_some() {
            ::std::process::exit(65);
        } else if e.downcast_ref::<LoxRuntimeError>().is_some() {
            ::std::process::exit(70);
        } else if e.downcast_ref::<LoxParseError>().is_some() {
            ::std::process::exit(65);
        }
    }
    rv
}
