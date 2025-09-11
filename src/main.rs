use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

mod excel;
mod utils;
mod verilog;

use verilog::wire::VerilogWire;

fn main() {
    let mut a = vec![1, 2, 3];
    let b = vec![2, 3, 4];
    a.extend(b);
    println!("{:?}", a);
}
