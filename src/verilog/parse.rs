use std::collections::HashMap;
use std::path::PathBuf;
use sv_parser::{parse_sv, unwrap_node, Define, Defines, ModuleDeclarationAnsi, PortDeclaration, RefNode, SyntaxTree};
use crate::verilog::module::VerilogModule;
use crate::verilog::port::{PortDir, VerilogPort};

#[derive(Default)]
struct VerilogParser {
    file: PathBuf,
    defines: HashMap<String, Option<Define>>,
    includes: Vec<PathBuf>,

    parse_res: Option<SyntaxTree>,
    module_info: Vec<VerilogModule>,
}

impl VerilogParser {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            ..Default::default()
        }
    }

    pub fn add_define(mut self, define: Define) -> Self {
        todo!("不知道hashmap的键值是啥");
        self
    }

    pub fn add_includes(mut self, includes: Vec<PathBuf>) -> Self {
        self.includes.extend(includes);
        self
    }

    pub fn parse(mut self) -> Self {
        let res = parse_sv(
            &self.file,
            &self.defines,
            &self.includes,
            false,
            false,
        );
        match res {
            Ok(t) => {
                log::info!("file {} parsed successfully", self.file.display());
                self.parse_res = Some(t.0)
            }
            Err(e) => {
                log::error!("file {} parse error: {:?}", self.file.display(), e);
            }
        }
        self
    }
    pub fn solve(mut self) -> Self {
        self.extract_module();

        self
    }

    pub fn get_module_info(self) -> Vec<VerilogModule> {
        self.module_info
    }

    pub fn extract_module(&mut self) {
        if self.parse_res.is_none() {
            log::error!("cannot extract module");
        }
        log::info!("start extract module");
        let tree = self.parse_res.as_ref().unwrap();
        for node in tree {
            match node {
                RefNode::ModuleDeclarationNonansi(module_node) => {
                    let module_id_node = unwrap_node!(module_node, ModuleIdentifier).unwrap();
                    let module_name = self.get_identifier_string(module_id_node.clone()).unwrap_or_else(|| {
                        log::error!("Can not extract module name");
                        "".into()
                    });
                    let mut module = VerilogModule::new(module_name);
                    module.add_ports(self.extract_ports(RefNode::from(module_node)));

                    // TODO add inst

                    // add module
                    self.module_info.push(module);


                },
                RefNode::ModuleDeclarationAnsi(module) => {
                    todo!()
                },
                _ => {}
            }
        }
        log::info!("end extract module");
    }

    fn extract_ports(&self, module_node: RefNode) -> Vec<VerilogPort> {
        log::info!("start extract ports");
        let mut port_list = Vec::new();
        for item in module_node.into_iter().flatten() {
            // println!("Node is {}", item);
            if let RefNode::Locate(t) = item {
                // println!("locate is {:?}", t);
            }
            if let RefNode::PortDeclaration(port_dir) = item {
                println!("        ++++++++");
                //port direction
                let inout = Self::get_direction(port_dir);

                //port width
                let width = 1;  //TODO  self.get_port_width()

                // port name
                for port_node in unwrap_node!(RefNode::from(port_dir), ListOfPortIdentifiers).into_iter().flatten() {
                    if let RefNode::PortIdentifier(t) = port_node {
                        let port_name = self.get_identifier_string(RefNode::from(t))
                            .unwrap_or_else(|| {
                                log::error!("Can not extract port name");
                                "".into()
                            });
                        let port_inst = VerilogPort::new(
                            inout.clone(),
                            &port_name,
                            width
                        );
                        port_list.push(port_inst);
                    }
                }
            }
        }
        port_list
    }


    fn get_port_width(&self, port_node: RefNode) -> Option<usize> {
        todo!()
    }
    fn get_direction(dir: &PortDeclaration) -> PortDir {
        match dir {
            PortDeclaration::Inout(_) => PortDir::InOutPort,
            PortDeclaration::Input(_) => PortDir::InPort,
            PortDeclaration::Output(_) => PortDir::OutPort,
            PortDeclaration::Ref(_) => PortDir::Unknown,
            PortDeclaration::Interface(_) => PortDir::Unknown,
        }
    }
    fn get_identifier_string(&self, node:RefNode) -> Option<String> {
        let locate = match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
            Some(RefNode::SimpleIdentifier(x)) => {
                Some(x.nodes.0)
            }
            Some(RefNode::EscapedIdentifier(x)) => {
                Some(x.nodes.0)
            }
            _ => None,
        }?;
        let t= self.parse_res
            .as_ref()
            .unwrap()
            .get_str(&locate);
        println!("        ****{}", t.unwrap());
        t
            .map(|s| s.to_string())
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_base() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        let module_info = VerilogParser::new("./test/std-7.1.6-primitives.v".into())
            .parse()
            .solve()
            .get_module_info();
        for m in module_info {
            println!("Module ---------------------");
            println!("{:#?}", m);
            println!("module port number is {}", m.port_list.len())
        }
    }
}