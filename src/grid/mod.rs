


use std::slice::IterMut;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rayon::par_iter::*;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use self::page::{Page, GrowthPhase};
use super::ReportMemory;

mod cell;
mod page;
mod zorder;



pub struct Grid {
    pages: Vec<Page>
}

impl ReportMemory for Grid {
    fn memory(&self) -> u32 {
        self.pages.into_par_iter()
            .map(|page| page.memory())
            .sum()
    }
}

impl Grid {
    pub fn new(size: usize, density: f32) -> Grid {
        //todo assert size
        let num_pages = (2 << size) >> 12;

        let mut pages = Vec::with_capacity(num_pages);
        for _ in 0..num_pages {
             pages.push(Page::new(density));
        }

        Grid {
            pages: pages
        }
    }

    fn grow(&mut self) {
        loop {
            let active_cells =  self.pages.par_iter_mut()
                .map(|page| page.grow(GrowthPhase::Axon))
                .sum();

            self.pages.par_iter_mut()
                .map(|page| page.update());

            if active_cells == 0 {
                break;
            }
        }
    }
}


impl Default for Grid {
    fn default() -> Grid {
        Grid::new(15, 0.05)
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
