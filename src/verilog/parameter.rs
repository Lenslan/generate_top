use crate::verilog::module::VerilogModule;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub value: Box<ParamValue>,
}

impl Param {
    pub fn new(name: String, value: usize) -> Param {
        Param { name, value: Box::new(value.into()) }
    }

    pub fn new_with_param(name: String, value: ParamValue) -> Param {
        Param {
            name,
            value: Box::new(value),
        }
    }

    pub fn get_value(&self) -> usize {
        match *self.value {
            ParamValue::Value(x) => {x}
            ParamValue::Param(ref p) => {p.get_value()}
        }
    }

    pub fn get_name(&self) -> String {
        match *self.value {
            ParamValue::Value(x) => {format!("{}", x)}
            ParamValue::Param(ref p) => {p.name.clone()}
        }
    }

}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Value(usize),
    Param(Param),
}

impl From<usize> for ParamValue {
    fn from(value: usize) -> Self {
        ParamValue::Value(value)
    }
}

impl From<Param> for ParamValue {
    fn from(value: Param) -> Self {
        ParamValue::Param(value)
    }
}

impl ParamValue {
    pub fn gen_from_params(params: &Vec<Param>, name: String) -> ParamValue {
        for item in params {
            if item.name == name {
                return item.clone().into();
            }
        }
        ParamValue::Value(0)
    }
}