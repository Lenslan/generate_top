use std::cell::RefCell;
use crate::excel::reader::ExcelReader;
use std::path::PathBuf;
use std::sync::Arc;
use colored::Colorize;
use regex::Regex;
use rust_xlsxwriter::{ColNum, Color, Format, FormatAlign, FormatBorder, FormatUnderline, Workbook, Worksheet};
use walkdir::WalkDir;
use crate::verilog::data::{VerilogData, WrapMacro};
use crate::verilog::module::VerilogModule;
use crate::verilog::parse::VerilogParser;
use crate::verilog::port::{UndefineWireCollector, VerilogPort};
use crate::verilog::wire::WireBuilder;

#[derive(Default)]
pub struct ExcelWriter {
    module_dir_path: PathBuf,
    file_list: Vec<PathBuf>,
}

impl ExcelWriter {
    pub fn new(module_dir_path: PathBuf) -> Self {
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

        let excel_name = parent_path.join(format!("{}.xlsx", module_name));
        log::debug!("start generate excel file {}", excel_name.display());

        let module = self.get_module_from_v(module_name);
        WireBuilder::check_health();

        // write excel
        self.write_excel(excel_name, module);
    }

    pub fn generate_or_update(&self) {
        let parent_path = self.module_dir_path.parent().expect("Could not get parent path");
        let module_name = self.module_dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("Could not get module name");
        let excel_name = parent_path.join(format!("{}.xlsx", module_name));

        if excel_name.exists() {
            log::debug!("excel {} already exists, next is to update it", excel_name.display());
            self.update();
            log::info!("excel {} update over!", excel_name.display());
        } else {
            log::debug!("excel {} does not exist, next is to generate it", excel_name.display());
            self.generate();
            log::info!("excel {} generate over!", excel_name.display());
        }
    }

