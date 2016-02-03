#![feature(alloc_system)]
extern crate alloc_system;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate cajal;
extern crate time;
extern crate rayon;

use cajal::{Cajal, CellType};
use time::{SteadyTime, Duration};

pub const PAGE_SIZE: u32 = 65536;
pub const PAGE_WIDTH: u32 = 256;

#[derive(Debug)]
enum Mode {
    Grow,
    Signal
}

fn main() {
    log4rs::init_file("examples/viz/log.toml", Default::default()).unwrap();

    rayon::initialize(rayon::Configuration::new().set_num_threads(4));

    let num_pages = 63u32;
    let dimension = num_pages * PAGE_WIDTH;
    let mut cajal = Cajal::new(num_pages, 0.01, &[1,2,3,7]);

    let start = SteadyTime::now();
    cajal.grow();
    let elapsed =  SteadyTime::now() - start;
    let signal = (*cajal.get_cell(10, 10)).get_signal();
    println!("Elapsed time: {:?} (signal: {})", elapsed, signal);
}
