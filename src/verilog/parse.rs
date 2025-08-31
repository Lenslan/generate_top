use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// to use verible or pyverilog
///
/// verilog-parser
/// https://github.com/ben-marshall/verilog-parser?tab=readme-ov-file
///
/// Pyverilog
/// https://github.com/PyHDI/Pyverilog
///      - need iverilog to preprocess
///
/// verible
/// https://github.com/chipsalliance/verible?tab=readme-ov-file
///

struct VeribleVerilogSyntax {
    executable: String,
}

impl VeribleVerilogSyntax {
    pub fn new(executable: &str) -> Self {
        Self {
            executable: executable.to_string(),
        }
    }

    fn run(&self) {
        let output = Command::new(&self.executable)
            .args(["-export_json", "-printtree"])
            .arg("test/npu_afifo_r.sv")
            .output()
            .expect("can not exec script");

        let v:Value = serde_json::from_slice(&output.stdout).unwrap();
        for (file_path, file_data) in v.as_object().unwrap() {
            println!("{}", file_path);
            let temp:VeribleErrors = serde_json::from_value(file_data.clone()).unwrap();
            println!("{:?}", temp);

        }

        println!("status {}", output.status);
        println!("status: {}", output.status.success());
        // println!("output: {}", String::from_utf8_lossy(&output.stdout));
        // println!("error: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Errors {
    column: u32,
    line: u32,
    phase: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VeribleErrors {
    errors: Vec<Errors>
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_command() {
        let v = VeribleVerilogSyntax::new("bin/verible-verilog-syntax.exe");
        v.run()

    }
}