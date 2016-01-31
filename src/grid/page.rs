
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rand::distributions::{IndependentSample, Range};
use rand::{Rng, SeedableRng, StdRng};

pub use super::cell::{Cell, Chromosome, CellType, Gate};
use super::zorder;
use super::super::ReportMemory;
use self::ChangeType::{Remote, Local, NoChange};


static CARDINAL_DIRECTIONS: &'static [Gate] = &[Gate::North, Gate::South, Gate::East, Gate::West];

pub struct Page {
    cells: Vec<Cell>,
    active: RoaringBitmap<u32>,
    changes: HashMap<u32, Cell>,
    remote_changes: Vec<RemoteChange>,
    offset_x: u32,
    offset_y: u32
}

#[derive(Debug, Copy, Clone)]
pub struct RemoteChange {
    pub x: u32,
    pub y: u32,
    pub cell: Cell,
    pub travel_direction: Gate
}

enum ChangeType {
    Local((u32, Cell)),
    Remote(RemoteChange),
    NoChange
}

impl ChangeType {
    pub fn is_some(&self) -> bool {
        match *self {
            NoChange => false,
            _ => true
        }
    }
}

impl ReportMemory for Page {
    fn memory(&self) -> u32 {
        (self.cells.len() as u32 * 16) +
        (self.active.len() as u32 * 8) +  // <-- This is not true!
        ((self.changes.len() as u32 + self.changes.capacity() as u32) * 8) // Rough approximation
    }
}

impl Page {
    pub fn new(density: f32, offset_x: u32, offset_y: u32, seed: &[usize]) -> Page {
        debug!("Creating new Page with {} density.", density);
        //let mut rng = thread_rng();
        let mut rng: StdRng = SeedableRng::from_seed(seed);

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
            let dendrite_direction = !axon_direction;

            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Axon, axon_direction) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                bitmap.insert(target);
            }

            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Dendrite, dendrite_direction) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                bitmap.insert(target);
            }
        }

        Page {
            cells: cells,
            active: bitmap,
            changes: HashMap::new(),
            offset_x: offset_x,
            offset_y: offset_y,
            remote_changes: Vec::with_capacity(32)
        }
    }

    pub fn grow(&mut self) -> (u32, Option<Vec<RemoteChange>>) {

        debug!("Growing {} cells.", self.active.len());
        debug!("Changelist size: {}", self.changes.len());
        if self.active.is_empty() == true {
            return (0, None);
        }

        let mut cells = &mut self.cells;

        for index in self.active.iter() {
            let (x, y) = zorder::z_to_xy(index);
            let cell_type = cells[index as usize].get_cell_type();

            for direction in CARDINAL_DIRECTIONS {
                if cells[index as usize].chromosome_contains(Chromosome::from(*direction)) {
                    let change = Page::process_chromosome_direction(*direction, &mut cells, x, y,
                                    self.offset_x, self.offset_y, cell_type);

                    Page::persist_change(&mut self.changes, &mut self.remote_changes, change);
                }
            }
        }

        debug!("After growth: Changelist size: {}", self.changes.len());

        // Return the number of newly activated cells
        (self.changes.len() as u32, Some(self.remote_changes.clone()))
    }

    fn persist_change(local: &mut HashMap<u32, Cell>, remote: &mut Vec<RemoteChange>, change: ChangeType) {
        match change {
            Local((target, change)) => {local.insert(target, change);},
            Remote(change) => {remote.push(change);},
            NoChange => {}
        }
    }

    fn process_chromosome_direction(travel_direction: Gate, cells: &mut Vec<Cell>,
                                x: u32, y: u32, offset_x: u32, offset_y: u32, cell_type: CellType) -> ChangeType {

        match (travel_direction, x, y) {
            (Gate::North, _, y) if y < 63 => Page::grow_local(cells, x, y, cell_type, travel_direction),
            (Gate::South, _, y) if y > 0  => Page::grow_local(cells, x, y, cell_type, travel_direction),
            (Gate::East, x, _) if x < 63  => Page::grow_local(cells, x, y, cell_type, travel_direction),
            (Gate::West, x, _) if x > 0   => Page::grow_local(cells, x, y, cell_type, travel_direction),
            (_, _, _) =>  Page::create_remote_change(x, y, offset_x, offset_y, cell_type, travel_direction)
        }
    }

    pub fn update(&mut self) {

        debug!("Updating {} cells.", self.changes.len());

        // Clear out the active cell bitmap, and add the cells we just grew
        debug!("Stale active cells: {}", self.active.len());
        self.active.clear();
        debug!("Cleared active cells: {}", self.active.len());

        if self.changes.is_empty() {
            return;
        }

        for (k, v) in &self.changes {
            self.cells[*k as usize].set_cell_type(v.get_cell_type());
            self.cells[*k as usize].set_gate(v.get_gate());
            self.active.insert(*k);
        }

        self.changes.clear();
        self.remote_changes.clear();
        debug!("New active cells: {}", self.active.len());
        debug!("New Change list: {}", self.changes.len());
    }

    pub fn add_change(&mut self, x: u32, y: u32, cell: Cell, travel_direction: Gate) {
        debug!("Attempting to add remote change: ({}, {})", x, y);
        let cell_type = cell.get_cell_type();

        let target = zorder::xy_to_z(x, y);
        if self.cells[target as usize].get_cell_type() == CellType::Empty {
            debug!("Inserting external change.");
            self.changes.insert(target, Page::create_change(cell_type, !travel_direction));
        }
    }

    fn create_remote_change(x: u32, y: u32, offset_x: u32, offset_y: u32,
                            cell_type: CellType, travel_direction: Gate) -> ChangeType {
        debug!("create_remote_change: ({},{}) offsets: ({},{}) {:?}", x, y, offset_x, offset_y, travel_direction);

        if (offset_x == 0 && travel_direction == Gate::West) || (offset_y == 0 && travel_direction == Gate::South) {
            return NoChange;
        }

        let (x, y) = match travel_direction {
            Gate::North => (offset_x + x,     offset_y + y + 1),
            Gate::South => (offset_x + x,     offset_y + y - 1),
            Gate::West  => (offset_x + x - 1, offset_y + y),
            Gate::East  => (offset_x + x + 1, offset_y + y),
        };
        debug!("create_remote_change: new: ({},{}) {:?}", x, y, travel_direction);

        Remote(RemoteChange {
            x: x,
            y: y,
            cell: Page::create_change(cell_type, !travel_direction),
            travel_direction: travel_direction
        })
    }

    // TODO use i64 instead, so we can check for accidental negatives?
    fn grow_local(cells: &mut Vec<Cell>, x: u32, y: u32,
            cell_type: CellType, travel_direction: Gate) -> ChangeType {
        assert!((x > 63 && travel_direction == Gate::East) != true);
        assert!((y > 63 && travel_direction == Gate::North) != true);

        let (target, gate) = Page::calc_target(x, y, travel_direction);

        if cells[target as usize].get_cell_type() == CellType::Empty {
            Local((target, Page::create_change(cell_type, gate)))
        } else {
            NoChange
        }
    }


    fn calc_target(x: u32, y: u32, direction: Gate) -> (u32, Gate) {
        trace!("calc_target: ({},{}) -> {:?}", x, y, direction);
        match direction {
            Gate::North => (zorder::xy_to_z(x, y + 1), Gate::South),
            Gate::South => (zorder::xy_to_z(x, y - 1), Gate::North),
            Gate::East => (zorder::xy_to_z(x + 1, y), Gate::West),
            Gate::West => (zorder::xy_to_z(x - 1, y), Gate::East),
        }
    }

    fn create_change(cell_type: CellType, gate: Gate) -> Cell {
        // TODO reuse from a pool of allocated cells?
        let mut change = Cell::new();
        change.set_cell_type(cell_type);
        change.set_gate(gate);
        change
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &Cell {
        let z = zorder::xy_to_z(x, y);
        &self.cells[z as usize]
    }

    pub fn get_mut_cell(&mut self, x: u32, y: u32) -> &mut Cell {
        let z = zorder::xy_to_z(x, y);
        &mut self.cells[z as usize]
    }
}


