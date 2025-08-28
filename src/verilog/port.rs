use std::{sync::Arc, vec};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{LazyLock, Mutex};
use strum::Display;
use crate::verilog::port::VerilogValue::{Number, Wire};
use crate::verilog::wire::{VerilogWire, WireBuilder};
use crate::utils::solve_func::SolveFunc;

#[derive(Debug, Default)]
pub struct VerilogPort {
    pub inout: PortDir,
    pub name: String,
    pub width: usize,

    pub info: String,

    signals: Vec<VerilogValue>,
    has_undefine: u8,
    undefine_wires_idx: Vec<(usize,usize)>,

    health_checked: bool,
    undefine_registered: bool,

}
impl VerilogPort {
    pub fn new(inout: PortDir, name: &str, width: usize) -> Self {
        Self {
            inout,
            name: String::from(name),
            width,
            signals: vec![VerilogValue::NONE],
            ..Default::default()
        }
    }

    pub fn set_info_msg(&mut self, msg: &str) {
        self.info = String::from(msg);
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
    fn set_undefine_wire(&mut self, name: &str, width: usize, position: usize) {
        let arc_wire = self.connect_wire(
            name,
            &(0..width)
        );
        self.signals[position] = Wire(Arc::clone(&arc_wire), 0..width);
        self.has_undefine -= 1;
    }
    fn set_undefine_wire_1(&mut self) {
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
        self.set_undefine_wire(
            signal.clone().get_name(),
            wire_infer_width,
            idx);
        self.health_checked = true;
    }


    ///
    /// check the signals are full connected
    ///
    fn check_connected(&mut self) {
        let width_sum = self.get_connected_width();
        match self.width.cmp(&width_sum) {
            Ordering::Greater => log::warn!("Port {} has not been full connected", self.name),
            Ordering::Less => log::warn!("Port {} has been over connected", self.name),
            _ => {}
        }
        self.health_checked = true;
    }

    ///
    /// process the undefine wires which connected to this port more than 1
    /// 通过 `HashMap<String, usize>`来注册，通过wire 名字得到她的索引
    /// 矩阵计算完成之后，通过usize 来查看Vec<u8> 来获取位宽
    ///
    fn register_undefine_wire(&mut self) {
        let mut func_group = Vec::new();
        for (undefine_idx, sig) in self.signals.iter().enumerate() {
            if sig.is_undefine() {
                let res_idx = UndefineWireCollector::add_wires(sig.get_name());
                self.undefine_wires_idx.push((undefine_idx,res_idx));
                func_group.push(res_idx);
            }
        }
        let width = self.get_connected_width();
        let infer_width = self.width - width;
        if infer_width <= 0 {
            log::warn!("Port {} has not been full connected", self.name);
        }
        UndefineWireCollector::add_func(
            func_group,
            infer_width as i64,
        );
        self.undefine_registered = true;
    }

    fn update_undefine_wire(&mut self) {
        let collector = WIRECOLLECTOR.lock().unwrap();
        let mut temp = Vec::new();
        for (undefine_idx, res_idx) in self.undefine_wires_idx.iter() {
            let refer_width = collector.res[*res_idx];
            let name = self.signals[*undefine_idx].get_name();
            temp.push((name.to_string(), refer_width, *undefine_idx));
        }
        for (name, refer_width, undefine_idx) in temp {
            self.set_undefine_wire(&name, refer_width, undefine_idx);
        }
        self.health_checked = true;
    }


    ///
    /// check connect
    /// much call this function after this port has benn all connected
    ///
    fn check_health(&mut self) {
        if self.health_checked {
            return ;
        }
        match self.has_undefine {
            0 => self.check_connected(),
            1 => self.set_undefine_wire_1(),
            _ => if self.undefine_registered {self.update_undefine_wire()}
                else { self.register_undefine_wire() }
        };
    }


    pub fn to_inst_string(&self, name_len: u8, signal_len: u8) -> String {
        todo!()
    }

    pub fn to_port_string(&self) -> String {
        todo!()
    }
}
#[derive(Default)]
struct UndefineWireCollector {
    wires: HashMap<String, usize>,
    func_groups: Vec<Vec<usize>>,
    value: Vec<i64>,
    res: Vec<usize>,
}

static WIRECOLLECTOR: LazyLock<Mutex<UndefineWireCollector>> = LazyLock::new(|| {
    Mutex::new(UndefineWireCollector {
        ..Default::default()
    })
});

impl UndefineWireCollector {

    ///
    /// call this function by VerilogPort
    ///
    fn add_wires(wire: &str) -> usize {
        let mut collector = WIRECOLLECTOR.lock().unwrap();
        let len = collector.wires.len();
        let res = collector.wires
            .entry(wire.to_string())
            .or_insert(len);
        *res
    }

    ///
    /// call this function by VerilogPort
    ///
    fn add_func(func: Vec<usize>, value: i64) {
        let mut collector = WIRECOLLECTOR.lock().unwrap();
        collector.func_groups.push(func);
        collector.value.push(value);
    }

    ///
    /// call this function after all the port are connected
    ///
    fn solve_func() {
        let mut collector = WIRECOLLECTOR.lock().unwrap();
        let num_vars = collector.wires.len();
        let new_func = collector.func_groups
            .iter()
            .enumerate()
            .map(|(idx, f)| {
                let mut temp = vec![0; num_vars];
                for c in f { temp[*c] = 1};
                temp.push(collector.value[idx]);
                temp
            })
            .collect::<Vec<Vec<_>>>()
            .solve();

        if let Some(res) = new_func {
            collector.res = res;
        } else {
            log::error!("Can not infer wire-width from wires: \n{:#?}", collector.wires);
        }
    }

    fn has_wires() -> bool {
        let collector = WIRECOLLECTOR.lock().unwrap();
        collector.wires.len() > 0
    }
}





#[derive(Debug, Default, Display)]
pub enum PortDir {
    #[strum(to_string = "input")]
    InPort,
    #[default]
    #[strum(to_string = "output")]
    OutPort,

    #[strum(to_string = "inout")]
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