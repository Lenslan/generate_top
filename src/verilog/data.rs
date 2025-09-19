use std::ops::Deref;
use rust_xlsxwriter::ChartAxisLabelAlignment;
use strum::Display;
use crate::verilog::module::VerilogModule;
use crate::verilog::port::VerilogPort;

///
/// type T may be VerilogModule, VerilogPort, VerilogWire
#[derive(Debug, Display)]
pub enum VerilogData<T> {
    Raw(T),
    Macro {
        name: String,
        value: Box<VerilogData<T>>,
    }
}

impl<T> VerilogData<T> {
    fn get_raw(&self) -> &T {
        match self {
            Self::Raw(x) => x,
            Self::Macro { value, .. } => value.get_raw(),
        }
    }
}

impl VerilogData<VerilogModule> {
    pub fn to_inst_string(&self) -> Vec<String> {
        match self {
            VerilogData::Raw(x) => {x.to_inst_string()}
            VerilogData::Macro { name, value} => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_inst_string());
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }
}

impl VerilogData<VerilogPort> {
    
    pub fn to_inst_string(&self, is_last: bool) -> Vec<String> {
        match self {
            VerilogData::Raw(x) => {
                x.to_inst_string(is_last)
            }
            VerilogData::Macro {name, value} => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_inst_string(is_last));
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }
}

impl VerilogData<Vec<VerilogPort>> {
    pub fn to_inst_string(&self, is_last: bool) -> Vec<String> {
        match self {
            VerilogData::Raw(x) => {
                let mut res = Vec::new();
                if let Some((last_para, paras)) = x.split_last() {
                    for item in paras {
                        res.extend(item.to_inst_string(false));
                    }
                    res.extend(last_para.to_inst_string(is_last));
                }
                res
            }
            VerilogData::Macro {name, value} => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_inst_string(is_last));
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }
}

impl<T> Deref for VerilogData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            VerilogData::Raw(x) => {x}
            VerilogData::Macro { value, .. } => {value.deref()}
        }
    }
}

pub trait WrapMacro<T> {
    fn wrap_macro(self, name: impl Into<String>) -> VerilogData<T>;
    fn wrap_raw(self) -> VerilogData<T>;
}
impl<T> WrapMacro<T> for T {
    fn wrap_macro(self, name: impl Into<String>) -> VerilogData<T> {
        VerilogData::Macro {
            name: name.into(),
            value: Box::new(VerilogData::Raw(self))
        }
    }

    fn wrap_raw(self) -> VerilogData<T> {
        VerilogData::Raw(self)
    }
}


#[cfg(test)]
mod test {
    use crate::verilog::data::VerilogData::{Macro, Raw};
    use crate::verilog::module::VerilogModule;

    #[test]
    fn test_data_type() {
        let module = VerilogModule::new(String::from("test_module"));
        let data = Raw(module);
        let macro_data = Macro {
            name: "test_macro".into(),
            value: Box::new(data),
        };
        println!("{:#?}", &macro_data);
    }
}