#[cfg(test)]
mod test {
    use super::{Page, Cell, CellType, Gate, Chromosome};
    use super::ChangeType::{Remote, Local, NoChange};
    use test::Bencher;

    #[test]
    fn page_new() {
        let p = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
    }

    #[test]
    fn grow() {
        let mut p = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
        p.grow();
    }

    #[test]
    fn create_change() {
        let change = Page::create_change(CellType::Axon, Gate::North);
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::North);

        let change = Page::create_change(CellType::Dendrite, Gate::West);
        assert!(change.get_cell_type() == CellType::Dendrite);
        assert!(change.get_gate() == Gate::West);
    }

    #[test]
    fn grow_local() {
        let mut data = vec![Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        let change = Page::grow_local(&mut data, 0, 0, CellType::Axon, Gate::North);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        match change {
            Local((target, change)) => {
                assert!(change.get_cell_type() == CellType::Axon);
                assert!(change.get_gate() == Gate::South); // Gate is opposite of the growth direction
                assert!(target == 2);
            },
            _ => assert!(1 == 2)
        }

        let change = Page::grow_local(&mut data, 1, 0, CellType::Dendrite, Gate::West);
        assert!(data[0].get_cell_type() == CellType::Empty);
        assert!(data[0].get_gate() == Gate::North);
        assert!(data[1].get_cell_type() == CellType::Empty);
        assert!(data[1].get_gate() == Gate::North);

        assert!(change.is_some() == true);
        match change {
            Local((target, change)) => {
                assert!(change.get_cell_type() == CellType::Dendrite);
                assert!(change.get_gate() == Gate::East); // Gate is opposite of the growth direction
                assert!(target == 0);
            },
            _ => assert!(1 == 2)
        }

    }


    #[test]
    #[should_panic]
    fn grow_local_bad_north() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 0, 63, CellType::Axon, Gate::North);
    }

    #[test]
    #[should_panic]
    fn grow_local_bad_east() {
        let mut data = vec![Cell::new(), Cell::new()];
        let change = Page::grow_local(&mut data, 63, 0, CellType::Axon, Gate::East);
    }


    #[bench]
    fn bench_grow(b: &mut Bencher) {
        let mut page = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
        b.iter(|| {
            page.grow()
        });
    }

}
