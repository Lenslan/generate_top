use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use rust_xlsxwriter::ChartAxisLabelAlignment;
use serde_json::value::Index;
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
    pub fn get_raw(&self) -> &T {
        match self {
            Self::Raw(x) => x,
            Self::Macro { value, .. } => value.get_raw(),
        }
    }

    pub fn get_macro_name(&self) -> String {
        let mut res = Vec::new();
        match self {
            VerilogData::Raw(_) => {}
            VerilogData::Macro { name, value } => {
                res.push(name.clone());
                res.push(value.get_macro_name())
            }
        }
        res.join(", ")
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

    pub fn to_port_string(&self, is_last: bool) -> Vec<String> {
        match self {
            VerilogData::Raw(x) => {
                x.to_port_string(is_last)
            }
            VerilogData::Macro { name, value } => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_port_string(is_last));
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }

    pub fn to_assign_string(&self) -> Option<Vec<String>> {
        match self {
            VerilogData::Raw(x) => {
                x.to_assign_string()
            }
            VerilogData::Macro { name, value } => {
                if let Some(t) = value.to_assign_string() {
                    let mut res = Vec::new();
                    res.push(format!("`ifdef {}", name));
                    res.extend(t);
                    res.push(format!("`endif  // {}", name));
                    Some(res)
                } else { None }
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

impl<T> DerefMut for VerilogData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            VerilogData::Raw(x) => {x}
            VerilogData::Macro {value, ..} => {value.deref_mut()}
        }
    }
}

impl<T:Hash> Hash for VerilogData<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            VerilogData::Raw(x) => x.hash(state),
            VerilogData::Macro {value, ..} => value.hash(state)
        }
    }
}

impl<T: PartialEq> PartialEq for VerilogData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_raw() == other.get_raw()
    }
}
impl<T: PartialEq> Eq for VerilogData<T> {}

pub trait WrapMacro<T> {
    fn wrap_macro_with(self, name: Vec<impl Into<String>>) -> VerilogData<T>;
    fn wrap_raw(self) -> VerilogData<T>;

    fn wrap_macro_as(self, other: &VerilogData<T>) -> VerilogData<T>;
}
impl<T> WrapMacro<T> for T {

    // 通过传入指定的宏来进行wrap
    fn wrap_macro_with(self, name: Vec<impl Into<String>>) -> VerilogData<T> {
        let mut t = VerilogData::Raw(self);
        for s in name {
            t = VerilogData::Macro {
                name: s.into(),
                value: Box::new(t)
            }
        }
        t
    }

    fn wrap_raw(self) -> VerilogData<T> {
        VerilogData::Raw(self)
    }

    // 通过参考其他的宏来对self进行wrap
    fn wrap_macro_as(self, other: &VerilogData<T>) -> VerilogData<T> {
        match other {
            VerilogData::Raw(_) => VerilogData::Raw(self),
            VerilogData::Macro {name, value} => {
                VerilogData::Macro {
                    name: name.into(),
                    value: Box::from(self.wrap_macro_as(&value))
                }
            }
        }
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