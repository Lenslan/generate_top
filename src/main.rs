use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

mod verilog;

use verilog::wire::VerilogWire;

fn main() {
    let mut set1 = HashSet::new();
    let mut set2 = HashSet::new();
    set1.insert(1);
    set1.insert(2);
    set1.insert(3);

    set2.insert(2);
    set2.insert(3);
    set2.insert(4);

    let res = set2.difference(&set1).collect::<Vec<_>>();
    println!("{:?}", res);
}