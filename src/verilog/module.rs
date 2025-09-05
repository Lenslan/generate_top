use std::sync::Arc;
use crate::verilog::port::{PortDir, VerilogPort};


const INST_NAME_LEN:u8 = 20;
const INST_SIGNAL_LEN:u8 = 20;
#[derive(Default, Debug)]
pub struct VerilogModule {
    module_name: String,
    inst_name: Option<String>,
    port_list: Vec<VerilogPort>,
    inst_list: Vec<Arc<VerilogModule>>

}
impl VerilogModule {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            ..Default::default()

        }
    }

    ///
    /// Adds a new port to the module's port list.
    ///
    pub fn add_port(&mut self, inout: PortDir, name: &str, width: u32) {
        self.port_list.push(VerilogPort::new(
            inout,
            name,
            width as usize
        ))
    }
    
    pub fn add_ports(&mut self, ports: Vec<VerilogPort>) {
        self.port_list.extend(ports);
    }

    pub fn add_inst_module(&mut self, module: Arc<VerilogModule>) {
        self.inst_list.push(module);
    }

    ///
    /// Fix instance name
    ///
    pub fn fix_inst_name(&mut self, inst_name: &str) {
        self.inst_name = Some(inst_name.into());
    }

    ///
    /// output instance String
    ///
    fn to_inst_string(&self) -> Vec<String> {
        let mut res = Vec::new();
        res.push(format!("{} {} (", self.module_name, self.inst_name.as_ref().unwrap()));

        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports {
                res.push(format!("    .{}, {}", port.to_inst_string(INST_NAME_LEN, INST_SIGNAL_LEN), port.info));
                todo!()
            }
            res.push(format!("    .{}); {}", last_port.to_inst_string(INST_NAME_LEN, INST_SIGNAL_LEN), last_port.info));
        } else {
            log::error!("There is no port in module {}", self.module_name);
        }
        res
    }

    fn to_module_string(&self) -> Vec<String> {
        let mut res = Vec::new();
        let mut indent = 0;
        res.push(format!("{} (", self.module_name));

        indent += 4;
        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in self.port_list.iter() {
                res.push(format!("{indent_space}{inout} wire [0:{width}] {name},",
                                 indent_space=" ".repeat(indent),
                                 inout=port.inout,
                                 width=port.width,
                                 name=port.name))
            }
            res.push(format!("{indent_space}{inout} wire [0:{width}] {name});",
                             indent_space=" ".repeat(indent),
                             inout=last_port.inout,
                             width=last_port.width,
                             name=last_port.name))
        }
        // 线网的定义  TODO

        for inst in self.inst_list.iter() {
            res.extend(inst.to_inst_string());
            res.push("\n\n".into());
        }
        res.push("endmodule".into());

        todo!()
    }
}