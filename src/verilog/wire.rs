use std::borrow::Borrow;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::{RangeInclusive};
use std::sync::{Arc, LazyLock, Mutex};

pub struct WireBuilder {
    wires: HashMap<String, (Arc<VerilogWire>, WirePayload)>
}
static WIRE_BUILDER_INSTANCE: LazyLock<Mutex<WireBuilder>> = LazyLock::new(|| {
    Mutex::new(WireBuilder {
        wires: HashMap::new()
    })
});
impl WireBuilder {

    pub fn add_driver_wire(name:&str, range: RangeInclusive<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload) = wire_builder.wires
            .entry(name.into())
            .or_insert_with(|| {(Arc::new(VerilogWire::new(name.into())), Default::default())});
        for i in range {
            if !payload.driver.insert(i) {
                log::error!("wire {} has multi driver", name)
            }
        }
        Arc::clone(arc_wire)
    }

    pub fn add_load_wire(name: &str, range: RangeInclusive<usize>) -> Arc<VerilogWire>{
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload) = wire_builder.wires
            .entry(name.into())
            .or_insert_with(|| {(Arc::new(VerilogWire::new(name.into())), Default::default())});
        for i in range {
            payload.load.insert(i);
        }
        Arc::clone(arc_wire)
    }

    fn get_width(name: &str) -> usize {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (_wire, WirePayload {driver, load}) = wire_builder.wires
            .get(name)
            .expect(&format!("Wire {} has not been defined", name));
        let res = max(
            *driver.iter().max().unwrap_or(&0),
            *load.iter().max().unwrap_or(&0));
        res + 1
    }

    fn builder_show() {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let res = &wire_builder.wires;
        println!("{:#?}", res)
    }
}
#[derive(Debug)]
pub struct VerilogWire {
    pub(crate) name: String,
}
impl VerilogWire {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl PartialEq for VerilogWire {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for VerilogWire {}

impl Hash for VerilogWire {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Default, Debug)]
struct WirePayload {
    driver: HashSet<usize>,
    load: HashSet<usize>
}

impl Borrow<str> for VerilogWire {
    fn borrow(&self) -> &str {
        &self.name
    }
}



#[cfg(test)]
mod test {
    use crate::verilog::wire::WireBuilder;

    #[test]
    fn test_builder() {
        simple_logger::init_with_level(log::Level::Info).unwrap();
        WireBuilder::add_load_wire("testwire1", 0..=0);
        WireBuilder::add_driver_wire("testwire1", 0..=0);
        WireBuilder::add_driver_wire("testwire2", 0..=6);
        WireBuilder::add_load_wire("testwire3", 0..=2);
        WireBuilder::builder_show();
        println!("wire1 width is {}", WireBuilder::get_width("testwire1"));
        println!("wire2 width is {}", WireBuilder::get_width("testwire2"));
        println!("wire3 width is {}", WireBuilder::get_width("testwire3"));

        WireBuilder::add_driver_wire("testwire2", 0..=0);
    }
}