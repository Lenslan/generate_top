use std::sync::Arc;
use crate::verilog::wire::VerilogWire;

pub struct VerilogPort {
    inout: PortDir,
    name: String,
    signal: Option<Arc<VerilogWire>>,
}
impl VerilogPort {
    fn new(inout: PortDir, name: &str) -> Self {
        Self {
            inout,
            name: String::from(name),
            signal: None
        }
    }

    fn connect_signal(&self, sig: &str) {

    }
}

pub enum PortDir {
    InPort,
    OutPort,
    InOutPort,
}

impl PortDir {
    fn is_in(&self) -> bool {
        match self {
            Self::OutPort => false,
            _ => true
        }
    }

    fn is_out(&self) -> bool {
        match self {
            Self::InPort => false,
            _ => true
        }
    }
}