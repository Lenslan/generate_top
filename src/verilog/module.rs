use std::cell::RefCell;
use std::collections::HashSet;
use crate::verilog::port::{PortDir, UndefineWireCollector, VerilogPort};
use crate::verilog::wire::WireBuilder;
use std::sync::Arc;
use crate::verilog::data::{VerilogData, WrapMacro};
use crate::verilog::parameter::Param;
use crate::verilog::VerilogBase;

const INST_NAME_LEN: u8 = 20;
const INST_SIGNAL_LEN: u8 = 25;
#[derive(Default, Debug)]
pub struct VerilogModule {
    pub module_name: String,
    pub inst_name: Option<String>,
    pub param_list: Vec<Param>,
    pub port_list: Vec<VerilogData<VerilogPort>>,
    pub inst_list: Vec<Arc<RefCell<VerilogData<VerilogModule>>>>,
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
    pub fn add_port(&mut self, inout: PortDir, name: &str, width: usize) {
        self.port_list
            .push(VerilogPort::new(inout, name, width.into()).wrap_raw())
    }

    pub fn add_port_inst(&mut self, port: VerilogData<VerilogPort>) {
        self.port_list.push(port);
    }

    pub fn add_ports(&mut self, ports: Vec<VerilogData<VerilogPort>>) {
        for item in ports {
            self.port_list.push(item);
        }
    }

    pub fn add_inst_module(&mut self, module: Arc<RefCell<VerilogData<VerilogModule>>>) {
        self.inst_list.push(module);
    }
    
    pub fn add_param_list(&mut self, param_list: Vec<Param>) {
        self.param_list.extend(param_list);
    }
    
    ///
    /// According module name to find inst module
    /// 
    pub fn find_inst_module_by_name(&self, name: &str) -> Option<Arc<RefCell<VerilogData<VerilogModule>>>> {
        for item in self.inst_list.iter() {
            if item.borrow().module_name == name {
                return Some(Arc::clone(item));
            }
        }
        None
    }

    // TODO replace by `same_ports_with`
    // ///
    // /// According port name to find port
    // ///
    // pub fn find_same_port(&self, port: &VerilogPort) -> Option<&VerilogPort> {
    //     for item in self.port_list.iter() {
    //         if port == item {
    //             return Some(item);
    //         }
    //     }
    //     None
    // }
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
    /// generate a module from other VerilogModule
    /// used to copy submodule
    ///
    pub fn copy_module_from(other: &VerilogData<VerilogModule>) -> VerilogData<VerilogModule>{
        let mut new_module = VerilogModule::new(other.module_name.clone());
        for p in other.port_list.iter() {
            let mut new_port = VerilogPort::copy_inst_port_from(p);
            new_port.check_health();
            new_module.add_port_inst(new_port);
        }
        new_module.wrap_macro_as(other)
    }

    pub fn copy_parameter_from(other: &VerilogModule) -> Vec<Param> {
        let mut res = Vec::with_capacity(other.param_list.len());
        for p in other.param_list.iter() {
            res.push(p.clone());
        }
        res
    }

    pub fn update_literal_port(&mut self, params: &Vec<Param>) {
        for p in self.port_list.iter_mut() {
            p.update_literal_port_width(params)
        }
    }

    ///
    /// Compare with other VerilogModules
    /// to find inst module in self not in other
    ///
    pub fn diff_inst_with(&self, other:&VerilogModule) -> Vec<Arc<RefCell<VerilogData<VerilogModule>>>> {
        let ids: HashSet<_> = other.inst_list
            .iter()
            .map(|x| x.borrow().module_name.clone())
            .collect();
        self.inst_list
            .iter()
            .filter(|x| !ids.contains(&x.borrow().module_name))
            .cloned()
            .collect()
    }

    ///
    /// Compared with other VerilogModules
    /// to find ports in self not in other
    ///
    pub fn diff_ports_with(&self, other: &VerilogModule) -> Vec<&VerilogData<VerilogPort>> {
        let other_ports: HashSet<_> = other.port_list.iter().collect();
        self.port_list.iter().filter(|item| {
            !other_ports.contains(item)
        }).collect()
    }

