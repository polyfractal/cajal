
use rayon::par_iter::*;
use self::page::{Page};
use super::{ReportMemory, PAGE_SIZE, PAGE_WIDTH};

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
            size, num_pages, PAGE_SIZE, num_pages * PAGE_SIZE);

        let mut pages = Vec::with_capacity(num_pages as usize);
        for i in 0..num_pages {
            let offset_x = (i as u32 % size) * PAGE_WIDTH;
            let offset_y = (i as u32 / size) * PAGE_WIDTH;
            debug!("Offsets: ({},{})", offset_x, offset_y);
            pages.push(Page::new(density, offset_x, offset_y, seed));
        }

        Grid {
            pages: pages,
            dimension: size * PAGE_WIDTH,
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

        self.pages.par_iter_mut()
            .for_each(|page| page.grow());;

        let active_cells = self.pages.iter()
                            .map(|page| page.get_active_cell_count())
                            .fold(0u32, |acc, x| acc + x);

        for i in (0..self.pages.len()) {
            let changes = self.pages[i].get_remote_changes().clone();

            if changes.is_empty() {
                continue;
            }

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
                    .add_change(c.x % PAGE_WIDTH, c.y % PAGE_WIDTH, c.cell, c.travel_direction, c.stim);
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
            .for_each(|page| page.signal());

        for i in 0..self.pages.len() {

            let signals = self.pages[i].get_remote_signal().clone();

            if signals.is_empty() {
                continue;
            }

            debug!("Remote signal to process: {}", signals.len());

            for s in signals {
                debug!("Absolute signal position: ({},{})", s.x, s.y);
                if !(s.x > 0 && s.x < self.dimension && s.y > 0 && s.y < self.dimension ) {
                    debug!("x > 1 {}", s.x > 0);
                    debug!("x < dimension - 1{}", s.x < self.dimension);
                    debug!("y > 1 {}", s.y > 0);
                    debug!("y < dimension - 1 {}", s.y < self.dimension );

                    continue;
                }
                self.get_mut_page(s.x, s.y)
                    .add_signal(s.x % PAGE_WIDTH, s.y % PAGE_WIDTH, s.strength, s.stim);
            }
        }

        debug!("Updating Pages...");
        self.pages.par_iter_mut()
            .map(|page| page.update_signal())
            .sum()
    }


    fn get_mut_page(&mut self, x: u32, y: u32) -> &mut Page {
        let i = x / PAGE_WIDTH + ((y / PAGE_WIDTH) * self.pages_per_side);
        debug!("get_mut_page: ({},{}) -> {}", x, y, i);
        &mut self.pages[i as usize]
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &Cell {
        let i = x / PAGE_WIDTH + ((y / PAGE_WIDTH) * self.pages_per_side);
        self.pages[i as usize].get_cell(x % PAGE_WIDTH, y % PAGE_WIDTH)
    }

    fn get_mut_cell(&mut self, x: u32, y: u32) -> &mut Cell {
        let i = x / PAGE_WIDTH + ((y / PAGE_WIDTH) * self.pages_per_side);
        self.pages[i as usize].get_mut_cell(x % PAGE_WIDTH, y % PAGE_WIDTH)
    }

    pub fn set_input(&mut self, x: u32, y: u32, sig: u8) {
        self.get_mut_page(x, y).set_input(x % PAGE_WIDTH, y % PAGE_WIDTH, sig);
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
