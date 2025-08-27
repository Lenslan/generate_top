use std::borrow::Borrow;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::{Range};
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

    ///
    /// register wire which connected to output port
    ///
    pub fn add_driver_wire(name:&str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload) = wire_builder.wires
            .entry(name.into())
            .or_insert_with(|| {(Arc::new(VerilogWire::new(name.into())), Default::default())});
        for i in range.clone().into_iter() {
            if !payload.driver.insert(i) {
                log::error!("wire {} has multi driver", name)
            }
        }
        Arc::clone(arc_wire)
    }

    ///
    /// register wire which connected to input port
    ///
    pub fn add_load_wire(name: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload) = wire_builder.wires
            .entry(name.into())
            .or_insert_with(|| {(Arc::new(VerilogWire::new(name.into())), Default::default())});
        for i in range.clone().into_iter() {
            payload.load.insert(i);
        }
        Arc::clone(arc_wire)
    }

    ///
    /// get wire width
    ///
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

    ///
    /// check wire has driver & load
    ///
    fn check_driver_load(driver: &HashSet<usize>, load: &HashSet<usize>, name: &str) {
        let mut no_driver = load.difference(driver).collect::<Vec<_>>();
        if !no_driver.is_empty() {
            no_driver.sort();
            for bit in no_driver{
                log::error!("wire {}[{}] has load but no driver", name, bit);
            }
        }

        let mut no_load = driver.difference(load).collect::<Vec<_>>();
        if !no_load.is_empty() {
            no_load.sort();
            for bit in no_load{
                log::warn!("wire {}[{}] has driver but no load", name, bit);
            }
        }
    }

    ///
    /// check all the wires has driver & load
    /// must call this function after connected all the port
    ///
    pub fn check_health() {
        log::info!("WireBuilder health check start >>>>");
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        for (wire, payload) in wire_builder.wires.values() {
            Self::check_driver_load(&payload.driver, &payload.load, &wire.name);
        }
        log::info!("WireBuilder health check end  <<<<");
    }

    ///
    /// debug: show the builder
    ///
    pub fn builder_show() {
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
        WireBuilder::add_load_wire("testwire1", &(0..1));
        WireBuilder::add_driver_wire("testwire1", &(0..1));
        WireBuilder::add_driver_wire("testwire2", &(0..6));
        WireBuilder::add_load_wire("testwire3", &(0..2));
        WireBuilder::builder_show();
        println!("wire1 width is {}", WireBuilder::get_width("testwire1"));
        println!("wire2 width is {}", WireBuilder::get_width("testwire2"));
        println!("wire3 width is {}", WireBuilder::get_width("testwire3"));
        // println!("wire3 width is {}", WireBuilder::get_width("testwire333"));

        WireBuilder::add_driver_wire("testwire2", &(0..1));
        WireBuilder::check_health();
    }
}