    fn update(&self) {
        let parent_path = self.module_dir_path.parent().expect("Could not get parent path");
        let module_name = self.module_dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("Could not get module name");
        let excel_name = parent_path.join(format!("{}.xlsx", module_name));
        log::info!(">> start to parse verilog source file");
        let module_v = self.get_module_from_v(module_name);
        log::info!(">> start to parse excel file");
        let module_xlsx = self.get_module_from_excel(&excel_name);

        UndefineWireCollector::clear();
        WireBuilder::clear();
        let mut module = VerilogModule::new(module_name.into());
        module.add_param_list(VerilogModule::copy_parameter_from(&module_xlsx));
        // add inst
        for inst_excel in module_xlsx.inst_list.iter() {
            let inst_excel = inst_excel.borrow();
            if let Some(inst_v) = module_v.find_inst_module_by_name(&inst_excel.module_name) {
                log::debug!("add inst {} in excel", inst_excel.module_name);
                let params = VerilogModule::copy_parameter_from(&inst_excel);
                // update width
                {
                    let mut inst_v_mut = inst_v.borrow_mut();
                    inst_v_mut.update_literal_port(&params);
                }

                // update inst name
                let inst_v = inst_v.borrow();
                let mut inst_module = VerilogModule::new(inst_excel.module_name.clone());
                inst_module.fix_inst_name(inst_excel.inst_name.as_deref().unwrap());


                inst_module.add_param_list(params);

                // traverse all the port of `inst_v`
                for p in inst_excel.same_ports_with(&inst_v) {
                    let mut new_port = VerilogPort::copy_inst_port_from(p);
                    new_port.check_health();
                    inst_module.add_port_inst(new_port);
                }
                for p in inst_v.diff_ports_with(&inst_excel) {
                    let mut new_port = VerilogPort::copy_inst_port_without_wire_from(p);
                    new_port.check_health();
                    inst_module.add_port_inst(new_port);
                }

                module.add_inst_module(Arc::new(RefCell::new(inst_module.wrap_macro_as(&inst_excel))));
            } else {
                log::info!("Inst {} in excel was not found in rtl, delete it", inst_excel.module_name);
                continue;
            }
        }

        for inst in module_v.diff_inst_with(&module_xlsx) {
            let inst = inst.borrow();
            log::debug!("add inst {} in rtl", inst.module_name);
            let new_module = VerilogModule::copy_module_from(&inst);
            module.add_inst_module(Arc::new(RefCell::new(new_module)));
        }

        let mut temp_module = VerilogModule::new("temp".into());
        for (inout, width, name) in WireBuilder::traverse_unload_undriven() {
            temp_module.add_port(inout, &name, width)
        }

        // add port
        for p in module_xlsx.same_ports_with(&temp_module) {
            log::debug!("add port in rtl & xlsx: {}", p.name);
            let mut new_port = VerilogPort::copy_main_port_from(p);
            new_port.register_port_as_wire();
            module.add_port_inst(new_port.wrap_macro_as(p));
        }
        log::info!("{}", "======>  Change Messages  <======".bright_purple().bold());
        // 存在bug，先不采用这种方式
        // for p in temp_module.diff_ports_with(&module_xlsx) {
        //     log::debug!("add port in rtl but not in xlsx: {}", p.name);
        //     log::info!("add port {} by verilog source file", p.name);
        //     let mut new_port = VerilogPort::copy_main_port_from(p);
        //     new_port.register_port_as_wire();
        //     module.add_port_inst(new_port);
        // }
        for p in module_xlsx.diff_ports_with(&temp_module) {
            if WireBuilder::find_wire_in(p) {
                log::debug!("add wire in xlsx but not in rtl: {}", p.name);
                log::info!("add port {} by excel file", p.name);
                let mut new_port = VerilogPort::copy_main_port_from(p);
                new_port.register_port_as_wire();
                module.add_port_inst(new_port.wrap_macro_as(p));
            } else {
                log::debug!("Port {} in xlsx but not in rtl was dropped", p.name);
                log::info!("{} {}","drop port".bright_black(), p.name.bright_black());
            }
        }

        for (inout, width, name) in WireBuilder::traverse_unload_undriven() {
            log::debug!("add port in rtl but not in xlsx: {}", name);
            log::info!("add port {} by verilog source file", name);
            let mut new_port = VerilogPort::new(inout, &name, width.into());
            new_port.register_port_as_wire();
            module.add_port_inst(new_port.wrap_raw());
        }

        log::info!("{}", "<======  Change Messages  ======>".bright_purple().bold());

        WireBuilder::check_health();

        self.write_excel(excel_name, module.wrap_raw());

    }

    fn write_excel(&self, excel_name: PathBuf, module: VerilogData<VerilogModule>) {
        let mut workbook = Workbook::new();

        workbook.push_worksheet(self.add_inst_sheet(&module));
        for item in module.inst_list.iter() {
            workbook.push_worksheet(self.add_inst_sheet(&*item.borrow()));
        }
        workbook.save(excel_name).unwrap();
    }

    ///
    /// get VerilogModule from verilog source file
    ///
    fn get_module_from_v(&self, module_name: &str) -> VerilogData<VerilogModule> {
        UndefineWireCollector::clear();
        WireBuilder::clear();
        let mut module = VerilogModule::new(module_name.into());

        for f in self.file_list.iter() {
            let inst_module = VerilogParser::new(f)
                .add_includes(vec![self.module_dir_path.parent().unwrap().to_path_buf()])
                .parse()
                .solve()
                .get_module_info();
            for mut inst_item in inst_module {
                inst_item.set_default_inst_name();
                inst_item.set_default_port_wires();
                module.add_inst_module(Arc::new(RefCell::new(inst_item.wrap_raw())));
            }
        }

        // 遍历wire builder 将所有没有驱动/没有load的信号连接到端口
        for (inout, width, name) in WireBuilder::traverse_unload_undriven() {
            let mut new_port = VerilogPort::new(inout, &name, width.into());
            new_port.register_port_as_wire();
            module.add_port_inst(new_port.wrap_raw());
        }

        module.wrap_raw()
    }