    ///
    /// Compared with other VerilogModules
    /// to find ports in self & other
    ///
    pub fn same_ports_with(&self, other: &VerilogModule) -> Vec<&VerilogData<VerilogPort>> {
        let other_ports: HashSet<_> = other.port_list.iter().collect();
        self.port_list.iter().filter(|item| {
            other_ports.contains(item)
        }).collect()
    }

    ///
    /// final check
    ///
    pub fn final_check(&mut self) {
        if UndefineWireCollector::has_wires() {
            UndefineWireCollector::solve_func();
            self.port_list.iter_mut().for_each(|p| p.check_health());
            self.inst_list.iter_mut().for_each(|inst| {
                inst.borrow_mut().port_list.iter_mut().for_each(|p| {
                    p.check_health();
                });
            });
        }
        WireBuilder::check_health();
    }

    ///
    /// output instance String
    ///
    pub fn to_inst_string(&self) -> Vec<String> {
        let mut res = Vec::new();

        if let Some((last_para, params)) = self.param_list.split_last() {
            res.push(format!("{} #(", self.module_name));
            for p in params {
                res.push(format!("    .{:<20}({:<10}),", p.name, p.value));
            }
            res.push(format!("    .{:<20}({:<10})", last_para.name, last_para.value));
            res.push(format!(") {} (", self.inst_name.as_ref().unwrap()));
        } else {
            res.push(format!(
                "{} {} (",
                self.module_name,
                self.inst_name.as_ref().unwrap()
            ));
        }

        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports {
                res.extend(port.to_inst_string(false));
            }
            res.extend(last_port.to_inst_string(true));
            res.push(");".to_string());
        } else {
            log::error!("There is no port in module {}", self.module_name);
        }
        res
    }

    pub fn to_module_string(&self) -> Vec<String> {
        let mut res = Vec::new();
        let mut indent = 0;
        res.push(format!("module {} (", self.module_name));

        indent += 4;

        // port info
        if let Some((last_port, ports)) = self.port_list.split_last() {
            for port in ports.iter() {
                res.extend(port.to_port_string(false));
            }
            
            res.extend(last_port.to_port_string(true));

            res.push(");\n".to_string());
        }

        // wire definition
        let s = WireBuilder::traverse_unport_wires()
            .iter()
            .flat_map(|w| {
                w.to_string()
            })
            .collect::<Vec<_>>();
        res.extend(s.into_iter().map(|s| format!("{}{}", " ".repeat(indent), s)).collect::<Vec<String>>());
        res.push("\n".to_string());

        // port wire connected
        let temp = self.port_list.iter()
            .filter_map(|item| {
                item.to_assign_string()
            })
            .flatten()
            .collect::<Vec<_>>();
        res.extend(temp);
        res.push("\n".to_string());

        // inst info
        for inst in self.inst_list.iter() {
            res.extend(inst
                .borrow()
                .to_inst_string()
                .into_iter()
                .map(|s| format!("{}{}", " ".repeat(indent), s))
                .collect::<Vec<String>>()
            );
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
    use crate::verilog::data::WrapMacro;
    use crate::verilog::module::VerilogModule;
    use crate::verilog::port::{PortDir, VerilogPort};

    #[test]
    fn test_inst_string() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        let mut module = VerilogModule::new("test".to_string());
        module.fix_inst_name("u_test_module");
        let mut port1 = VerilogPort::new(PortDir::InPort, "port1", 12.into());
        port1.set_info_msg("test1 info message");
        port1.connect_partial_signal("wire1", &(0..4), false);
        port1.connect_partial_signal("wire2", &(0..5), false);
        port1.connect_partial_signal("wire3", &(0..3), false);
        println!("{:?}", port1.signals);
        let mut port2 = VerilogPort::new(PortDir::InPort, "port2", 12.into());
        port2.connect_undefined_signal("undefined-wires", false);
        let mut port3 = VerilogPort::new(
            PortDir::OutPort,
            "pordddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddt3",
            12.into(),
        );
        port3.connect_number_signal(43, 8);
        module.add_ports(vec![port1.wrap_raw(), port2.wrap_raw(), port3.wrap_raw()]);
        println!("{}", module.to_inst_string().join("\n"));
    }
}
