use crate::verilog::wire::VerilogWire;

struct VerilogExpression {
    left: VerilogWire,
    right: Option<VerilogWire>,
    op: Option<VerilogOperator>
}

impl VerilogExpression {
    pub fn parse_from(s: &str) -> Self {
        todo!()
    }
}

enum VerilogOperator {
    And,
    Or,
    Not
}