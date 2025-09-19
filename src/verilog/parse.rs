use crate::verilog::module::VerilogModule;
use crate::verilog::port::{PortDir, VerilogPort};
use std::collections::HashMap;
use std::fs::File;
use std::{usize};
use std::io::Write;
use std::path::PathBuf;
use sv_parser::{ConstantExpression, Define, PortDeclaration, PortDirection, RefNode, SyntaxTree, parse_sv, unwrap_node};
use crate::verilog::parameter::Param;
use crate::verilog::width::Width;
use crate::verilog::width::Width::RawWidth;

pub struct VerilogParser<'a> {
    file: &'a PathBuf,
    defines: HashMap<String, Option<Define>>,
    includes: Vec<PathBuf>,

    parse_res: Option<SyntaxTree>,
    module_info: Vec<VerilogModule>,
}

impl<'a> VerilogParser<'a> {
    pub fn new(file: &'a PathBuf) -> Self {
        Self {
            file,
            defines: HashMap::new(),
            includes: Vec::new(),
            
            parse_res: None,
            module_info: Vec::new(),
        }
    }

    pub fn add_define(self, _define: Define) -> Self {
        todo!("不知道hashmap的键值是啥");
        self
    }

    pub fn add_includes(mut self, includes: Vec<PathBuf>) -> Self {
        self.includes.extend(includes);
        self
    }

