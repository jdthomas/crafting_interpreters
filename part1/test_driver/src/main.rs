use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use colored::*;
use itertools::zip;
use lazy_static::lazy_static;
use lib::lox::Lox;
use regex::Regex;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
    static ref expectedOutputPattern: Regex = Regex::new(r"// expect: ?(.*)").unwrap();
    static ref expectedErrorPattern: Regex = Regex::new(r"// (Error.*)").unwrap();
    static ref errorLinePattern: Regex =
        Regex::new(r"// \[((java|c) )?line (\d+)\] (Error.*)").unwrap();
    static ref expectedRuntimeErrorPattern: Regex =
        Regex::new(r"// expect runtime error: (.+)").unwrap();
    static ref syntaxErrorPattern: Regex = Regex::new(r"\[.*line (\d+)\] (Error.+)").unwrap();
    static ref stackTracePattern: Regex = Regex::new(r"\[line (\d+)\]").unwrap();
    static ref nonTestPattern: Regex = Regex::new(r"// nontest").unwrap();
}

#[derive(Debug)]
struct ExpectedOutput {
    line: i32,
    output: String,
}

#[derive(Debug)]
struct Test {
    test_file: PathBuf,
    expectedOutput: Vec<ExpectedOutput>,
    /// The set of expected compile error messages.
    expectedErrors: Vec<String>,
    /// The expected runtime error message or `None` if there should not be one.
    expectedRuntimeError: Option<ExpectedOutput>,
    expectedExitCode: i32,
}

impl Test {
    fn try_parse(test_input_path: &PathBuf) -> Option<Self> {
        // let mut expectedOutput: Vec<ExpectedOutput> = vec![];
        // let mut expectedErrors: Vec<String> = vec![];
        // let mut expectedExitCode: i32 = 0;
        // let mut expectedRuntimeError: Option<ExpectedOutput> = None;
        let mut test = Test {
            test_file: test_input_path.clone(),
            expectedOutput: vec![],
            expectedErrors: vec![],
            expectedExitCode: 0,
            expectedRuntimeError: None,
        };
        let file = File::open(test_input_path).ok()?;
        let lines = io::BufReader::new(file).lines();
        lines.enumerate().for_each(|(lineno, line)| {
            let line = line.unwrap(); // FIXME
            if let Some(eo) = expectedOutputPattern.captures(&line) {
                test.expectedOutput.push(ExpectedOutput {
                    line: lineno as i32,
                    output: eo[1].to_string(),
                });
            }

            if let Some(ee) = expectedOutputPattern.captures(&line) {
                test.expectedErrors.push(format!("[{}] {}", lineno, &ee[1]));
                // If we expect a compile error, it should exit with EX_DATAERR.
                test.expectedExitCode = 65;
            }

            if let Some(_ee) = errorLinePattern.captures(&line) {
                // The two interpreters are slightly different in terms of which
                // cascaded errors may appear after an initial compile error because
                // their panic mode recovery is a little different. To handle that,
                // the tests can indicate if an error line should only appear for a
                // certain interpreter.
                //   var language = match[2];
                //   if (language == null || language == _suite.language) {
                //     _expectedErrors.add("[${match[3]}] ${match[4]}");

                //     // If we expect a compile error, it should exit with EX_DATAERR.
                //     _expectedExitCode = 65;
                //     _expectations++;
                //   }
                //   continue;
            }
            if let Some(rte) = expectedRuntimeErrorPattern.captures(&line) {
                test.expectedRuntimeError = Some(ExpectedOutput {
                    line: lineno as i32,
                    output: rte[1].to_owned(),
                });
                // If we expect a runtime error, it should exit with EX_SOFTWARE.
                test.expectedExitCode = 70;
            }
        });

        if test.expectedErrors.len() > 0 && test.expectedRuntimeError.is_some() {
            println!(
                "{} {} Cannot expect both compile and runtime errors.",
                "TEST ERROR".magenta(),
                test.test_file.into_os_string().into_string().unwrap()
            );
            None
        } else {
            Some(test)
        }
    }
    fn validate_runtime_error(&self, std_err: &Vec<String>) -> Result<()> {
        // if (errorLines.length < 2) {
        //     fail("Expected runtime error '$_expectedRuntimeError' and got none.");
        //     return;
        //  }

        if let Some(expected_runtime_error) = &self.expectedRuntimeError {
            if std_err[0] != expected_runtime_error.output {
                return Err(anyhow!(
                    "Expected runtime error '{}' and got:\n{}",
                    expected_runtime_error.output,
                    std_err[0]
                ));
            }
        }

        //   // Make sure the stack trace has the right line.
        //   RegExpMatch match;
        //   var stackLines = errorLines.sublist(1);
        //   for (var line in stackLines) {
        //     match = _stackTracePattern.firstMatch(line);
        //     if (match != null) break;
        //   }

        //   if (match == null) {
        //     fail("Expected stack trace and got:", stackLines);
        //   } else {
        //     var stackLine = int.parse(match[1]);
        //     if (stackLine != _runtimeErrorLine) {
        //       fail("Expected runtime error on line $_runtimeErrorLine "
        //           "but was on line $stackLine.");
        //     }
        //   }
        Ok(())
    }

