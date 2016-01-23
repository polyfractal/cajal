
#[macro_use] extern crate enum_primitive;
extern crate num;

extern crate roaring;
extern crate rayon;
extern crate rand;

use grid::Grid;
use rayon::par_iter::*;

mod grid;

trait ReportMemory {
    fn memory(&self) -> u32;
}


struct Cajal {
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
    fn new(size: usize, density: f32) -> Cajal {
        Cajal {
            grid: Grid::new(size, density)
        }
    }

    fn grow(&self) {

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
