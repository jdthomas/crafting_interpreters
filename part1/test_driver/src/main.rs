use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use colored::*;
use itertools::zip;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
    static ref EXPECTED_OUTPUT_PATTERN: Regex = Regex::new(r"// expect: ?(.*)").unwrap();
    static ref EXPECTED_ERROR_PATTERN: Regex = Regex::new(r"// (Error.*)").unwrap();
    static ref ERROR_LINE_PATTERN: Regex =
        Regex::new(r"// \[((java|c) )?line (\d+)\] (Error.*)").unwrap();
    static ref EXPECTED_RUNTIME_ERROR_PATTERN: Regex =
        Regex::new(r"// expect runtime error: (.+)").unwrap();
    static ref SYNTAX_ERROR_PATTERN: Regex = Regex::new(r"\[.*line (\d+)\] (Error.+)").unwrap();
    static ref STACK_TRRACE_PATTERN: Regex = Regex::new(r"\[line (\d+)\]").unwrap();
    static ref NON_TEST_PATTERN: Regex = Regex::new(r"// nontest").unwrap();
}

#[derive(Debug)]
struct ExpectedOutput {
    line: i32,
    output: String,
}

#[derive(Debug)]
struct Test {
    test_file: PathBuf,
    expected_output: Vec<ExpectedOutput>,
    /// The set of expected compile error messages.
    expected_errors: Vec<String>,
    /// The expected runtime error message or `None` if there should not be one.
    expected_runtime_error: Option<ExpectedOutput>,
    expected_exit_code: i32,
}

