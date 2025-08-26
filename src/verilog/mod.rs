

mod writer;
mod parse;
pub mod wire;
mod port;
mod inst;


use port::VerilogPort;
use inst::VerilogInst;
pub struct VerilogFile {
    module_name: String,
    port_list: Vec<VerilogPort>,
    inst_list: Vec<VerilogInst>,
}




#[cfg(test)]
mod test {
    use crate::*;

    // #[test]
    // fn test_wire_eq() {
    //     let a = VerilogWire::new(32, "temp".into());
    //     let b = VerilogWire::new(12, "temp".into());
    //     assert!(a==b, "false");
    // }
}