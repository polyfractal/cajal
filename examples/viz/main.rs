extern crate piston_window;
extern crate cajal;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate time;
extern crate rayon;


use piston_window::*;
use cajal::{Cajal, CellType};
use time::{SteadyTime, Duration};


pub const SQ_SIZE: u32 = 3;
pub const PAGE_SIZE: u32 = 65536;
pub const PAGE_WIDTH: u32 = 256;


#[derive(Debug)]
enum Mode {
    Grow,
    Signal,
}

fn main() {
    log4rs::init_file("examples/viz/log.toml", Default::default()).unwrap();

    let _ = rayon::initialize(rayon::Configuration::new().set_num_threads(4));

    let num_pages = 2u32;
    let dimension = num_pages * PAGE_WIDTH;
    let mut cajal = Cajal::new(num_pages, 0.001, &[1, 2, 3, 7]);

    let window: PistonWindow = WindowSettings::new("Cajal Visualization",
                                                   [dimension * SQ_SIZE, dimension * SQ_SIZE])
                                   .exit_on_esc(true)
                                   .build()
                                   .unwrap();

    let first = SteadyTime::now();
    let mut mode = Mode::Grow;
    let mut counter = 0;

    for e in window {
        e.draw_2d(|c, g| {
            if SteadyTime::now() - first < Duration::seconds(5) {
                return;
            }
            // if SteadyTime::now() - last > Duration::milliseconds(1) {
            //    last = SteadyTime::now();
            counter += 1;

            mode = match mode {
                Mode::Grow => {
                    let active = cajal.grow_step();
                    info!("GROW >>> {} ({:?})", active, mode);
                    match active {
                        0 => {
                            warn!("Returning Signal");
                            Mode::Signal
                        }
                        _ => Mode::Grow,
                    }
                }
                Mode::Signal => {
                    if counter >= 1 {
                        for i in (0..dimension).filter(|i| i % 2 == 0) {
                            cajal.set_input(i, i, 63);
                            cajal.set_input(i, dimension - i - 1, 63);
                        }


                        counter = 0;
                    }

                    let active = cajal.signal_step();
                    info!("SIGNAL >>> {} ({:?})", active, mode);
                    Mode::Signal
                }
            };
            // }

            clear([1.0; 4], g);

            for x in 0u32..dimension {
                for y in 0u32..dimension {
                    let mut color = match (*cajal.get_cell(x, y)).get_cell_type() {
                        CellType::Axon => color::hex("F25F5C"), // red
                        CellType::Dendrite => color::hex("70C1B3"), // blue
                        CellType::Body => color::hex("50514F"), // brown
                        CellType::Empty => [1.0, 1.0, 1.0, 1.0], // white
                    };

                    if (*cajal.get_cell(x, y)).get_signal() >
                       (*cajal.get_cell(x, y)).get_threshold() {
                        color = color::hex("F9C22E");   // yellow
                    } else if (*cajal.get_cell(x, y)).get_signal() > 0 {
                        color = color::hex("FAEBC3");
                    }



                    rectangle(color,
                              [1.0, 1.0, SQ_SIZE as f64, SQ_SIZE as f64],
                              c.transform.trans((SQ_SIZE * x) as f64, (SQ_SIZE * y) as f64),
                              g);
                }
            }

        });

    }
}