    fn validate_compile_errors(&self, std_err: &Vec<String>) -> Result<()> {
        // let matching = zip(&self.expectedErrors, std_err)
        //     .filter(|&(a, b)| a == b)
        //     .count();
        // println!("{:?} {:?} {}", &self.expectedErrors, std_err, matching );
        // if matching == std_err.len() && matching == self.expectedErrors.len() {
        //     Ok(())
        // } else {
        //     Err(anyhow!("boop"))
        // }

        // // Validate that every compile error was expected.
        // var foundErrors = <String>{};
        // var unexpectedCount = 0;
        // for (var line in error_lines) {
        // var match = _syntaxErrorPattern.firstMatch(line);
        // if (match != null) {
        //     var error = "[${match[1]}] ${match[2]}";
        //     if (_expectedErrors.contains(error)) {
        //     foundErrors.add(error);
        //     } else {
        //     if (unexpectedCount < 10) {
        //         fail("Unexpected error:");
        //         fail(line);
        //     }
        //     unexpectedCount++;
        //     }
        // } else if (line != "") {
        //     if (unexpectedCount < 10) {
        //     fail("Unexpected output on stderr:");
        //     fail(line);
        //     }
        //     unexpectedCount++;
        // }
        // }

        // if (unexpectedCount > 10) {
        // fail("(truncated ${unexpectedCount - 10} more...)");
        // }

        // // Validate that every expected error occurred.
        // for (var error in _expectedErrors.difference(foundErrors)) {
        // fail("Missing expected error: $error");
        // }
        Ok(())
    }
    fn validate_exit_code(&self, exit_code: i32) -> Result<()> {
        if exit_code == self.expectedExitCode {
            Ok(())
        } else {
            Err(anyhow!(
                "Expected return code {} and got {}",
                self.expectedExitCode,
                exit_code
            ))
        }
    }
    fn validate_output(&self, std_out: &Vec<String>) -> Result<()> {
        Ok(())
    }
}

fn run_test(test: Test, prog: &str) -> Result<()> {
    // if (path.contains("benchmark")) return;

    // Make a nice short path relative to the working directory. Normalize it to
    // use "/" since the interpreters expect the argument to use that.
    let test_input_path = std::fs::canonicalize(&test.test_file)?;

    // Check if we are just running a subset of the tests.
    // if (_filterPath != null) {
    //   var thisTest = p.posix.relative(path, from: "test");
    //   if (!thisTest.startsWith(_filterPath)) return;
    // }
    let _passed: u32 = 0;
    let _failed: u32 = 0;
    let _skipped: u32 = 0;

    // Update the status line.
    println!(
        "Passed: {} Failed: {} Skipped: {} ({})",
        _passed.to_string().green(),
        _failed.to_string().red(),
        _skipped.to_string().yellow(),
        test_input_path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap(), //.into_os_string().into_string().context("")?.dimmed(),
    );

    let mut process = Command::new(prog)
        .args(&[test_input_path])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let exit_code = process.wait()?.code().unwrap();

    let output_lines: Vec<String> = BufReader::new(process.stdout.unwrap())
        .lines()
        .filter_map(|x| x.ok())
        .collect();
    let error_lines: Vec<String> = BufReader::new(process.stderr.unwrap())
        .lines()
        .filter_map(|x| x.ok())
        .collect();

    println!("stdout: {:?}", output_lines);
    println!("stderr: {:?}", error_lines);
    println!("exitcode: {:?}", exit_code);

    test.validate_exit_code(exit_code)?;
    test.validate_runtime_error(&error_lines)?;
    test.validate_compile_errors(&error_lines)?;
    test.validate_output(&output_lines)?;

    // // Display the results.
    // if (failures.isEmpty) {
    //   _passed++;
    // } else {
    //   _failed++;
    //   term.writeLine("${term.red("FAIL")} $path");
    //   print("");
    //   for (var failure in failures) {
    //     print("     ${term.pink(failure)}");
    //   }
    //   print("");
    // }
    // assert_eq!(exit_code, test.expectedExitCode);
    // assert_eq!(output_lines, test.expectedOutput);
    // println!("{:?}", zip(output_lines, test.expectedOutput));

    Ok(())
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Hello, world! {:?}", args);
    // let test_input = "test_lox_files/0005_presidence.lox";
    let test_input = args.input_file;
    let test_binary = "target/debug/interpreter";
    let test = Test::try_parse(&PathBuf::from(test_input));
    println!("test: {:#?}", test);
    run_test(test.unwrap(), test_binary)
}
