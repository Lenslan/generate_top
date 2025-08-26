use crate::verilog::port::VerilogPort;


pub struct VerilogInst {
    inst_name: String,
    module_name: String,
    port_list: VerilogPort
}