    pub fn parse(mut self) -> Self {
        let res = parse_sv(&self.file, &self.defines, &self.includes, false, false);
        match res {
            Ok(t) => {
                log::info!("file {} parsed successfully", self.file.display());
                self.parse_res = Some(t.0)
            }
            Err(e) => {
                panic!("file {} parse error: {:?}", self.file.display(), e)
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
        log::debug!("start extract module");

        let tree = self.parse_res.as_ref().unwrap();

        #[cfg(debug_assertions)]
        {
            let mut file = File::create("dump-tree.txt").unwrap();
            writeln!(file, "{}", tree).unwrap();
        }

        for node in tree {
            match node {
                RefNode::ModuleDeclarationNonansi(module_node) => {
                    let module_id_node = unwrap_node!(module_node, ModuleIdentifier).unwrap();
                    let module_name = self
                        .get_identifier_string(module_id_node.clone())
                        .unwrap_or_else(|| {
                            log::error!("Can not extract module name");
                            "".into()
                        });
                    let mut module = VerilogModule::new(module_name);

                    // add parameter list
                    let params = self.extract_params(RefNode::from(module_node));

                    //add port
                    module.add_ports(self.extract_ports(RefNode::from(module_node), &params));

                    // add parameter list
                    module.add_param_list(params);

                    // TODO add inst

                    // add module
                    self.module_info.push(module);
                }
                RefNode::ModuleDeclarationAnsi(module_node) => {
                    let module_id_node = unwrap_node!(module_node, ModuleIdentifier).unwrap();
                    let module_name = self
                        .get_identifier_string(module_id_node.clone())
                        .unwrap_or_else(|| {
                            log::error!("Can not extract module name");
                            "".into()
                        });
                    let mut module = VerilogModule::new(module_name);

                    // add parameter list
                    let params = self.extract_ansi_params(RefNode::from(module_node));

                    // add ports
                    module.add_ports(self.extract_ansi_ports(RefNode::from(module_node), &params));

                    // add parameter list
                    module.add_param_list(params);

                    // TODO add inst

                    // add module
                    self.module_info.push(module);
                }
                _ => {}
            }
        }
        log::debug!("end extract module");
    }

    fn extract_ansi_params(&self, module_node: RefNode) -> Vec<Param> {
        log::debug!("start extract ansi parameters");
        let mut params = Vec::new();

        for para_node in unwrap_node!(module_node, ParameterPortListDeclaration)
            .into_iter()
            .flatten()
        {
            if let RefNode::ParameterDeclaration(para) = para_node {
                let token = if let Some(id) = unwrap_node!(para, ParameterIdentifier) {
                    self.get_identifier_string(id)
                } else { None };
                let value = if let Some(t) = unwrap_node!(para, ConstantExpression) {
                    self.get_literal_string(t)
                } else { None };
                if token.is_some() && value.is_some() {
                    log::debug!("Find parameter token is {:?}, value is {:?}", token, value);
                    params.push(Param::new(token.unwrap(), value.unwrap().width()));
                } else {
                    log::debug!("Error find parameter token is {:?}, value is {:?}", token, value);
                }
            }
        }
        params
    }

    fn extract_params(&self, node: RefNode) -> Vec<Param> {
        log::debug!("start extract non-ansi params");
        let mut params = Vec::new();
        for item in node.into_iter() {
            if let RefNode::ParameterDeclaration(para) = item {
                let token = if let Some(id) = unwrap_node!(para, ParameterIdentifier) {
                    self.get_identifier_string(id)
                } else { None };
                let value = if let Some(t) = unwrap_node!(para, ConstantExpression) {
                    self.get_literal_string(t)
                } else { None };
                if token.is_some() && value.is_some() {
                    log::debug!("Find parameter token is {:?}, value is {:?}", token, value);
                    params.push(Param::new(token.unwrap(), value.unwrap().width()));
                } else {
                    log::debug!("Error find parameter token is {:?}, value is {:?}", token, value);
                }
            }
        }
        params
    }

    fn extract_ports(&self, module_node: RefNode, params: &Vec<Param>) -> Vec<VerilogPort> {
        log::debug!("start non-ansi extract ports");
        let mut port_list = Vec::new();
        for item in module_node.into_iter() {
            if let RefNode::PortDeclaration(port_dir) = item {
                //port direction
                let inout = Self::get_direction(port_dir);

                //port width
                let width = self.get_port_width(RefNode::from(port_dir));
                let width = width.width_from(params);

                // port name
                for port_node in unwrap_node!(port_dir, ListOfPortIdentifiers)
                    .into_iter()
                    .flatten()
                {
                    if let RefNode::PortIdentifier(t) = port_node {
                        let port_name = self
                            .get_identifier_string(RefNode::from(t))
                            .unwrap_or_else(|| {
                                log::error!("Can not extract port name");
                                "".into()
                            });
                        let port_inst = VerilogPort::new(inout.clone(), &port_name, width.clone());
                        port_list.push(port_inst);
                    }
                }
            }
        }
        log::debug!("end  non-ansi extract ports");
        port_list
    }

    fn extract_ansi_ports(&self, module_node: RefNode, params: &Vec<Param>) -> Vec<VerilogPort> {
        log::debug!("start extract ansi ports");
        let mut port_list = Vec::new();
        for item in module_node.into_iter() {
            if let RefNode::AnsiPortDeclaration(port_dir) = item {
                let inout = if let Some(RefNode::PortDirection(dir)) =
                    unwrap_node!(port_dir, PortDirection)
                {
                    Self::get_ansi_direction(dir)
                } else {
                    log::error!("Can not extract ansi port direction");
                    PortDir::Unknown
                };

                let width = self.get_port_width(RefNode::from(port_dir));
                let width = width.width_from(params);

                let port_name = if let Some(id) = unwrap_node!(port_dir, PortIdentifier) {
                    self.get_identifier_string(id).unwrap_or_else(|| {
                        log::warn!("Can not extract port name");
                        "".into()
                    })
                } else {
                    log::error!("Can not extract ansi port name");
                    "".into()
                };
                log::debug!("extract port name is {}", port_name);

                let port_inst = VerilogPort::new(inout, &port_name, width);
                port_list.push(port_inst);
            }
        }
        port_list
    }

    fn get_port_width(&self, port_node: RefNode) -> Width {
        log::debug!("extract port width >>>");
        if let Some(range) = unwrap_node!(port_node, PackedDimension) {
            log::debug!("find node {:?}", range);
            if let Some(RefNode::ConstantRange(range)) = unwrap_node!(range, ConstantRange) {
                let upper = self.extract_expr(&range.nodes.0);
                let lower = self.extract_expr(&range.nodes.2);

                log::debug!("port range upper: {:?} and lower: {:?}", upper, lower);
                upper - lower + 1
            } else {
                log::debug!("[extract width] Cannot find node ConstantRange");
                RawWidth(1)
            }
        } else {
            RawWidth(1)
        }
    }

    fn extract_expr(&self, expr: &ConstantExpression) -> Width {
        match unwrap_node!(
            expr,
            ConstantPrimary,
            ConstantExpressionBinary,
            ConstantExpressionUnary,
            ConstantExpressionTernary
        ) {
            Some(RefNode::ConstantPrimary(t)) => self
                .get_literal_string(RefNode::from(t))
                .unwrap_or_else(|| {
                    log::debug!("Cannot extract ConstantPrimary");
                    "".into()
                }),
            Some(RefNode::ConstantExpressionBinary(t)) => {
                let left = self.extract_expr(&t.nodes.0);
                let right = self.extract_expr(&t.nodes.3);
                let op = self
                    .get_operator_string(RefNode::from(&t.nodes.1))
                    .unwrap_or_else(|| {
                        log::error!("Can not extract operator");
                        "".into()
                    });
                match (&left, &right) { 
                    (RawWidth(x), RawWidth(y)) => {
                        match op.as_str() {
                            "+" => RawWidth(x+y),
                            "-" => RawWidth(x-y),
                            _ => format!("{}{}{}", left, op, right).into()
                        }
                    }  
                    _ => format!("{}{}{}", left, op, right).into()
                }
            }
            Some(RefNode::ConstantExpressionUnary(t)) => {
                log::warn!("Not Support ConstantExpressionUnary");
                "".into()
            }
            Some(RefNode::ConstantExpressionTernary(t)) => {
                log::warn!("Not Support ConstantExpressionTernary");
                "".into()
            }
            _ => {
                log::warn!("Not Support Expression");
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

    fn get_ansi_direction(dir: &PortDirection) -> PortDir {
        match dir {
            PortDirection::Input(_) => PortDir::InPort,
            PortDirection::Output(_) => PortDir::OutPort,
            PortDirection::Inout(_) => PortDir::InOutPort,
            PortDirection::Ref(_) => PortDir::Unknown,
        }
    }

    fn get_literal_string(&self, node: RefNode) -> Option<Width> {
        match unwrap_node!(node, DecimalNumber, BinaryNumber, HexNumber, OctalNumber, PsOrHierarchicalTfIdentifier) {
            Some(RefNode::DecimalNumber(n)) => self.get_dec_number_string(RefNode::from(n)),
            Some(RefNode::BinaryNumber(n)) => self.get_bin_number_string(RefNode::from(n)),
            Some(RefNode::HexNumber(n)) => self.get_hex_number_string(RefNode::from(n)),
            Some(RefNode::OctalNumber(n)) => {
                log::debug!("cannot support OctalNumber");
                None
            }
            Some(RefNode::PsOrHierarchicalTfIdentifier(n)) => 
                self.get_identifier_string(RefNode::from(n)).and_then(|x| Some(x.into())),
            _ => None,
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
        } else {
            None
        }
    }

    fn get_dec_number_string(&self, node: RefNode) -> Option<Width> {
        if let Some(RefNode::UnsignedNumber(number)) = unwrap_node!(node, UnsignedNumber) {
            let locate = number.nodes.0;
            self.parse_res
                .as_ref()
                .unwrap()
                .get_str(&locate)
                .map(|s| s.parse::<usize>().unwrap().into())
        } else {
            None
        }
    }

    fn get_bin_number_string(&self, node: RefNode) -> Option<Width> {
        if let Some(RefNode::BinaryNumber(number)) = unwrap_node!(node, BinaryNumber) {
            let locate = number.nodes.2.nodes.0;
            self.parse_res.as_ref().unwrap().get_str(&locate).map(|s| {
                usize::from_str_radix(s, 2).unwrap_or_else(|e| {
                    log::error!("Can not extract binary number: {}", e);
                    1
                }).into()
            })
        } else {
            None
        }
    }

    fn get_hex_number_string(&self, node: RefNode) -> Option<Width> {
        if let Some(RefNode::HexNumber(number)) = unwrap_node!(node, HexNumber) {
            let locate = number.nodes.2.nodes.0;
            self.parse_res.as_ref().unwrap().get_str(&locate).map(|s| {
                usize::from_str_radix(s, 16).unwrap_or_else(|e| {
                    log::error!("Can not extract hex number: {}", e);
                    1
                }).into()
            })
        } else {
            None
        }
    }

    fn get_identifier_string(&self, node: RefNode) -> Option<String> {
        let locate = match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
            Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
            Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
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
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        // let module_info = VerilogParser::new(&PathBuf::from("./test/npu_afifo_r.sv"))
        let module_info = VerilogParser::new(&PathBuf::from("./test/std-7.1.6-primitives.v"))
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
