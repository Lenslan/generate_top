use strum::Display;
use crate::verilog::module::VerilogModule;
use crate::verilog::port::VerilogPort;

///
/// type T may be VerilogModule, VerilogPort, VerilogWire
#[derive(Debug, Display)]
enum Data<T> {
    Raw(T),
    Macro {
        name: String,
        value: Box<Data<T>>,
    }
}

impl<T> Data<T> {
    fn get_raw(&self) -> &T {
        match self {
            Self::Raw(x) => x,
            Self::Macro { value, .. } => value.get_raw(),
        }
    }
}

impl Data<VerilogModule> {
    pub fn to_inst_string(&self) -> Vec<String> {
        match self {
            Data::Raw(x) => {x.to_inst_string()}
            Data::Macro { name, value} => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_inst_string());
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }
}

impl Data<VerilogPort> {
    
    pub fn to_inst_string(&self, is_last: bool) -> Vec<String> {
        match self {
            Data::Raw(x) => {
                x.to_inst_string(is_last)
            }
            Data::Macro {name, value} => {
                let mut res = Vec::new();
                res.push(format!("`ifdef {}", name));
                res.extend(value.to_inst_string(is_last));
                res.push(format!("`endif  // {}", name));
                res
            }
        }
    }
}

// impl Data<Vec<VerilogPort>> {
//     pub fn to_inst_string
// }


#[cfg(test)]
mod test {
    use crate::verilog::data::Data::{Macro, Raw};
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