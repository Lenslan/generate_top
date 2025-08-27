use std::{ops::RangeInclusive, sync::Arc, vec};
use std::collections::HashSet;
use std::ops::Range;
use crate::verilog::port::VerilogValue::{Number, Wire};
use crate::verilog::wire::{VerilogWire, WireBuilder};

#[derive(Debug)]
pub struct VerilogPort {
    inout: PortDir,
    name: String,
    width: usize,

    info: String,

    signals: Vec<VerilogValue>,
    has_undefine: bool,

}
impl VerilogPort {
    pub fn new(inout: PortDir, name: &str, width: usize) -> Self {
        Self {
            inout,
            name: String::from(name),
            width,
            info: String::new(),
            signals: vec![VerilogValue::NONE],
            has_undefine: false,

        }
    }

    fn connect_full_signal(&mut self, sig: &str) {
        self.signals.push(VerilogValue::UndefinedWire(sig.into()));
        self.has_undefine = true;
    }

    fn connect_partial_signal(&mut self, sig: &str, range: &Range<usize>){
        let wire = match self.inout {
            PortDir::InPort => WireBuilder::add_load_wire(sig, range),
            PortDir::OutPort => WireBuilder::add_driver_wire(sig, range),
            _ => WireBuilder::add_load_wire(sig, range)             //TODO how to process inout port
        };
        self.signals.push(Wire(Arc::clone(&wire), range.clone()));
    }

    fn connect_number_signal(&mut self, num_val: u128, num_bits: u8) {
        self.signals.push(Number {
            width: num_bits,
            value: num_val
        });
    }


    fn check_health(&self) -> anyhow::Result<()> {
        todo!()
        // for (bit, flag) in self.connected.iter().enumerate() {
        //     if flag.is_none() {
        //         log::warn!("Port {}[{}] has not connected", self.name, bit)
        //     }
        // }
    }

    pub fn get_info(&self) -> &str {
        &self.info
    }


    pub fn to_inst_string(&self) -> String {
        todo!()
    }

    pub fn to_port_string(&self) -> String {
        todo!()
    }
}
#[derive(Debug)]
pub enum PortDir {
    InPort,
    OutPort,
    InOutPort,
}

impl PortDir {
    fn is_in(&self) -> bool {
        match self {
            Self::OutPort => false,
            _ => true
        }
    }

    fn is_out(&self) -> bool {
        match self {
            Self::InPort => false,
            _ => true
        }
    }
}

#[derive(Debug, Clone)]
enum VerilogValue {
    Wire(Arc<VerilogWire>, Range<usize>),
    UndefinedWire(String),
    Number{
        width: u8,
        value: u128
    },          // Max value is 2^128 -1
    NONE
}

impl VerilogValue {
    fn is_none(&self) -> bool {
        match self {
            Self::NONE => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_port() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        let mut port1 = VerilogPort::new(PortDir::InPort, "port1", 6);
        port1.connect_full_signal("wire1");
        let mut port2 = VerilogPort::new(PortDir::OutPort, "port2", 6);
        port2.connect_partial_signal("wire1", &(0..3));


        WireBuilder::builder_show();
        WireBuilder::check_health();

        println!("{:#?}", port1);
        println!("{:#?}", port2);

        


    }
}