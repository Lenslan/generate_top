use std::borrow::Borrow;
use std::cmp::max;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::{Arc, LazyLock, Mutex};
use colored::Colorize;
use crate::verilog::port::{PortDir, VerilogPort, VerilogValue};

pub struct WireBuilder {
    wires: BTreeMap<String, (Arc<VerilogWire>, WirePayload, WireError)>,
}
static WIRE_BUILDER_INSTANCE: LazyLock<Mutex<WireBuilder>> = LazyLock::new(|| {
    Mutex::new(WireBuilder {
        wires: BTreeMap::new(),
    })
});
impl WireBuilder {
    ///
    /// register wire which connected to output port
    ///
    pub fn add_driver_wire(name: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload, error) = wire_builder
            .wires
            .entry(name.into())
            .or_insert_with(|| (Arc::new(VerilogWire::new(name.into())), WirePayload::default(), WireError::default()));
        for i in range.clone().into_iter() {
            if !payload.driver.insert(i) {
                // dont report error in anytime, only in health_check()
                // log::error!("wire {} bit[{}] has multi driver", name, i)
                error.multi_driver.insert(i);
            }
        }
        Arc::clone(arc_wire)
    }

    ///
    /// register wire which connected to input port
    ///
    pub fn add_load_wire(name: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload, _error) = wire_builder
            .wires
            .entry(name.into())
            .or_insert_with(|| (Arc::new(VerilogWire::new(name.into())), WirePayload::default(), WireError::default()));
        for i in range.clone().into_iter() {
            payload.load.insert(i);
        }
        Arc::clone(arc_wire)
    }

    ///
    /// register wire which connected to output port
    ///
    pub fn add_driver_wire_asport(name: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload, error) = wire_builder.wires.entry(name.into()).or_insert_with(|| {
            (
                Arc::new(VerilogWire::new_port(name.into())),
                WirePayload::default(),
                WireError::default(),
            )
        });
        for i in range.clone().into_iter() {
            if !payload.driver.insert(i) {
                // dont report error in anytime, only in health_check()
                // log::error!("wire {} bit[{}] has multi driver", name, i)
                error.multi_driver.insert(i);
            }
        }
        Arc::clone(arc_wire)
    }

    ///
    /// register wire which connected to input port
    ///
    pub fn add_load_wire_asport(name: &str, range: &Range<usize>) -> Arc<VerilogWire> {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let (arc_wire, payload, _error) = wire_builder.wires.entry(name.into()).or_insert_with(|| {
            (
                Arc::new(VerilogWire::new_port(name.into())),
                WirePayload::default(),
                WireError::default(),
            )
        });
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
        let (_wire, WirePayload { driver, load}, _error) = wire_builder
            .wires
            .get(name)
            .expect(&format!("Wire {} has not been defined", name));
        let res = max(
            *driver.iter().max().unwrap_or(&0),
            *load.iter().max().unwrap_or(&0),
        );
        res + 1
    }

    ///
    /// check wire has driver & load
    ///
    // fn check_driver_load(driver: &HashSet<usize>, load: &HashSet<usize>, name: &str) {
    //     let mut no_driver = load.difference(driver).collect::<Vec<_>>();
    //     if !no_driver.is_empty() {
    //         no_driver.sort();
    //         for bit in no_driver {
    //             log::error!("wire {}[{}] has load but no driver", name, bit);
    //         }
    //     }
    //
    //     let mut no_load = driver.difference(load).collect::<Vec<_>>();
    //     if !no_load.is_empty() {
    //         no_load.sort();
    //         for bit in no_load {
    //             log::warn!("wire {}[{}] has driver but no load", name, bit);
    //         }
    //     }
    // }
    fn check_undriven(driver: &HashSet<usize>, load: &HashSet<usize>) -> Vec<usize> {
        let mut no_driver = load.difference(driver).collect::<Vec<_>>();
        let mut res = Vec::new();
        if !no_driver.is_empty() {
            no_driver.sort();
            for bit in no_driver {
                res.push(*bit);
            }
        }
        res
    }
    fn check_unload(driver: &HashSet<usize>, load: &HashSet<usize>) -> Vec<usize> {
        let mut no_load = driver.difference(load).collect::<Vec<_>>();
        let mut res = Vec::new();
        if !no_load.is_empty() {
            no_load.sort();
            for bit in no_load {
                res.push(*bit);
            }
        }
        res
    }

    ///
    /// check all the wires has driver & load
    /// must call this function after connected all the port
    ///
    pub fn check_health() {
        log::info!("{}",">>> WireBuilder health check start >>>>".bright_green().bold());
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        for (wire, payload, error) in wire_builder.wires.values() {
            // Self::check_driver_load(&payload.driver, &payload.load, &wire.name);
            let undriven = Self::check_undriven(&payload.driver, &payload.load);
            let unload = Self::check_unload(&payload.driver, &payload.load);
            for bit in undriven {
                log::error!("wire {}[{}] has load but no driver", wire.name.red().bold(), bit);
            }
            for bit in unload {
                log::warn!("wire {}[{}] has driver but no load", wire.name.yellow().bold(), bit);
            }
            for bit in error.multi_driver.iter() {
                log::error!("wire {}[{}] has multi-driver", wire.name.red().bold(), bit)
            }
        }
        log::info!("{}","<<< WireBuilder health check end  <<<<".bright_green().bold());
    }

    ///
    /// find wire in WireBuilder
    ///
    pub fn find_wire_in(port: &VerilogPort) -> bool {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let mut res = vec![false];
        let judge = |name: &str| {
            if let Some((_, payload, _)) = wire_builder.wires.get(name) {
                let width = 1+max(
                    payload.driver.iter().max().unwrap_or(&0),
                    payload.load.iter().max().unwrap_or(&0)
                );

                if width == port.width.width() {
                    match port.inout {
                        PortDir::InPort => {
                            if payload.load.len() > 0 { return true }
                        }
                        PortDir::OutPort => {
                            if payload.driver.len() > 0 { return true }
                        }
                        PortDir::InOutPort => {}
                        PortDir::Unknown => {}
                    }
                }
            }
            false
        };
        for wire in port.signals.iter() {
            res.push(match wire {
                VerilogValue::Wire(w, _range) => judge(&w.name),
                VerilogValue::UndefinedWire(s) => judge(s),
                VerilogValue::Number { .. } => true,
                VerilogValue::NONE => judge(&port.name)
            });
        }
        res.iter().any(|&x| x)
    }

    ///
    /// debug: show the builder
    ///
    pub fn builder_show() {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let res = &wire_builder.wires;
        println!("{:#?}", res)
    }

    ///
    /// clear HashMap of current Module
    ///
    pub fn clear() {
        let mut wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        wire_builder.wires = BTreeMap::new();
    }

    ///
    /// traverse to find wires which need to be declared
    ///
    pub fn traverse_unport_wires() -> Vec<WirePrinter> {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let mut res = Vec::new();
        for item in wire_builder.wires.values() {
            if item.0.need_declaration() {
                let name = item.0.name.clone();
                // let width = WireBuilder::get_width(&name);
                let width = 1+max(
                    item.1.driver.iter().max().unwrap_or(&0),
                    item.1.load.iter().max().unwrap_or(&0)
                );
                res.push(WirePrinter::new(name, width));
            }
        }
        res
    }

    pub fn traverse_unload_undriven() -> Vec<(PortDir, usize, String)> {
        let wire_builder = WIRE_BUILDER_INSTANCE.lock().unwrap();
        let mut res = Vec::new();
        for (wire, payload, _) in wire_builder.wires.values() {
            let undriven = Self::check_undriven(&payload.driver, &payload.load);
            let unload = Self::check_unload(&payload.driver, &payload.load);
            if undriven.len() > 0 {
                res.push((PortDir::InPort, undriven.len(), wire.name.clone()));
                continue;
            }
            if unload.len() > 0 {
                res.push((PortDir::OutPort, unload.len(), wire.name.clone()))
            }
        }

        res

    }
}
#[derive(Debug, Default)]
pub struct VerilogWire {
    pub(crate) name: String,
    port_tag: bool,
}
impl VerilogWire {
    fn new(name: String) -> Self {
        Self {
            name,
            port_tag: false,
        }
    }

    fn new_port(name: String) -> Self {
        Self {
            name,
            port_tag: true,
        }
    }

    pub fn need_declaration(&self) -> bool {
        !self.port_tag
    }

    fn set_port_tag(&mut self, port_tag: bool) {
        self.port_tag = port_tag;
    }
}

impl fmt::Display for VerilogWire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
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
    load: HashSet<usize>,
}

#[derive(Default, Debug)]
struct WireError {
    multi_driver: HashSet<usize>,
}

impl Borrow<str> for VerilogWire {
    fn borrow(&self) -> &str {
        &self.name
    }
}

pub struct WirePrinter {
    name: String,
    width: usize
}

impl WirePrinter {
    
    pub fn new(name: String, width: usize) -> Self {
        Self {
            name, width
        }
    }
    pub fn to_string(&self) -> Vec<String> {
        let width_str = if self.width < 2 {
            " ".repeat(8)
        } else { 
            format!("[{:<4}:0]", self.width-1)
        };
        vec![format!(
            "wire {} {:<20}",
            width_str,
            self.name
        )]
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
