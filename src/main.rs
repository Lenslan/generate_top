use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

mod verilog;

use verilog::wire::VerilogWire;

fn main() {
    let mut map: HashMap<Arc<VerilogWire>, i32> = HashMap::new();

    let a = Arc::new(VerilogWire { name: "hello".to_string() });
    map.insert(a.clone(), 42);

    // 用 &str 查找，不会发生 String 的拷贝
    if let Some(v) = map.get("hello") {
        println!("found: {}", v);
    } else {
        println!("not found");
    }
}