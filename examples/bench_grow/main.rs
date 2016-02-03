#![feature(alloc_system)]
extern crate alloc_system;

extern crate cajal;
extern crate time;
extern crate rayon;

use cajal::{Cajal, CellType};
use time::{SteadyTime, Duration};


#[derive(Debug)]
enum Mode {
    Grow,
    Signal
}

fn main() {

    rayon::initialize(rayon::Configuration::new().set_num_threads(4));

    let num_pages = 250u32;
    let dimension = num_pages * 64u32;
    let mut cajal = Cajal::new(num_pages, 0.01, &[1,2,3,7]);

    let start = SteadyTime::now();
    cajal.grow();
    let elapsed =  SteadyTime::now() - start;
    let signal = (*cajal.get_cell(10, 10)).get_signal();
    println!("Elapsed time: {:?} (signal: {})", elapsed, signal);
}
