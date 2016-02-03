
use rayon::par_iter::*;
use self::page::{Page};
use super::ReportMemory;

pub use self::page::{Cell, CellType};

mod cell;
mod page;
mod zorder;



pub struct Grid {
    pages: Vec<Page>,
    dimension: u32,
    pages_per_side: u32
}

impl ReportMemory for Grid {
    fn memory(&self) -> u32 {
        self.pages.into_par_iter()
            .map(|page| page.memory())
            .sum()
    }
}

impl Grid {
    pub fn new(size: u32, density: f32, seed: &[usize]) -> Grid {
        //todo assert size
        let num_pages = size * size;

        info!("Creating grid with {} pages per side ({} pages total), each with {} cells ({} total cells)",
            size, num_pages, 4096, num_pages * 4096);

        let mut pages = Vec::with_capacity(num_pages as usize);
        for i in 0..num_pages {
            let offset_x = (i as u32 % size) * 64;
            let offset_y = (i as u32 / size) * 64;
            debug!("Offsets: ({},{})", offset_x, offset_y);
            pages.push(Page::new(density, offset_x, offset_y, seed));
        }

        Grid {
            pages: pages,
            dimension: size * 64,
            pages_per_side: size
        }
    }

    pub fn grow(&mut self) {
        loop {
            let active_cells = self.grow_step();

            if active_cells == 0 {
                break;
            }
        }
    }

    pub fn grow_step(&mut self) -> u32 {
        debug!("Growing Pages...");

        // TODO replace tuples with structs
        let (active_cells, remote_changes) = self.pages.par_iter_mut()
            .map(|page| page.grow())
            .reduce_with(|a, b| {
                let active_cells = a.0 + b.0;
                let remote_changes = match (a.1, b.1) {
                    (Some(a_r), Some(b_r)) => {
                        let mut t = Vec::with_capacity(a_r.len() + b_r.len());
                        t.extend(a_r);
                        t.extend(b_r);
                        Some(t)
                    },
                    (Some(a_r), None) => Some(a_r),
                    (None, Some(b_r)) => Some(b_r),
                    (None, None) => None
                };
                (active_cells, remote_changes)
            }).unwrap();


        if let Some(changes) = remote_changes {
            debug!("Remote changes to process: {}", changes.len());

            for c in changes {
                debug!("Absolute change position: ({},{})", c.x, c.y);
                if !(c.x > 0 && c.x < self.dimension && c.y > 0 && c.y < self.dimension ) {
                    debug!("x > 1 {}", c.x > 0);
                    debug!("x < dimension - 1{}", c.x < self.dimension);
                    debug!("y > 1 {}", c.y > 0);
                    debug!("y < dimension - 1 {}", c.y < self.dimension );

                    continue;
                }
                self.get_mut_page(c.x, c.y)
                    .add_change(c.x % 64, c.y % 64, c.cell, c.travel_direction, c.stim);
            }
        }

        debug!("Active cells after growth: {}", active_cells);

        debug!("Updating Pages...");
        self.pages.par_iter_mut()
            .for_each(|page| page.update());

        active_cells
    }

    pub fn signal(&mut self) {
        loop {
            let active_cells = self.signal_step();

            if active_cells == 0 {
                break;
            }
        }
    }

    pub fn signal_step(&mut self) -> u32 {
        debug!("Processing signals...");

        let remote_signal = self.pages.par_iter_mut()
            .map(|page| page.signal())
            .reduce_with(|a, b| {
                let remote_signal = match (a, b) {
                    (Some(a_r), Some(b_r)) => {
                        let mut t = Vec::with_capacity(a_r.len() + b_r.len());
                        t.extend(a_r);
                        t.extend(b_r);
                        Some(t)
                    },
                    (Some(a_r), None) => Some(a_r),
                    (None, Some(b_r)) => Some(b_r),
                    (None, None) => None
                };
                remote_signal
            }).unwrap();

        if let Some(changes) = remote_signal {
            debug!("Remote signal to process: {}", changes.len());

            for c in changes {
                debug!("Absolute signal position: ({},{})", c.x, c.y);
                if !(c.x > 0 && c.x < self.dimension && c.y > 0 && c.y < self.dimension ) {
                    debug!("x > 1 {}", c.x > 0);
                    debug!("x < dimension - 1{}", c.x < self.dimension);
                    debug!("y > 1 {}", c.y > 0);
                    debug!("y < dimension - 1 {}", c.y < self.dimension );

                    continue;
                }
                self.get_mut_page(c.x, c.y)
                    .add_signal(c.x % 64, c.y % 64, c.strength, c.stim);
            }
        }

        debug!("Updating Pages...");
        self.pages.par_iter_mut()
            .map(|page| page.update_signal())
            .sum()
    }


    fn get_mut_page(&mut self, x: u32, y: u32) -> &mut Page {
        let i = x / 64 + ((y / 64) * self.pages_per_side);
        debug!("get_mut_page: ({},{}) -> {}", x, y, i);
        &mut self.pages[i as usize]
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &Cell {
        let i = x / 64 + ((y / 64) * self.pages_per_side);
        self.pages[i as usize].get_cell(x % 64, y % 64)
    }

    fn get_mut_cell(&mut self, x: u32, y: u32) -> &mut Cell {
        let i = x / 64 + ((y / 64) * self.pages_per_side);
        self.pages[i as usize].get_mut_cell(x % 64, y % 64)
    }

    pub fn set_input(&mut self, x: u32, y: u32, sig: u8) {
        self.get_mut_page(x, y).set_input(x % 64, y % 64, sig);
    }
}


impl Default for Grid {
    fn default() -> Grid {
        Grid::new(10, 0.05, &[1, 2, 3, 4])
    }
}


#[cfg(test)]
mod test {
    use super::{Grid};

    #[test]
    fn grid_default_params() {
        let _ = Grid::default();
    }
}
