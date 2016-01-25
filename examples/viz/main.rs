extern crate piston_window;
extern crate cajal;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate time;

use piston_window::*;
use cajal::{Cajal, CellType, Cell};
use time::{SteadyTime, Duration};
use std::thread;


pub const SQ_SIZE: u32 = 7;

fn main() {
    log4rs::init_file("examples/viz/log.toml", Default::default()).unwrap();

    let num_pages = 1u32;
    let dimension = num_pages * 64u32;
    let mut cajal = Cajal::new(num_pages, 0.005);

    let window: PistonWindow =
        WindowSettings::new("Cajal Visualization", [dimension * SQ_SIZE, dimension * SQ_SIZE])
        .exit_on_esc(true).build().unwrap();

    let first = SteadyTime::now();
    let mut last = SteadyTime::now();


    for e in window {
        e.draw_2d(|c, g| {
            if SteadyTime::now() - first < Duration::seconds(10) {
                return;
            }
            if SteadyTime::now() - last > Duration::milliseconds(500) {
                last = SteadyTime::now();
                cajal.grow_step();
            }

            clear([1.0; 4], g);

            for x in 0u32..dimension {
                for y in 0u32..dimension {
                    let color = match (*cajal.get_cell(x, y)).get_cell_type() {
                        CellType::Axon => color::hex("F25F5C"), // red
                        CellType::Dendrite => color::hex("70C1B3"), // blue
                        CellType::Body => color::hex("50514F"), // brown
                        CellType::Empty => [1.0, 1.0, 1.0, 1.0], // white
                    };

                    rectangle(color, [1.0, 1.0, SQ_SIZE as f64, SQ_SIZE as f64],
                              c.transform.trans((SQ_SIZE * x) as f64, (SQ_SIZE * y) as f64), g);
                }
            }

        });

    }
}