impl Test {
    fn try_parse(test_input_path: &PathBuf) -> Option<Self> {
        // let mut expected_output: Vec<ExpectedOutput> = vec![];
        // let mut expected_errors: Vec<String> = vec![];
        // let mut expected_exit_code: i32 = 0;
        // let mut expected_runtime_error: Option<ExpectedOutput> = None;
        let mut test = Test {
            test_file: test_input_path.clone(),
            expected_output: vec![],
            expected_errors: vec![],
            expected_exit_code: 0,
            expected_runtime_error: None,
        };
        let file = File::open(test_input_path).ok()?;
        let lines = io::BufReader::new(file).lines();
        lines.enumerate().for_each(|(lineno, line)| {
            let line = line.unwrap(); // FIXME
            if let Some(eo) = EXPECTED_OUTPUT_PATTERN.captures(&line) {
                test.expected_output.push(ExpectedOutput {
                    line: lineno as i32,
                    output: eo[1].to_string(),
                });
            }

            if let Some(ee) = EXPECTED_ERROR_PATTERN.captures(&line) {
                test.expected_errors
                    .push(format!("[{}] {}", lineno, &ee[1]));
                // If we expect a compile error, it should exit with EX_DATAERR.
                test.expected_exit_code = 65;
            }

            if let Some(ee) = ERROR_LINE_PATTERN.captures(&line) {
                // The two interpreters are slightly different in terms of which
                // cascaded errors may appear after an initial compile error because
                // their panic mode recovery is a little different. To handle that,
                // the tests can indicate if an error line should only appear for a
                // certain interpreter.

                //   var language = match[2];
                //   if (language == null || language == _suite.language) {
                if ee.get(2).is_none() {
                    test.expected_errors
                        .push(format!("[line {}] {}", &ee[3], &ee[4]));
                    test.expected_exit_code = 65;
                }
                //     // If we expect a compile error, it should exit with EX_DATAERR.
                //     _expected_exit_code = 65;
                //     _expectations++;
                //   }
                //   continue;
            }
            if let Some(rte) = EXPECTED_RUNTIME_ERROR_PATTERN.captures(&line) {
                test.expected_runtime_error = Some(ExpectedOutput {
                    line: lineno as i32,
                    output: rte[1].to_owned(),
                });
                // If we expect a runtime error, it should exit with EX_SOFTWARE.
                test.expected_exit_code = 70;
            }
        });

        if !test.expected_errors.is_empty() && test.expected_runtime_error.is_some() {
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
    fn validate_runtime_error(&self, std_err: &[String]) -> Result<()> {
        if let Some(expected_runtime_error) = &self.expected_runtime_error {
            // if std_err.len() < 2 {
            //     return Err(anyhow!(
            //         "Expected runtime error '{:?}' and got none.",
            //         self.expected_runtime_error
            //     ));
            // }
            if std_err[0] != expected_runtime_error.output {
                return Err(anyhow!(
                    "Expected runtime error '{}' and got:\n{}",
                    expected_runtime_error.output,
                    std_err[0]
                ));
            }
            // Make sure the stack trace has the right line.
            let matching = std_err[1..]
                .iter()
                .find(|line| STACK_TRRACE_PATTERN.is_match(line));
            if let Some(stack) = matching {
                let captured = STACK_TRRACE_PATTERN.captures(stack);
                let stack_line = captured.unwrap()[1].parse::<i32>().unwrap();
                if stack_line != expected_runtime_error.line {
                    return Err(anyhow!(
                        "Expected runtime error on line {} but was on line {}",
                        expected_runtime_error.line,
                        stack_line
                    ));
                }
            } else {
                //     fail("Expected stack trace and got:", stackLines);
                return Err(anyhow!("Expected stack trace and got: {:?}", &std_err[1..]));
            }
        }
        Ok(())
    }

    fn validate_compile_errors(&self, std_err: &Vec<String>) -> Result<()> {
        if !self.expected_errors.is_empty() {
            let matching = zip(&self.expected_errors, std_err)
                .filter(|&(a, b)| a == b)
                .count();
            println!("{:?} {:?} {}", &self.expected_errors, std_err, matching);
            if matching == std_err.len() && matching == self.expected_errors.len() {
                Ok(())
            } else {
                Err(anyhow!("Compliation Error"))
            }
            // // Validate that every compile error was expected.
            // var foundErrors = <String>{};
            // var unexpectedCount = 0;
            // for (var line in error_lines) {
            // var match = _syntax_error_pattern.firstMatch(line);
            // if (match != null) {
            //     var error = "[${match[1]}] ${match[2]}";
            //     if (_expected_errors.contains(error)) {
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
            // for (var error in _expected_errors.difference(foundErrors)) {
            // fail("Missing expected error: $error");
            // }
        } else {
            Ok(())
        }
    }

    fn validate_exit_code(&self, exit_code: i32) -> Result<()> {
        if exit_code == self.expected_exit_code {
            Ok(())
        } else {
            Err(anyhow!(
                "Expected return code {} and got {}",
                self.expected_exit_code,
                exit_code
            ))
        }
    }
    fn validate_output(&self, _std_out: &[String]) -> Result<()> {
        // if !self.expected_output.is_empty() {
        //     let matching = zip(&self.expected_output, std_out)
        //         .filter(|&(a, b)| &a.output == b)
        //         .count();
        //     println!("{:?} {:?} {}", &self.expected_output, std_out, matching);
        //     if matching == std_out.len() && matching == self.expected_output.len() {
        //         Ok(())
        //     } else {
        //         Err(anyhow!("Output Error"))
        //     }
        //     //         // Remove the trailing last empty line.
        //     // if (outputLines.isNotEmpty && outputLines.last == "") {
        //     //     outputLines.removeLast();
        //     //   }

        //     //   var index = 0;
        //     //   for (; index < outputLines.length; index++) {
        //     //     var line = outputLines[index];
        //     //     if (index >= _expectedOutput.length) {
        //     //       fail("Got output '$line' when none was expected.");
        //     //       continue;
        //     //     }

        //     //     var expected = _expectedOutput[index];
        //     //     if (expected.output != line) {
        //     //       fail("Expected output '${expected.output}' on line ${expected.line} "
        //     //           " and got '$line'.");
        //     //     }
        //     //   }

        //     //   while (index < _expectedOutput.length) {
        //     //     var expected = _expectedOutput[index];
        //     //     fail("Missing expected output '${expected.output}' on line "
        //     //         "${expected.line}.");
        //     //     index++;
        //     //   }
        // } else {
        //     Ok(())
        // }
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
    // println!(
    //     "Passed: {} Failed: {} Skipped: {} ({})",
    //     _passed.to_string().green(),
    //     _failed.to_string().red(),
    //     _skipped.to_string().yellow(),
    //     test_input_path
    //         .clone()
    //         .into_os_string()
    //         .into_string()
    //         .unwrap(), //.into_os_string().into_string().context("")?.dimmed(),
    // );

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

    test.validate_runtime_error(&error_lines)?;
    test.validate_compile_errors(&error_lines)?;
    test.validate_exit_code(exit_code)?;
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
    // assert_eq!(exit_code, test.expected_exit_code);
    // assert_eq!(output_lines, test.expected_output);
    // println!("{:?}", zip(output_lines, test.expected_output));

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
    let test = Test::try_parse(&PathBuf::from(&test_input));
    println!("test: {:#?}", test);
    let e = run_test(test.unwrap(), test_binary);
    match e {
        Ok(_) => println!("[{}] ({})", "PASSED".green(), &test_input),

        Err(_) => println!("[{}] ({})", "FAILED".red(), &test_input),
    };
    e
}
