use crate::excel::reader::ExcelReader;
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;
use crate::verilog::module::VerilogModule;
use crate::verilog::parse::VerilogParser;
use crate::verilog::port::UndefineWireCollector;
use crate::verilog::wire::WireBuilder;

#[derive(Default)]
struct ExcelWriter {
    module_dir_path: PathBuf,
    file_list: Vec<PathBuf>,
}

impl ExcelWriter {
    fn new(module_dir_path: PathBuf) -> Self {
        Self {
            module_dir_path,
            ..Default::default()
        }
    }

    fn generate(&self) {
        let parent_path = self.module_dir_path.parent().expect("Could not get parent path");
        let module_name = self.module_dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("Could not get module name");

        UndefineWireCollector::clear();
        WireBuilder::clear();
        let mut module = VerilogModule::new(module_name.into());

        for f in self.file_list.iter() {
            let inst_module = VerilogParser::new(f).parse().solve().get_module_info();
            for mut inst_item in inst_module {
                inst_item.set_default_inst_name();
                inst_item.set_default_port_wires();
                module.add_inst_module(Arc::new(inst_item));
            }
        }
        
        // 遍历wire builder 将所有没有驱动/没有load的信号连接到端口
    }

    fn generate_or_update(&self) {}

    fn traverse_v(&mut self) {
        let mut dir_list = Vec::new();
        let mut excel_list = Vec::new();
        for entry in WalkDir::new(&self.module_dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                dir_list.push(entry.into_path());
            } else {
                let extension = entry.path().extension().unwrap_or_default();
                if extension == "v" || extension == "sv" {
                    self.file_list.push(entry.clone().into_path());
                }
                if extension == "xlsx" {
                    excel_list.push(entry.clone().into_path());
                }
            }
        }

        for d in dir_list {
            ExcelWriter::new(d.clone()).generate_or_update();
            let parent = d.parent().expect("Can not get parent name");
            let file_name = d
                .file_name()
                .and_then(|s| s.to_str())
                .expect("Can not get file name");
            let excel_name = parent.join(format!("{}.xlsx", file_name));
            ExcelReader::new(excel_name).generate_v();
            let file_v = parent.join(format!("{}.v", file_name));
            self.file_list.push(file_v)
        }
    }
}
