use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

mod verilog;

use verilog::wire::VerilogWire;

fn main() {
    let mut map: HashMap<Arc<VerilogWire>, i32> = HashMap::new();

}