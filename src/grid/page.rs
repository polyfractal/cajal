
use std::slice::IterMut;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rayon::par_iter::*;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use std::thread;


pub use super::cell::{Cell, Chromosome, CellType, Gate};
use super::zorder;
use super::super::ReportMemory;

pub struct Page {
    cells: Vec<Cell>,
    active: RoaringBitmap<u32>,
    changes: HashMap<u32, Cell>
}

impl ReportMemory for Page {
    fn memory(&self) -> u32 {
        (self.cells.len() as u32 * 16) +
        (self.active.len() as u32 * 8) +  // <-- This is not true!
        ((self.changes.len() as u32 + self.changes.capacity() as u32) * 8) // Rough approximation
    }
}

impl Page {
    pub fn new(density: f32) -> Page {
        debug!("Creating new Page with {} density.", density);
        let mut rng = thread_rng();

        let num_cells = 4096;
        let mut cells: Vec<Cell> = Vec::with_capacity(num_cells);

        for _ in 0..num_cells {
            let mut cell = Cell::new();
            cell.set_chromosome(rng.gen());
            cells.push(cell);
        }

        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

        // TODO roll this into the initialization loop
        let active_cells: u32 = (4096f32 * density).round() as u32;
        debug!("Active cells in this Page: {}", active_cells);

        let range_cells = Range::new(1, 62);

        for _ in 0..active_cells {
            let (x, y) = (range_cells.ind_sample(&mut rng), range_cells.ind_sample(&mut rng));
            let index = zorder::xy_to_z(x, y);

            cells[index as usize].set_cell_type(CellType::Body);

            let axon_direction: Gate = rng.gen();
            let dendrite_direction = match axon_direction {
                Gate::North => Gate::South,
                Gate::South => Gate::North,
                Gate::East => Gate::West,
                Gate::West => Gate::East
            };

            if let Some((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Axon, axon_direction) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                bitmap.insert(target);
            }

            if let Some((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Dendrite, dendrite_direction) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                bitmap.insert(target);
            }
        }

        Page {
            cells: cells,
            active: bitmap,
            changes: HashMap::new()
        }
    }

