
use std::slice::IterMut;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rayon::par_iter::*;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
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
    pub fn new(size: u32, density: f32) -> Grid {
        //todo assert size
        let num_pages = size * size;

        info!("Creating grid with {} pages per side ({} pages total), each with {} cells ({} total cells)",
            size, num_pages, 4096, num_pages * 4096);

        let mut pages = Vec::with_capacity(num_pages as usize);
        for _ in 0..num_pages {
             pages.push(Page::new(density));
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
        let mut active_cells = self.pages.par_iter_mut()
            .map(|page| page.grow())
            .sum();

        debug!("Active cells after growth: {}", active_cells);

        debug!("Updating Pages...");
        self.pages.par_iter_mut()
            .for_each(|page| page.update());

        active_cells
    }

    pub fn get_cell<'a>(&'a self, x: u32, y: u32) -> &'a Cell {
        let i = x / 64 + ((y / 64) * self.pages_per_side);
        self.pages[i as usize].get_cell(x % 64, y % 64)
    }
}


impl Default for Grid {
    fn default() -> Grid {
        Grid::new(10, 0.05)
    }
}


#[cfg(test)]
mod test {
    use super::{Grid};
    use super::super::ReportMemory;

    #[test]
    fn grid_default_params() {
        let g = Grid::default();
    }
}
