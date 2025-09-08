use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use sv_parser::{parse_sv, unwrap_node, ConstantExpression, Define, Defines, IntegralNumber, ModuleDeclarationAnsi, PortDeclaration, RefNode, SyntaxTree};
use crate::verilog::module::VerilogModule;
use crate::verilog::port::{PortDir, VerilogPort};
use crate::utils::calculator::StrCalc;

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

        let mut file = File::create("dump-tree.txt").unwrap();
        writeln!(file, "{}", tree).unwrap();

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
        for item in module_node.into_iter() {
            if let RefNode::PortDeclaration(port_dir) = item {
                //port direction
                let inout = Self::get_direction(port_dir);

                //port width
                let width = self.get_port_width(RefNode::from(port_dir)).unwrap_or_default();

                // port name
                for port_node in unwrap_node!(port_dir, ListOfPortIdentifiers).into_iter().flatten() {
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
        log::info!("end extract ports");
        port_list
    }


    fn get_port_width(&self, port_node: RefNode) -> Option<usize> {
        log::info!("extract port width");
        if let Some(range) = unwrap_node!(port_node, PackedDimension) {
            log::info!("find node {:?}", range);
            if let Some(RefNode::ConstantRange(range)) = unwrap_node!(range, ConstantRange) {
                let upper = self.extract_expr(&range.nodes.0)
                    .calculate();
                let lower = self.extract_expr(&range.nodes.2)
                    .calculate();

                log::info!("port range upper: {:?} and lower: {:?}", upper, lower);

                Some(upper - lower + 1)
            } else {
                log::info!("cannot find node ConstantRange");
                None
            }

        } else {
            Some(1)
        }
    }

    fn extract_expr(&self, expr: &ConstantExpression) -> String {
        match unwrap_node!(expr, ConstantPrimary, ConstantExpressionBinary, ConstantExpressionUnary, ConstantExpressionTernary) {
            Some(RefNode::ConstantPrimary(t)) => {
                self.get_literal_string(RefNode::from(t)).unwrap_or_else(
                    || {
                        log::info!("Cannot extract ConstantPrimary");
                        "".into()
                    }
                )
            }
            Some(RefNode::ConstantExpressionBinary(t)) => {
                let left = self.extract_expr(&t.nodes.0);
                let right = self.extract_expr(&t.nodes.3);
                let op = self.get_operator_string(RefNode::from(&t.nodes.1)).unwrap_or_else(||{
                    log::error!("Can not extract operator");
                    "".into()
                });
                format!("{}{}{}", left, op, right)
            }
            Some(RefNode::ConstantExpressionUnary(t)) => {
                log::info!("Not Support ConstantExpressionUnary");
                "".into()
            }
            Some(RefNode::ConstantExpressionTernary(t)) => {
                log::info!("Not Support ConstantExpressionTernary");
                "".into()
            }
            _ => {
                log::info!("Not Support Expression");
                "".into()
            }
        }
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

    fn get_literal_string(&self, node:RefNode) -> Option<String> {
        match unwrap_node!(node, DecimalNumber, BinaryNumber, HexNumber, OctalNumber) {
            Some(RefNode::DecimalNumber(n)) => {self.get_dec_number_string(RefNode::from(n))},
            Some(RefNode::BinaryNumber(n)) => {self.get_bin_number_string(RefNode::from(n))},
            Some(RefNode::HexNumber(n)) => {self.get_hex_number_string(RefNode::from(n))},
            Some(RefNode::OctalNumber(n)) => {
                log::info!("cannot support OctalNumber");
                None
            }
            _ => {None}
        }
    }

    fn get_operator_string(&self, node: RefNode) -> Option<String> {
        if let Some(RefNode::BinaryOperator(t)) = unwrap_node!(node, BinaryOperator) {
            let locate = t.nodes.0.nodes.0;
            self.parse_res
                .as_ref()
                .unwrap()
                .get_str(&locate)
                .map(|s| s.to_string())
        } else { None }
    }

    fn get_dec_number_string(&self, node:RefNode) -> Option<String> {
        if let Some(RefNode::UnsignedNumber(number)) = unwrap_node!(node, UnsignedNumber) {
            let locate = number.nodes.0;
            self.parse_res
                .as_ref()
                .unwrap()
                .get_str(&locate)
                .map(|s| s.to_string())
        } else { None }
    }

    fn get_bin_number_string(&self, node:RefNode) -> Option<String> {
        if let Some(RefNode::BinaryNumber(number)) = unwrap_node!(node, BinaryNumber) {
            let locate = number.nodes.2.nodes.0;
            self.parse_res
                .as_ref()
                .unwrap()
                .get_str(&locate)
                .map(|s| format!("{}", i32::from_str_radix(s, 2).unwrap_or_else(|e| {
                    log::error!("Can not extract binary number: {}", e);
                    1
                })))
        } else { None }
    }

    fn get_hex_number_string(&self, node:RefNode) -> Option<String> {
        if let Some(RefNode::HexNumber(number)) = unwrap_node!(node, HexNumber) {
            let locate = number.nodes.2.nodes.0;
            self.parse_res
                .as_ref()
                .unwrap()
                .get_str(&locate)
                .map(|s| format!("{}", i32::from_str_radix(s, 16).unwrap_or_else(|e| {
                    log::error!("Can not extract hex number: {}", e);
                    1
                })))
        } else { None }
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
        self.parse_res
            .as_ref()
            .unwrap()
            .get_str(&locate)
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