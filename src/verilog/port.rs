use std::{ops::RangeInclusive, sync::Arc, vec};
use crate::verilog::wire::{VerilogWire, WireBuilder};

#[derive(Debug)]
pub struct VerilogPort {
    inout: PortDir,
    name: String,
    width: usize,
    signal: Vec<Arc<VerilogWire>>,
    connected: Vec<bool>
}
impl VerilogPort {
    fn new(inout: PortDir, name: &str, width: usize) -> Self {
        Self {
            inout,
            name: String::from(name),
            width,
            signal: Vec::new(),
            connected: vec![false;width]
        }
    }

    fn full_connect_signal(&mut self, sig: &str) {
        self.partial_connect_signal(sig, &(0..=self.width-1));
    }

    fn partial_connect_signal<T>(&mut self, sig: &str, range: &T) 
    where 
        T: IntoIterator<Item = usize> + Clone,
    {
        let wire = match self.inout {
            PortDir::InPort => WireBuilder::add_load_wire(sig, range),
            PortDir::OutPort => WireBuilder::add_driver_wire(sig, range),
            _ => WireBuilder::add_load_wire(sig, range) //TODO how to process inout port
        };
        self.signal.push(Arc::clone(&wire));
        for i in range.clone().into_iter() {
            self.connected[i] = true;
        }
    }

    fn check_health(&self) {
        for (bit, flag) in self.connected.iter().enumerate() {
            if !flag {
                log::warn!("Port {}[{}] has not connected", self.name, bit)
            }
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_port() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        let mut port1 = VerilogPort::new(PortDir::InPort, "port1", 6);
        port1.full_connect_signal("wire1");
        let mut port2 = VerilogPort::new(PortDir::OutPort, "port2", 6);
        port2.partial_connect_signal("wire1", &(0..=3));


        WireBuilder::builder_show();
        WireBuilder::check_health();

        println!("{:#?}", port1);
        println!("{:#?}", port2);

        


    }
}