    fn get_module_from_excel(&self, path: &PathBuf) -> VerilogModule {
        ExcelReader::new(path.clone()).get_excel_info()
    }

    pub fn traverse_v(&mut self) {
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

    fn add_inst_sheet(&self, module: &VerilogData<VerilogModule>) -> Worksheet {
        let macro_string = module.get_macro_name();
        let mut sheet = Worksheet::new();
        let header_format = Format::new()
            .set_bold()
            .set_font_size(16)
            .set_align(FormatAlign::Center)
            .set_border_bottom(FormatBorder::Medium)
            .set_border_top(FormatBorder::Medium)
            .set_background_color(Color::Gray);
        let bold_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center);
        let number_format = Format::new()
            .set_align(FormatAlign::Left);
        let same_wire_port_format = Format::new()
            .set_underline(FormatUnderline::Single);
        let title_list = ["Port-name", "InOut", "Width", "Wire-name", "Port-comment", macro_string.as_str()];
        let width_list = [30, 10, 10, 30, 40, 20];

        let mut current_line = 0;

        sheet.set_name(&module.module_name).unwrap();
        sheet.set_row_height(0, 18).unwrap();
        sheet.set_row_height(1, 20).unwrap();

        // write Module Inst name
        sheet.write_with_format(current_line, 0, "Module Inst Name", &bold_format).unwrap();
        sheet.write(current_line, 1, format!("{}", module.inst_name.as_deref().unwrap_or_default())).unwrap();
        current_line += 1;

        // write parameter list
        sheet.write_with_format(current_line, 0, "Parameter:", &bold_format).unwrap();
        current_line += 1;
        for (_idx, para) in module.param_list.iter().enumerate() {
            sheet.write(current_line, 1, para.name.as_str()).unwrap();
            sheet.write(current_line, 2, para.get_name()).unwrap();
            current_line += 1;
        }

        // write Title
        for item in title_list.into_iter().enumerate() {
            sheet.write_with_format(current_line, item.0 as ColNum, item.1, &header_format).unwrap();
            sheet.set_column_width(item.0 as ColNum, width_list[item.0]).unwrap();
        }
        sheet.set_freeze_panes(current_line+1, 0).unwrap();
        current_line += 1;

        // write port
        for (_idx, port) in module.port_list.iter().enumerate() {
            sheet.write(current_line, 0, &port.name).unwrap();
            sheet.write(current_line, 1, format!("{}", port.inout)).unwrap();
            // sheet.write_with_format(current_line, 2, port.width.width() as u32, &number_format).unwrap();
            sheet.write(current_line, 2, port.width.width() as u32).unwrap();
            let signal_string = port.get_signal_string()
                .replace('{', "")
                .replace('}', "");
            if Self::drop_bracket(&signal_string) == port.name && port.signals.len() == 2 {
                sheet.write_with_format(current_line, 3, signal_string, &same_wire_port_format).unwrap();
            } else {
                sheet.write(current_line, 3, signal_string).unwrap();
            }
            sheet.write(current_line, 4, &port.info).unwrap();
            sheet.set_row_height(current_line, 16).unwrap();
            
            // write macro tag
            sheet.write(current_line, 5, port.get_macro_name()).unwrap();
            
            current_line += 1;
        }
        
        // write assign logic
        sheet.write_with_format(current_line, 0, "Assing-Logic", &header_format).unwrap();
        

        sheet
    }

    fn drop_bracket(s: &str) -> &str {
        let name_re = Regex::new(r"\b[a-zA-Z_]\w*\b").unwrap();
        if let Some(res) = name_re.find(s) {
            res.as_str()
        } else {
            ""
        }
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
        println!("start running");
        writer.traverse_v();
        writer.generate_or_update();
    }
}