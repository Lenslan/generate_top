use std::cell::RefCell;
use std::collections::HashSet;
use crate::verilog::port::{PortDir, VerilogPort};
use crate::verilog::wire::WireBuilder;
use std::sync::Arc;
use crate::verilog::VerilogBase;

const INST_NAME_LEN: u8 = 30;
const INST_SIGNAL_LEN: u8 = 30;
#[derive(Default, Debug)]
pub struct VerilogModule {
    pub module_name: String,
    pub inst_name: Option<String>,
    pub port_list: Vec<VerilogPort>,
    pub inst_list: Vec<Arc<RefCell<VerilogModule>>>,
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
        self.port_list
            .push(VerilogPort::new(inout, name, width as usize))
    }

    pub fn add_ports(&mut self, ports: Vec<VerilogPort>) {
        self.port_list.extend(ports);
    }

    pub fn add_inst_module(&mut self, module: Arc<RefCell<VerilogModule>>) {
        self.inst_list.push(module);
    }

    ///
    /// Fix instance name
    ///
    pub fn fix_inst_name(&mut self, inst_name: &str) {
        self.inst_name = Some(inst_name.into());
    }
    pub fn set_default_inst_name(&mut self) { self.inst_name = Some(format!("u_{}", self.module_name)) }
    
    ///
    /// set all the ports connect to self
    /// 
    pub fn set_default_port_wires(&mut self) {
        for p in self.port_list.iter_mut() {
            p.connect_self();
            p.check_health();
        }
    }

    ///
    /// Compared with other VerilogModules
    /// to find ports in self not in other
    ///
    pub fn diff_ports_with(&self, other: &VerilogModule) -> Vec<&VerilogPort> {
        let other_ports: HashSet<_> = other.port_list.iter().collect();
        self.port_list.iter().filter(|item| {
            !other_ports.contains(item)
        }).collect()
    }

    ///
    /// output instance String
    ///
    fn to_inst_string(&self) -> Vec<String> {
        let mut res = Vec::new();
        res.push(format!(
            "{} {} (",
            self.module_name,
            self.inst_name.as_ref().unwrap()
        ));

        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports {
                res.push(format!(
                    "    .{}, {}",
                    port.to_inst_string(INST_NAME_LEN, INST_SIGNAL_LEN),
                    port.info
                ));
            }
            res.push(format!(
                "    .{}\n); {}",
                last_port.to_inst_string(INST_NAME_LEN, INST_SIGNAL_LEN),
                last_port.info
            ));
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

        // port info
        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports.iter() {
                res.push(format!(
                    "{indent_space}{inout} wire [{width}:0] {name},",
                    indent_space = " ".repeat(indent),
                    inout = port.inout,
                    width = port.width,
                    name = port.name
                ))
            }
            res.push(format!(
                "{indent_space}{inout} wire [{width}:0] {name});",
                indent_space = " ".repeat(indent),
                inout = last_port.inout,
                width = last_port.width,
                name = last_port.name
            ))
        }

        // wire definition
        let s = WireBuilder::traverse_unport_wires()
            .iter()
            .map(|(width, name)| {
                format!(
                    "{}wire {:<20} {}",
                    " ".repeat(indent),
                    format!("[{}:0]", width - 1),
                    name
                )
            })
            .collect::<Vec<String>>();
        res.extend(s);

        // inst info
        for inst in self.inst_list.iter() {
            res.extend(inst.borrow().to_inst_string());
            res.push("\n\n".into());
        }
        res.push("endmodule".into());

        res
    }
}

impl VerilogBase for VerilogModule {
    fn get_name(&self) -> String {
        self.module_name.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::verilog::module::VerilogModule;
    use crate::verilog::port::{PortDir, VerilogPort};

    #[test]
    fn test_inst_string() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        let mut module = VerilogModule::new("test".to_string());
        module.fix_inst_name("u_test_module");
        let mut port1 = VerilogPort::new(PortDir::InPort, "port1", 12);
        port1.set_info_msg("test1 info message");
        port1.connect_partial_signal("wire1", &(0..4));
        port1.connect_partial_signal("wire2", &(0..5));
        port1.connect_partial_signal("wire3", &(0..3));
        println!("{:?}", port1.signals);
        let mut port2 = VerilogPort::new(PortDir::InPort, "port2", 12);
        port2.connect_undefined_signal("undefined-wires");
        let mut port3 = VerilogPort::new(
            PortDir::OutPort,
            "pordddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddt3",
            12,
        );
        port3.connect_number_signal(43, 8);
        module.add_ports(vec![port1, port2, port3]);
        println!("{}", module.to_inst_string().join("\n"));
    }
}
