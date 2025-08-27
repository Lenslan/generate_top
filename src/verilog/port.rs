use std::{sync::Arc, vec};
use std::cmp::Ordering;
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
    has_undefine: u8,

}
impl VerilogPort {
    pub fn new(inout: PortDir, name: &str, width: usize) -> Self {
        Self {
            inout,
            name: String::from(name),
            width,
            info: String::new(),
            signals: vec![VerilogValue::NONE],
            has_undefine: 0,

        }
    }

    ///
    /// connect wire to this port
    /// register wires by WireBuilder
    ///
    fn connect_wire(&self, sig: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        match self.inout {
            PortDir::InPort => WireBuilder::add_load_wire(sig, range),
            PortDir::OutPort => WireBuilder::add_driver_wire(sig, range),
            _ => WireBuilder::add_load_wire(sig, range)             //TODO how to process inout port
        }
    }

    ///
    /// register undefined wire whose width is not declared
    /// those width will be inferred by `set_undefine_wire`
    /// or `solve_func`
    ///
    fn connect_undefined_signal(&mut self, sig: &str) {
        self.signals.push(VerilogValue::UndefinedWire(sig.into()));
        self.has_undefine += 1;
    }

    ///
    /// register wires whose width is declared
    ///
    fn connect_partial_signal(&mut self, sig: &str, range: &Range<usize>){
        let wire = self.connect_wire(sig, range);
        self.signals.push(Wire(Arc::clone(&wire), range.clone()));
    }

    ///
    /// register const number which connected to this port
    ///
    fn connect_number_signal(&mut self, num_val: u128, num_bits: u8) {
        self.signals.push(Number {
            width: num_bits,
            value: num_val
        });
    }

    ///
    /// get bit-width of the existing signal which connected to this port
    ///
    fn get_connected_width(&self) -> usize {
        let mut width_sum = 0;
        for sig in self.signals.iter() {
            match sig {
                Wire(_, range) => {width_sum += range.len()},
                Number { width, value } => {width_sum += width_sum},
                _ => {}
            }
        }
        width_sum
    }

    ///
    /// infer undefined wire's width
    ///
    fn set_undefine_wire(&mut self) {
        let width_sum = self.get_connected_width();
        let wire_infer_width = self.width - width_sum;
        if wire_infer_width <= 0 {
            log::warn!("Port {} has been over connected", self.name);
            return ;
        }
        let (idx, signal) = self.signals
            .iter()
            .enumerate()
            .find(|&sig| sig.1.is_undefine()).unwrap();
        let arc_wire = self.connect_wire(
            signal.get_name(),
            &(0..wire_infer_width)
        );
        self.signals[idx] = Wire(Arc::clone(&arc_wire), 0..wire_infer_width);
        self.has_undefine -= 1;
    }

    ///
    /// check the signals are full connected
    ///
    fn check_connected(&self) {
        let width_sum = self.get_connected_width();
        match self.width.cmp(&width_sum) {
            Ordering::Greater => log::warn!("Port {} has not been full connected", self.name),
            Ordering::Less => log::warn!("Port {} has been over connected", self.name),
            _ => {}
        }
    }

    ///
    /// process the undefine wires which connected to this port more than 1
    /// 通过 `HashMap<String, usize>`来注册，通过wire 名字得到她的索引
    /// 矩阵计算完成之后，通过usize 来查看Vec<u8> 来获取位宽
    ///
    fn register_undefine_wire(&self) {
        todo!()
    }


    ///
    /// check connect
    /// much call this function after this port has benn all connected
    ///
    fn check_health(&mut self) {
        match self.has_undefine {
            0 => self.check_connected(),
            1 => self.set_undefine_wire(),
            _ => self.register_undefine_wire()
        };
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

    fn is_undefine(&self) -> bool {
        match self {
            Self::UndefinedWire(_) => true,
            _ => false
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Self::UndefinedWire(s) => s,
            Self::Wire(w, _) => &w.name,
            _ => ""
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
        port1.connect_undefined_signal("wire1");
        let mut port2 = VerilogPort::new(PortDir::OutPort, "port2", 6);
        port2.connect_partial_signal("wire1", &(0..3));


        WireBuilder::builder_show();
        WireBuilder::check_health();

        println!("{:#?}", port1);
        println!("{:#?}", port2);

        


    }
}