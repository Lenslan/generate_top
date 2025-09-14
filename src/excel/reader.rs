use std::path::PathBuf;
use std::sync::Arc;
use calamine::{Data, Reader};
use walkdir::WalkDir;
use crate::verilog::module::VerilogModule;
use crate::verilog::port::{PortDir, VerilogPort};

pub struct ExcelReader {
    path: PathBuf,
}

impl ExcelReader {
    pub fn new(path: PathBuf) -> Self {
        ExcelReader { path }
    }

    pub fn generate_v(&self) {}

    pub fn get_excel_info(&self) -> VerilogModule {
        let mut workbook = calamine::open_workbook_auto(&self.path).unwrap();
        let sheets = workbook.sheet_names().to_owned();
        if sheets.len() == 0 {
            log::error!("excel is empty");
            std::process::exit(1);
        }
        let mut module = VerilogModule::new(sheets[0].clone());
        // todo process module ports

        for inst_name in sheets[1..].iter() {
            let mut inst_module = VerilogModule::new(String::from(inst_name));
            let mut port_list = Vec::new();
            if let Ok(range) = workbook.worksheet_range(inst_name) {
                for (row_idx, row_data) in  range.rows().enumerate() {
                    if row_idx == 0 {
                        let inst_name = match row_data.get(1) {
                            Some(Data::String(s)) => s.clone(),
                            _ => String::new(),
                        };
                        inst_module.fix_inst_name(&inst_name);
                    }
                    if row_idx > 1 {
                        let port_name = Self::extract_string(row_data.get(0));
                        if port_name.is_none() { continue }
                        let inout = Self::extract_inout(row_data.get(1));
                        let width = Self::extract_width(row_data.get(2));
                        let wire_name = Self::extract_wires(row_data.get(3));
                        let port_info = Self::extract_string(row_data.get(4));

                        let new_port = VerilogPort::new(inout, &port_name.unwrap(), width as usize);
                        // TODO 怎么处理连接的信号呢，有字符串，字符串带range，纯常数
                    }
                }
            }
            inst_module.add_ports(port_list);
            module.add_inst_module(Arc::new(inst_module));
        }


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

}
