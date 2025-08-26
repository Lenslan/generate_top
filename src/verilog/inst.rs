use crate::verilog::port::{PortDir, VerilogPort};


#[derive(Default)]
pub struct VerilogInst {
    inst_name: String,
    module_name: String,
    port_list: Vec<VerilogPort>,

    max_port_name_len: u32,
}
impl VerilogInst {
    pub fn new(module_name: &str) -> Self {
        Self {
            inst_name: format!("u_{}", module_name),
            module_name: module_name.into(),
            port_list: Vec::new(),
            ..Default::default()

        }
    }

    fn add_port(&mut self, inout: PortDir, name: &str, width: u32) {
        self.port_list.push(VerilogPort::new(
            inout,
            name,
            width as usize
        ))
    }

    pub fn fix_inst_name(&mut self, inst_name: &str) {
        self.inst_name = inst_name.into();
    }

    fn to_string(&self) -> Vec<String> {
        let mut res = Vec::new();
        res.push(format!("{} {} (", self.module_name, self.inst_name));

        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports {
                todo!()
                // res.push(format!("    .{}, {}", port.to_inst_string(), ));
            }
            res.push(format!("    .{});", last_port.to_inst_string()));
        } else {
            log::error!("There is no port in module {}", self.module_name);
        }
        res
    }
}