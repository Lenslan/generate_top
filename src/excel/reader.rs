use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use calamine::{Data, Range, Reader};
use regex::Regex;
use crate::verilog::module::VerilogModule;
use crate::verilog::port::{PortDir, UndefineWireCollector, VerilogPort};
use crate::verilog::wire::WireBuilder;

pub struct ExcelReader {
    path: PathBuf,
}

impl ExcelReader {

    ///
    /// 指定excel的路径
    ///
    pub fn new(path: PathBuf) -> Self {
        ExcelReader { path }
    }

    pub fn generate_v(&self) {
        let mut module = self.get_excel_info();
        module.final_check();
        let parent_path = self.path.parent().expect("Could not get parent path");
        let module_name = self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Could not get module name");
        let top_path = parent_path.join(format!("{}.v", module_name));
        let mut file = File::create(top_path).unwrap();
        file.write_all(&module.to_module_string().join("\n").as_bytes()).unwrap();

    }

    pub fn get_excel_info(&self) -> VerilogModule {
        log::debug!("Start extract excel file {}", self.path.display());
        let mut workbook = calamine::open_workbook_auto(&self.path).unwrap();
        let sheets = workbook.sheet_names().to_owned();
        if sheets.len() == 0 {
            log::error!("excel is empty");
            std::process::exit(1);
        }

        UndefineWireCollector::clear();
        WireBuilder::clear();

        let module_name = &sheets[0];
        let mut module = VerilogModule::new(sheets[0].clone());
        // extract module ports
        if let Ok(range) = workbook.worksheet_range(module_name) {
            let (port_list, inst_name) = Self::extract_port(&range);
            module.add_ports(port_list);
            if let Some(s) = inst_name {
                module.fix_inst_name(s);
            }
        }
        module.port_list.iter_mut().for_each(|p| p.register_port_as_wire());

        // extract inst module
        for inst_name in sheets[1..].iter() {
            log::debug!("Extracting sheet {}", inst_name);
            let mut inst_module = VerilogModule::new(String::from(inst_name));
            if let Ok(range) = workbook.worksheet_range(inst_name) {
                let (port_list, inst_name) = Self::extract_port(&range);
                inst_module.add_ports(port_list);
                if let Some(s) = inst_name {
                    inst_module.fix_inst_name(s);
                }
                inst_module.port_list.iter_mut().for_each(|p| p.check_health());
                module.add_inst_module(Arc::new(RefCell::new(inst_module)));
            }
        }
        
        // final check
        // dont exec this function, do it by function caller
        // module.final_check();

        log::debug!("end extract excel file {}", self.path.display());


        module
    }

    fn extract_string(data: Option<&Data>) -> Option<String> {
        match data {
            Some(Data::String(s)) => Some(s.clone()),
            _ => None
        }
    }

    fn extract_width(data: Option<&Data>) -> u32 {
        match data {
            Some(Data::Int(n)) => n.clone() as u32,
            Some(Data::String(s)) => s.parse().unwrap(),
            Some(Data::Float(n)) => n.clone() as u32,
            _ => 0
        }
    }

    fn extract_inout(data: Option<&Data>) -> PortDir {
        match data {
            Some(Data::String(s)) => s.into(),
            _ => PortDir::Unknown
        }
    }

    fn extract_wires(data: Option<&Data>) -> Vec<String> {
        match data {
            Some(Data::String(s)) => {
                s.split(|c| c == ',' || c == ' ')
                    .filter_map(|x| if x.is_empty() {None} else {Some(String::from(x))})
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn match_wires_by_re(port: &mut VerilogPort, wires: Vec<String>) {
        let name_re = Regex::new(r"\b[a-zA-Z_]\w*\b").unwrap();
        let name_range_re = Regex::new(r"(\b[a-zA-Z_]\w*\b)\s*\[\s*(\d+)\s*:\s*(\d+)\s*]").unwrap();
        let number_re = Regex::new(r"(\d+)'\s*([bodh])\s*([0-9a-fA-F_xzXZ]+)").unwrap();

        for wire in wires {
            log::debug!("Match wire `{}`:", wire);
            if let Some(s) = name_range_re.captures(&wire) {
                let name = s.get(1).unwrap().as_str();
                let range_end = s.get(2).unwrap().as_str().parse::<usize>().unwrap();
                let range_start = s.get(3).unwrap().as_str().parse::<usize>().unwrap();
                port.connect_partial_signal(name, &(range_start..(range_end+1)));
                log::debug!("=> Match range {}[{}:{}]", name, range_end, range_start);
            } else if let Some(s) = number_re.captures(&wire) {
                let width = s.get(1).unwrap().as_str().parse::<u8>().unwrap();
                let base = match s.get(2).unwrap().as_str() {
                    "b" => 2,
                    "o" => 8,
                    "h" => 16,
                    _ => 10
                };
                let val = u128::from_str_radix(s.get(3).unwrap().as_str(), base).unwrap();
                port.connect_number_signal(val, width);
                log::debug!("=> Match number {}'d{}", width, val);
            } else if let Some(s) = name_re.find(&wire) {
                let name = s.as_str();
                port.connect_undefined_signal(name);
                log::debug!("=> Match name {}",name);
            }
        }
    }

    /// extract message from one sheet
    /// return Portlist & inst_name
    fn extract_port(range: &Range<Data>) -> (Vec<VerilogPort>, Option<&String>) {
        let mut port_list = Vec::new();
        let mut inst_name = None;
        for (row_idx, row_data) in  range.rows().enumerate() {
            if row_idx == 0 {
                if let Some(Data::String(s)) = row_data.get(1) {
                    inst_name = Some(s);
                }
            }
            if row_idx > 1 {
                let port_name = Self::extract_string(row_data.get(0));
                if port_name.is_none() { continue }
                let inout = Self::extract_inout(row_data.get(1));
                let width = Self::extract_width(row_data.get(2));
                let wire_name = Self::extract_wires(row_data.get(3));
                let port_info = Self::extract_string(row_data.get(4));

                let mut new_port = VerilogPort::new(inout, &port_name.unwrap(), width as usize);
                if let Some(s) = port_info {
                    new_port.set_info_msg(&s);
                }
                Self::match_wires_by_re(&mut new_port, wire_name);
                // Dont exec check_health() function, used by the function caller
                // new_port.check_health();

                port_list.push(new_port);
            }
        }
        (port_list, inst_name)
    }

}


#[cfg(test)]
mod test {
    use crate::excel::reader::ExcelReader;
    use crate::verilog::port::{PortDir, VerilogPort};
    use crate::verilog::wire::WireBuilder;

    // #[test]
    fn test_re() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        let mut  port = VerilogPort::new(PortDir::InPort, "test_port", 32);
        let test_vec = vec![
            "testwire1".to_string(),
            "testwire2[3:0]".to_string(),
            "testwire3[1:0]".to_string(),
            "3'b101".to_string(),
            "10'd34".to_string(),
            "8'ha9".to_string()
        ];
        ExcelReader::match_wires_by_re(&mut port, test_vec);
        println!("{}", port.to_inst_string(1,1));

    }

    #[test]
    fn test_excel() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        let file = ExcelReader::new("src/excel/test/uart.xlsx".into());
        file.generate_v();
        // let module = file.get_excel_info();
        // WireBuilder::builder_show();
        // println!("{:#?}", module);
    }
}
