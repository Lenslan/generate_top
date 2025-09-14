use crate::excel::reader::ExcelReader;
use std::path::PathBuf;
use std::sync::Arc;
use rust_xlsxwriter::{ColNum, Color, Format, FormatAlign, FormatBorder, RowNum, Workbook, Worksheet};
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

        let mut workbook = Workbook::new();
        let excel_name = parent_path.join(format!("{}.xlsx", module_name));
        log::debug!("start generate excel file {}", excel_name.display());

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
        for (inout, width, name) in WireBuilder::traverse_unload_undriven() {
            module.add_port(inout, &name, width as u32)
        }

        // write excel
        workbook.push_worksheet(self.add_inst_sheet(&module));
        for item in module.inst_list.iter() {
            workbook.push_worksheet(self.add_inst_sheet(item));
        }
        workbook.save(excel_name).unwrap();
    }

    fn generate_or_update(&self) {}

    fn traverse_v(&mut self) {
        log::debug!("Traversing verilog files in dir {}", self.module_dir_path.display());
        let mut dir_list = Vec::new();
        let mut excel_list = Vec::new();
        for entry in WalkDir::new(&self.module_dir_path)
            .min_depth(1)
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
            log::debug!("dir list is  {}", d.display());
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

        // debug
        for item in self.file_list.iter() {
            log::debug!("file list is {}", item.display());
        }
    }

    // TODO how to write inst name
    fn add_inst_sheet(&self, module: &VerilogModule) -> Worksheet {
        let mut sheet = Worksheet::new();
        let header_format = Format::new()
            .set_bold()
            .set_font_size(16)
            .set_align(FormatAlign::Center)
            .set_border_bottom(FormatBorder::Medium)
            .set_border_top(FormatBorder::Medium)
            .set_background_color(Color::Gray);
        let number_format = Format::new()
            .set_align(FormatAlign::Left);
        let title_list = ["Port-name", "InOut", "Width", "Wire-name", "Port-info"];
        let width_list = [30, 10, 10, 30, 40];

        sheet.set_name(&module.module_name).unwrap();
        sheet.set_row_height(0, 18).unwrap();
        sheet.set_row_height(1, 20).unwrap();

        sheet.write(0, 0, "Module Inst Name").unwrap();
        sheet.write(0, 1, format!("{}", module.inst_name.as_deref().unwrap_or_default())).unwrap();

        for item in title_list.into_iter().enumerate() {
            sheet.write_with_format(1, item.0 as ColNum, item.1, &header_format).unwrap();
            sheet.set_column_width(item.0 as ColNum, width_list[item.0]).unwrap();
        }
        for (idx, port) in module.port_list.iter().enumerate() {
            sheet.write((idx + 2) as RowNum, 0, &port.name).unwrap();
            sheet.write((idx + 2) as RowNum, 1, format!("{}", port.inout)).unwrap();
            sheet.write_with_format((idx + 2) as RowNum, 2, port.width as u32, &number_format).unwrap();
            sheet.write((idx + 2) as RowNum, 3, port.get_signal_string()
                .replace('{',"")
                .replace('}',"")
            ).unwrap();
            sheet.write((idx + 2) as RowNum, 4, &port.info).unwrap();
            sheet.set_row_height((idx + 2) as RowNum, 16).unwrap();
        }

        sheet.set_freeze_panes(2, 0).unwrap();
        sheet
    }


}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::excel::writer::ExcelWriter;

    #[test]
    fn test_generate() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        let mut  writer = ExcelWriter::new(PathBuf::from("./src/excel/test/uart"));
        writer.traverse_v();
        writer.generate();
    }
}