    pub fn grow(&mut self) -> u32 {

        debug!("Growing {} cells.", self.active.len());
        debug!("Changelist size: {}", self.changes.len());
        if self.active.is_empty() == true {
            return 0;
        }

        let mut cells = &mut self.cells;

        for index in self.active.iter() {
            let (x, y) = zorder::z_to_xy(index);
            let cell_type = cells[index as usize].get_cell_type();

            debug!("active index: {}  ({},{})", index, x, y);

            // Explicitly going to clobber existing changes for simplicity
            if cells[index as usize].chromosome_contains(Chromosome::North) {
                if y < 63 {
                    if let Some((target, change)) = Page::grow_local(cells, x, y, cell_type, Gate::North) {
                        trace!("Inserting {:?} (Exists? {})", change, self.changes.get(&target).is_some());
                        self.changes.insert(target, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::South) {
                if y > 0 {
                    if let Some((target, change)) = Page::grow_local(cells, x, y, cell_type, Gate::South) {
                        trace!("Inserting {:?} (Exists? {})", change, self.changes.get(&target).is_some());
                        self.changes.insert(target, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::East) {
                if x < 63 {
                    if let Some((target, change)) = Page::grow_local(cells, x, y, cell_type, Gate::East) {
                        trace!("Inserting {:?} (Exists? {})", change, self.changes.get(&target).is_some());
                        self.changes.insert(target, change);
                    }
                }
            }

            if cells[index as usize].chromosome_contains(Chromosome::West) {
                if x > 0 {
                    if let Some((target, change)) = Page::grow_local(cells, x, y, cell_type, Gate::West) {
                        trace!("Inserting {:?} (Exists? {})", change, self.changes.get(&target).is_some());
                        self.changes.insert(target, change);
                    }
                }
            }
        }

        debug!("After growth: Changelist size: {}", self.changes.len());

        // Return the number of newly activated cells
        self.changes.len() as u32
    }

    pub fn update(&mut self) {

        debug!("Updating {} cells.", self.changes.len());

        // Clear out the active cell bitmap, and add the cells we just grew
        debug!("Stale active cells: {}", self.active.len());
        self.active.clear();
        debug!("Cleared active cells: {}", self.active.len());

        if self.changes.len() == 0 {
            return;
        }

        let mut i = 0;
        for (k, v) in &self.changes {
            //if i == 0 {
                //debug!("To update: {} from {:?} to {:?}", k, self.cells[*k as usize].get_cell_type(), v.get_cell_type());
            //}

            i += 1;

            //let (x, y) = zorder::z_to_xy(k);
            self.cells[*k as usize].set_cell_type(v.get_cell_type());
            self.cells[*k as usize].set_gate(v.get_gate());
            self.active.insert(*k);
        }

        self.changes.clear();
        debug!("New active cells: {}", self.active.len());
        debug!("New Change list: {}", self.changes.len());
    }

    // TODO use i64 instead, so we can check for accidental negatives?
    fn grow_local(cells: &mut Vec<Cell>, x: u32, y: u32, cell_type: CellType, direction: Gate) -> Option<(u32, Cell)> {
        assert!((x > 63 && direction == Gate::East) != true);
        assert!((y > 63 && direction == Gate::North) != true);

        let (target, gate) = match direction {
            Gate::North => (zorder::xy_to_z(x, y + 1), Gate::South),
            Gate::South => (zorder::xy_to_z(x, y - 1), Gate::North),
            Gate::East => (zorder::xy_to_z(x + 1, y), Gate::West),
            Gate::West => (zorder::xy_to_z(x - 1, y), Gate::East),
        };

        Page::create_change(&mut cells[target as usize], cell_type, gate).map(|cell| (target, cell))
    }

    fn create_change(cell: &mut Cell, cell_type: CellType, gate: Gate) -> Option<Cell>{
        if cell.get_cell_type() == CellType::Empty {
            // TODO reuse from a pool of allocated cells?
            let mut change = Cell::new();
            change.set_cell_type(cell_type);
            change.set_gate(gate);
            return Some(change);
        }
        None
    }

    pub fn get_cell<'a>(&'a self, x: u32, y: u32) -> &'a Cell {
        let z = zorder::xy_to_z(x, y);
        &self.cells[z as usize]
    }
}


#[cfg(test)]
mod test {
    use super::{Page, GrowthPhase, Cell, CellType, Gate, Chromosome};

    #[test]
    fn page_new() {
        let p = Page::new(0.05);
    }

    #[test]
    fn grow() {
        let mut p = Page::new(0.05);
        p.grow(GrowthPhase::Axon);
    }

    #[test]
    fn create_change_empty() {
        let mut cell = Cell::new();
        assert!(cell.get_cell_type() == CellType::Empty);
        assert!(cell.get_gate() == Gate::North);

        let change = Page::create_change(&mut cell, CellType::Axon, Gate::North);
        assert!(change.is_some() == true);

        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::North);
    }

    #[test]
    fn create_change_non_empty() {
        let mut cell = Cell::new();
        assert!(cell.get_cell_type() == CellType::Empty);
        assert!(cell.get_gate() == Gate::North);

        cell.set_cell_type(CellType::Dendrite);
        cell.set_gate(Gate::South);
        assert!(cell.get_cell_type() == CellType::Dendrite);
        assert!(cell.get_gate() == Gate::South);

        let change = Page::create_change(&mut cell, CellType::Axon, Gate::North);
        assert!(change.is_none() == true);

        assert!(cell.get_cell_type() == CellType::Dendrite);
        assert!(cell.get_gate() == Gate::South);
    }

    #[test]
    fn grow_local() {
        let mut data = vec![Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        let change = Page::grow_local(&mut data, 0, 0, CellType::Axon, Chromosome::North);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::South); // Gate is opposite of the growth direction

        let change = Page::grow_local(&mut data, 1, 0, CellType::Dendrite, Chromosome::West);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        let change = change.unwrap();
        assert!(change.get_cell_type() == CellType::Dendrite);
        assert!(change.get_gate() == Gate::East);   // Gate is opposite of the growth direction
    }


    #[test]
    #[should_panic]
    fn grow_local_bad_north() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 0, 63, CellType::Axon, Chromosome::North);
    }

    #[test]
    #[should_panic]
    fn grow_local_bad_east() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 63, 0, CellType::Axon, Chromosome::East);
    }
}
