use std::env;
use std::fs::canonicalize;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use serde_json;
use tempfile;

const DIFF: &str = "/usr/bin/diff";

fn run_single_test_case(prog_path: &Path, in_path: &Path, ans_path: &Path) {
    let f = File::open(in_path).unwrap();
    let in_bytes = f.bytes().collect::<io::Result<Vec<_>>>().unwrap();
    let mut child = process::Command::new(prog_path)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .expect("Failed to open stdin")
        .write_all(&in_bytes)
        .unwrap();
    let status = child.wait().unwrap();
    if let Some(code) = status.code() {
        if code != 0 {
            println!("RE: `{}` returned {}.", prog_path.display(), code);
            process::exit(10);
        }
    }

    let mut tmp_out_file = tempfile::NamedTempFile::new().unwrap();
    let out_bytes = child
        .stdout
        .expect("Failed to open stdout")
        .bytes()
        .collect::<io::Result<Vec<_>>>()
        .unwrap();
    tmp_out_file
        .write_all(&out_bytes)
        .expect("Failed to write to tempfile");
    tmp_out_file.flush().unwrap();

    let mut child = process::Command::new(DIFF)
        .arg("-Z")
        .arg("-B")
        .arg(canonicalize(ans_path).unwrap().as_os_str())
        .arg(tmp_out_file.path().as_os_str())
        .spawn()
        .unwrap();
    let status = child.wait().unwrap();
    if let Some(code) = status.code() {
        if code != 0 {
            println!("WA: `diff` returned {}.", code);
            tmp_out_file.close().unwrap();
            process::exit(11);
        }
    }
    tmp_out_file.close().unwrap();
}

fn main() {
    let prog_dir = env::var("PROG_PATH").expect("PROG_PATH not set");
    let base_dir = env::var("BASE_DIR").expect("BASE_DIR not set");

    let prog_path = Path::new(prog_dir.as_str());
    let base_path = Path::new(base_dir.as_str());

    let test_suite: serde_json::Value =
        serde_json::from_reader(File::open(base_path.join("in_out.json")).unwrap()).unwrap();
    for in_out in test_suite.as_array().unwrap().iter() {
        let tmp = in_out.as_array().unwrap();
        let in_path = tmp[0].as_str().unwrap();
        let ans_path = tmp[1].as_str().unwrap();
        run_single_test_case(
            prog_path,
            &base_path.join(in_path),
            &base_path.join(ans_path),
        );
        println!("{} OK", in_path);
    }
}
