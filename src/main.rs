use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

mod verilog;
mod utils;

use verilog::wire::VerilogWire;

fn main() {
    let mut table1 = vec![
        vec![1,2,3],
        vec![4,6],
        vec![8],
    ];


    println!("{:?}", table1);
    
}