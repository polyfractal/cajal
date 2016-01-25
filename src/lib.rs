
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate log;

extern crate num;
extern crate roaring;
extern crate rayon;
extern crate rand;

pub use grid::{Cell, CellType};
use grid::Grid;
use rayon::par_iter::*;

mod grid;

trait ReportMemory {
    fn memory(&self) -> u32;
}


pub struct Cajal {
    grid: Grid
}

impl Default for Cajal {
    fn default() -> Cajal {
        Cajal {
            grid: Grid::default()
        }
    }
}

impl ReportMemory for Cajal {
    fn memory(&self) -> u32 {
        self.grid.memory()
    }
}

impl Cajal {
    pub fn new(size: u32, density: f32) -> Cajal {
        Cajal {
            grid: Grid::new(size, density)
        }
    }

    pub fn grow(&mut self) {
        self.grid.grow();
    }

    pub fn grow_step(&mut self) {
        self.grid.grow_step();
    }

    pub fn get_cell<'a>(&'a self, x: u32, y: u32) -> &'a Cell {
        self.grid.get_cell(x, y)
    }
}





#[cfg(test)]
mod test {
    use super::Cajal;
    use super::ReportMemory;

    #[test]
    fn default_params() {
        let c = Cajal::default();
    }
}
