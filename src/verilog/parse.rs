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
}