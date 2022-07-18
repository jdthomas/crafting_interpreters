use anyhow::Result;
use clap::Parser;
use lib::lox::Lox;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

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
    let buffer = BufReader::new(io::stdin());
    let input_iter = buffer.lines();
    let mut l = Lox::new();
    println!("> ");

    for line in input_iter {
        // ...
        // FIXME: Don;t bail on bad line and reset l.has_error between
        l.run(line?)?;
        println!("> ");
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Hello, world! {:?}", args);
    match args.script {
        None => run_prompt(),
        Some(script) => run_file(&script),
    }
}
