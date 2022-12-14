use std::env;
use std::fs::canonicalize;
use std::fs::File;
use std::io;
// use std::io::prelude::*;
use std::fmt::Debug;
use std::path::Path;
use std::process;
use std::time::Duration;

use process_control::ChildExt;
use process_control::Control;


use serde_json;
// use rlimit::Resource;

const DIFF: &str = "/usr/bin/diff";
#[derive(Debug)]
enum ErrorType {
    // CompileError,
    RuntimeError(i64),
    // TimeLimitExceeded,
    // MemoryLimitExceeded,
    // OutputLimitExceeded,
    WrongAnswer,
    // PresentationError,
    // Accepted,
    Unknown(i32),
    IOError,
    JSONError,
}

impl From<io::Error> for ErrorType {
    fn from(_: io::Error) -> Self {
        ErrorType::IOError
    }
}

impl From<serde_json::Error> for ErrorType {
    fn from(_: serde_json::Error) -> Self {
        ErrorType::JSONError
    }
}

fn run_single_test_case(
    prog_path: &Path,
    in_path: &Path,
    ans_path: &Path,
) -> Result<(), ErrorType> {
    let mut input_file = File::open(in_path)?;
    let mut prog_child = process::Command::new(prog_path)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .spawn()
        .expect("Failed to spawn child process");
    io::copy(&mut input_file, prog_child.stdin.as_mut().unwrap())?;
    let result = prog_child
        .controlled()
        .time_limit(Duration::from_secs(10))
        .terminate_for_timeout()
        .wait()?
        .unwrap();
    if let Some(code) = result.code() {
        if code == 0 {
            let diff_output = process::Command::new(DIFF)
                .args(&[
                    "-Z",
                    "-B",
                    canonicalize(ans_path).unwrap().to_str().unwrap(),
                    "-",
                ])
                .stdin(process::Stdio::from(prog_child.stdout.unwrap()))
                // .stdout(process::Stdio::piped())
                .output()?;
            if let Some(code) = diff_output.status.code() {
                if code == 0 {
                    Ok(())
                } else {
                    // println!("{}", String::from_utf8(diff_output.stdout).unwrap());
                    Err(ErrorType::WrongAnswer)
                }
            } else {
                Err(ErrorType::Unknown(1))
            }
        } else {
            Err(ErrorType::RuntimeError(code))
        }
    } else {
        Err(ErrorType::Unknown(2))
    }
}

fn main() -> Result<(), ErrorType> {
    // Resource::NPROC.set(4, 4).unwrap();
    // Resource::CPU.set(2, 2).unwrap();

    let prog_dir = env::var("PROG_PATH").expect("PROG_PATH not set");
    let base_dir = env::var("BASE_DIR").expect("BASE_DIR not set");

    let prog_path = Path::new(prog_dir.as_str());
    let base_path = Path::new(base_dir.as_str());

    let test_suite: serde_json::Value =
        serde_json::from_reader(File::open(base_path.join("in_out.json"))?)?;
    for in_out in test_suite.as_array().unwrap().iter() {
        let tmp = in_out.as_array().unwrap();
        let in_path = tmp[0].as_str().unwrap();
        let ans_path = tmp[1].as_str().unwrap();
        match run_single_test_case(
            prog_path,
            &base_path.join(in_path),
            &base_path.join(ans_path),
        ) {
            Ok(_) => println!("{} OK", in_path),
            Err(e) => {
                println!("{} Err {:?}", in_path, e);
                return Err(e);
            }
        }
    }
    Ok(())
}
