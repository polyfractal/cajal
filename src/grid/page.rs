
use roaring::RoaringBitmap;
use std::collections::HashMap;
use rand::distributions::{IndependentSample, Range};
use rand::{Rng, SeedableRng, StdRng};

pub use super::cell::{Cell, Chromosome, CellType, Gate};
use super::zorder;
use super::super::{ReportMemory, PAGE_SIZE, PAGE_WIDTH};
use self::ChangeType::{Remote, Local, NoChange};


static CARDINAL_DIRECTIONS: &'static [Gate] = &[Gate::North, Gate::South, Gate::East, Gate::West];

pub struct Page {
    cells: Vec<Cell>,
    active: RoaringBitmap<u32>,
    changes: HashMap<u32, Cell>,
    remote_changes: Vec<RemoteChange>,
    local_signal: Vec<LocalSignal>,
    remote_signal: Vec<RemoteSignal>,
    offset_x: u32,
    offset_y: u32
}

#[derive(Debug, Copy, Clone)]
pub struct RemoteChange {
    pub x: u32,
    pub y: u32,
    pub cell: Cell,
    pub travel_direction: Gate,
    pub stim: bool
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

#[derive(Debug, Copy, Clone)]
enum SignalType {
    Local(LocalSignal),
    Remote(RemoteSignal),
    NoSignal
}

#[derive(Debug, Copy, Clone)]
pub struct RemoteSignal {
    pub x: u32,
    pub y: u32,
    pub strength: u8,
    pub stim: bool,
    pub origin_cell_type: CellType
}

#[derive(Debug, Copy, Clone)]
pub struct LocalSignal {
    pub x: u32,
    pub y: u32,
    pub to_index: usize,
    pub strength: u8,
    pub stim: bool,
    pub origin_cell_type: CellType
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
        let mut final_seed = vec![offset_x as usize, offset_y as usize];
        final_seed.extend(seed);
        let mut rng: StdRng = SeedableRng::from_seed(final_seed.as_slice());

        let mut cells: Vec<Cell> = Vec::with_capacity(PAGE_SIZE as usize);
        let range_threshold = Range::new(0, 4);

        for _ in 0..PAGE_SIZE as usize {
            let mut cell = Cell::new();
            cell.set_chromosome(rng.gen());
            cell.set_gate(rng.gen());
            cell.set_threshold(range_threshold.ind_sample(&mut rng));
            cells.push(cell);
        }

        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();

        // TODO roll this into the initialization loop
        let active_cells: u32 = (PAGE_SIZE as f32 * density).round() as u32;
        debug!("Active cells in this Page: {}", active_cells);

        let range_cells = Range::new(1, PAGE_WIDTH - 1);

        for _ in 0..active_cells {
            let (x, y) = (range_cells.ind_sample(&mut rng), range_cells.ind_sample(&mut rng));
            let index = zorder::xy_to_z(x, y);

            cells[index as usize].set_cell_type(CellType::Body);
            let stim: bool = rng.gen();
            cells[index as usize].set_stim(stim);

            let axon_direction: Gate = cells[index as usize].get_gate();
            let secondary_axon = match axon_direction {
                Gate::North => Gate::West,
                Gate::West => Gate::South,
                Gate::South => Gate::East,
                Gate::East => Gate::North,
            };

            let dendrite_direction = !axon_direction;
            let secondary_dendrite = !secondary_axon;

            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Axon, axon_direction, stim) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                cells[target as usize].set_stim(stim);
                bitmap.insert(target);
            }
            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Axon, secondary_axon, stim) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                cells[target as usize].set_stim(stim);
                bitmap.insert(target);
            }

            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Dendrite, dendrite_direction, false) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                cells[target as usize].set_stim(false);
                bitmap.insert(target);
            }
            if let Local((target, change)) = Page::grow_local(&mut cells, x, y, CellType::Dendrite, secondary_dendrite, false) {
                cells[target as usize].set_cell_type(change.get_cell_type());
                cells[target as usize].set_gate(change.get_gate());
                cells[target as usize].set_stim(false);
                bitmap.insert(target);
            }
        }

        Page {
            cells: cells,
            active: bitmap,
            changes: HashMap::new(),
            offset_x: offset_x,
            offset_y: offset_y,
            remote_changes: Vec::with_capacity(32),
            remote_signal: Vec::with_capacity(32),
            local_signal: Vec::with_capacity(32)
        }
    }

    pub fn grow(&mut self) {

        debug!("Growing {} cells.", self.active.len());
        debug!("Changelist size: {}", self.changes.len());
        if self.active.is_empty() == true {
            return;
        }

        let mut cells = &mut self.cells;

        for index in self.active.iter() {
            let (x, y) = zorder::z_to_xy(index);
            let cell_type = cells[index as usize].get_cell_type();
            let stim = cells[index as usize].get_stim();

            for direction in CARDINAL_DIRECTIONS {
                if cells[index as usize].get_chromosome().contains(Chromosome::from(*direction)) {
                    let change = Page::process_chromosome_direction(*direction, &mut cells, x, y,
                                    self.offset_x, self.offset_y, cell_type, stim);

                    Page::persist_change(&mut self.changes, &mut self.remote_changes, change);
                }
            }
        }

        debug!("After growth: Changelist size: {}", self.changes.len());

    }

    pub fn get_remote_changes(&self) -> &Vec<RemoteChange> {
        &self.remote_changes
    }

    pub fn get_active_cell_count(&self) -> u32 {
        self.changes.len() as u32
    }

    fn persist_change(local: &mut HashMap<u32, Cell>, remote: &mut Vec<RemoteChange>, change: ChangeType) {
        match change {
            Local((target, change)) => {local.insert(target, change);},
            Remote(change) => {remote.push(change);},
            NoChange => {}
        }
    }

    fn process_chromosome_direction(travel_direction: Gate, cells: &mut Vec<Cell>,
                                x: u32, y: u32, offset_x: u32, offset_y: u32, cell_type: CellType, stim: bool) -> ChangeType {

        match (travel_direction, x, y) {
            (Gate::North, _, y) if y < PAGE_WIDTH - 1 => Page::grow_local(cells, x, y, cell_type, travel_direction, stim),
            (Gate::South, _, y) if y > 0  => Page::grow_local(cells, x, y, cell_type, travel_direction, stim),
            (Gate::East, x, _) if x < PAGE_WIDTH - 1  => Page::grow_local(cells, x, y, cell_type, travel_direction, stim),
            (Gate::West, x, _) if x > 0   => Page::grow_local(cells, x, y, cell_type, travel_direction, stim),
            (_, _, _) =>  Page::create_remote_change(x, y, offset_x, offset_y, cell_type, travel_direction, stim)
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
            self.cells[*k as usize].set_stim(v.get_stim());
            self.active.insert(*k);
        }

        self.changes.clear();
        self.remote_changes.clear();
        debug!("New active cells: {}", self.active.len());
        debug!("New Change list: {}", self.changes.len());
    }

    pub fn add_change(&mut self, x: u32, y: u32, cell: Cell, travel_direction: Gate, stim: bool) {
        debug!("Attempting to add remote change: ({}, {})", x, y);
        let cell_type = cell.get_cell_type();

        let target = zorder::xy_to_z(x, y);
        if self.cells[target as usize].get_cell_type() == CellType::Empty {
            debug!("Inserting external change.");
            self.changes.insert(target, Page::create_change(cell_type, !travel_direction, stim));
        }
    }

    fn create_remote_change(x: u32, y: u32, offset_x: u32, offset_y: u32,
                            cell_type: CellType, travel_direction: Gate, stim: bool) -> ChangeType {
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
            cell: Page::create_change(cell_type, !travel_direction, stim),
            travel_direction: travel_direction,
            stim: stim
        })
    }

    // TODO use i64 instead, so we can check for accidental negatives?
    fn grow_local(cells: &mut Vec<Cell>, x: u32, y: u32,
            cell_type: CellType, travel_direction: Gate, stim: bool) -> ChangeType {
        assert!((x > PAGE_WIDTH - 1 && travel_direction == Gate::East) != true);
        assert!((y > PAGE_WIDTH - 1 && travel_direction == Gate::North) != true);

        let (target, gate) = Page::calc_target(x, y, travel_direction);

        if cells[target as usize].get_cell_type() == CellType::Empty {
            Local((target, Page::create_change(cell_type, gate, stim)))
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

    fn create_change(cell_type: CellType, gate: Gate, stim: bool) -> Cell {
        // TODO reuse from a pool of allocated cells?
        let mut change = Cell::new();
        change.set_cell_type(cell_type);
        change.set_gate(gate);
        change.set_stim(stim);
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



    //---------------------------------

    pub fn set_input(&mut self, x: u32, y: u32, sig: u8) {
        debug!("Adding artificial signal to ({}, {}) @ {}", x, y, sig);
        let z = zorder::xy_to_z(x, y);
        self.cells[z as usize].set_signal(sig);
        self.active.insert(z);
    }

    pub fn signal(&mut self) {

        debug!("Processing signals for {} cells.", self.active.len());
        if self.active.is_empty() == true {
            return;
        }

        for index in self.active.iter() {

            let threshold = self.cells[index as usize].get_threshold();
            let signal = self.cells[index as usize].get_signal();
            debug!("Threshold: {}, Signal: {}", threshold, signal);

            if signal < threshold {
                continue;
            }

            let (x, y) = zorder::z_to_xy(index);

            match self.cells[index as usize].get_cell_type() {
                CellType::Axon => {
                    debug!("Signal landed on Axon");

                    let targets = Chromosome::from(self.cells[index as usize].get_gate()).invert();
                    debug!("Axon targets: {:?}", targets);

                    for direction in CARDINAL_DIRECTIONS {
                        if targets.contains(Chromosome::from(*direction)) {
                            debug!("Target contains {:?}", direction);

                            let sig = Page::process_signal(*direction, &mut self.cells, index as usize, x, y,
                                            self.offset_x, self.offset_y);

                            Page::persist_signal(&mut self.local_signal, &mut self.remote_signal, sig);
                        }
                    }
                },
                CellType::Dendrite | CellType::Body => {
                    debug!("Signal landed on Dendrite / Body");

                    let target = self.cells[index as usize].get_gate();
                    debug!("Signal >= threshold, send to: {:?}", target);

                    let sig = Page::process_signal(target, &mut self.cells, index as usize, x, y,
                                    self.offset_x, self.offset_y);

                    //debug!("Propagated signal: {:?}", sig);

                    Page::persist_signal(&mut self.local_signal, &mut self.remote_signal, sig);

                },
                _ => {}
            };
        }

        debug!("local_signal: {:?}", self.local_signal);
        debug!("remote_signal: {:?}", self.remote_signal);
    }

    pub fn get_remote_signal(&self) -> &Vec<RemoteSignal> {
        &self.remote_signal
    }

    fn process_signal(travel_direction: Gate, cells: &mut Vec<Cell>, origin: usize, x: u32, y: u32,
                        offset_x: u32, offset_y: u32) -> SignalType {

        match (travel_direction, x, y) {
            (Gate::North, _, y) if y < PAGE_WIDTH - 1 => Page::signal_local(cells, origin, x, y, travel_direction),
            (Gate::South, _, y) if y > 0  => Page::signal_local(cells, origin, x, y, travel_direction),
            (Gate::East, x, _) if x < PAGE_WIDTH - 1  => Page::signal_local(cells, origin, x, y, travel_direction),
            (Gate::West, x, _) if x > 0   => Page::signal_local(cells, origin, x, y, travel_direction),
            (_, _, _) =>  {
                let strength = cells[origin].get_strength();
                let stim = cells[origin].get_stim();
                let cell_type = cells[origin].get_cell_type();
                Page::signal_remote(strength, stim, x, y, offset_x, offset_y, travel_direction, cell_type)
            }
        }
    }


    fn persist_signal(local: &mut Vec<LocalSignal>, remote: &mut Vec<RemoteSignal>, change: SignalType) {
        match change {
            SignalType::Local(l) => {
                local.push(l);
            },
            SignalType::Remote(r) => {
                remote.push(r);
            },
            SignalType::NoSignal => {}
        }
    }


    fn signal_local(cells: &mut Vec<Cell>, origin: usize, x: u32, y: u32, travel_direction: Gate) -> SignalType {
        assert!((x > PAGE_WIDTH - 1 && travel_direction == Gate::East) != true);
        assert!((y > PAGE_WIDTH - 1 && travel_direction == Gate::North) != true);

        let (target, _) = Page::calc_target(x, y, travel_direction);

        if cells[target as usize].get_cell_type() != CellType::Empty {
            SignalType::Local(LocalSignal {
                x: x,
                y: y,
                to_index: target as usize,
                strength: cells[origin].get_strength(),
                stim: cells[origin].get_stim(),
                origin_cell_type: cells[origin].get_cell_type()
            })
        } else {
            SignalType::NoSignal
        }
    }

    fn signal_remote(strength: u8, stim: bool, x: u32, y: u32,
                offset_x: u32, offset_y: u32, travel_direction: Gate, cell_type: CellType) -> SignalType {
        debug!("signal_remote: ({},{}) offsets: ({},{}) {:?}", x, y, offset_x, offset_y, travel_direction);

        if (offset_x == 0 && travel_direction == Gate::West) || (offset_y == 0 && travel_direction == Gate::South) {
            return SignalType::NoSignal;
        }

        let (x, y) = match travel_direction {
            Gate::North => (offset_x + x,     offset_y + y + 1),
            Gate::South => (offset_x + x,     offset_y + y - 1),
            Gate::West  => (offset_x + x - 1, offset_y + y),
            Gate::East  => (offset_x + x + 1, offset_y + y),
        };
        debug!("signal-remote: new: ({},{}) {:?}", x, y, travel_direction);

        SignalType::Remote(RemoteSignal {
            x: x,
            y: y,
            strength: strength,
            stim: stim,
            origin_cell_type: cell_type
        })
    }

    pub fn update_signal(&mut self) -> u32 {

        debug!("Stale active cells: {}", self.active.len());

        /*
        for index in self.active.iter() {
            let threshold = self.cells[index as usize].get_threshold();
            let signal = self.cells[index as usize].get_signal();

            if signal >= threshold {
                self.cells[index as usize].clear_signal();
            }
        }
        */
        self.active.clear();

        if self.local_signal.is_empty() {
            return 0;
        }

        debug!("Local signals to process: {}", self.local_signal.len());
        for signal in &self.local_signal {


            match (signal.origin_cell_type, self.cells[signal.to_index].get_cell_type()) {
                (CellType::Axon, CellType::Axon) => {
                    let (x, y) = zorder::z_to_xy(signal.to_index as u32);
                    debug!("({},{}) - ({},{}) == ({},{})", x, y, signal.x, signal.y, x as i32 - signal.x as i32, y as i32 - signal.y as i32);
                    let (x, y): (i32, i32) = (x as i32 - signal.x as i32, y as i32 - signal.y as i32);


                    let direction = match (x,y) {
                        (1,0)  => Gate::West,
                        (-1,0) => Gate::East,
                        (0,1)  => Gate::South,
                        (0,-1) => Gate::North,
                        // This condition shouldn't really happen, but if it does (oops), invert
                        // the gate so it doesn't affect anything
                        (_, _) => !self.cells[signal.to_index].get_gate()
                    };

                    debug!("{:?} == {:?}?", direction, self.cells[signal.to_index].get_gate());
                    if direction == self.cells[signal.to_index].get_gate() {
                        self.cells[signal.to_index].add_signal(signal.strength);
                    }

                },
                (CellType::Axon, CellType::Dendrite) | (CellType::Axon, CellType::Body) => {
                    if signal.stim == true {
                        self.cells[signal.to_index].add_signal(signal.strength)
                    } else {
                        self.cells[signal.to_index].sub_signal(signal.strength)
                    }
                },
                (CellType::Dendrite, CellType::Dendrite)
                    | (CellType::Dendrite, CellType::Body)
                    | (CellType::Body, CellType::Dendrite)
                    | (CellType::Body, CellType::Body)
                    | (CellType::Body, CellType::Axon) => self.cells[signal.to_index].add_signal(signal.strength),
                (_, _) => {}
            }

            let from_index = zorder::xy_to_z(signal.x, signal.y);
            self.cells[from_index as usize].clear_signal();
            self.active.insert(signal.to_index as u32);
        }

        self.local_signal.clear();
        self.remote_signal.clear();
        debug!("After signaling, {} active cells", self.active.len());
        self.active.len()
    }

    pub fn add_signal(&mut self, x: u32, y: u32, strength: u8, stim: bool) {
        let target = zorder::xy_to_z(x, y);

        debug!(">>>>>>>> Attempting to add remote signal: ({}, {}) ({}): {} stimulatory? {}", x, y, target, strength, stim);
        if self.cells[target as usize].get_cell_type() != CellType::Empty {
            debug!("Inserting external signal");
            self.local_signal.push(LocalSignal {
                x: x,
                y: y,
                to_index: target as usize,
                strength: strength,
                stim: stim,
                origin_cell_type: CellType::Axon
            });
        }
    }


}


#[cfg(test)]
mod test {
    use super::{Page, Cell, CellType, Gate};
    use super::ChangeType::{Local};
    use test::Bencher;

    #[test]
    fn page_new() {
        let _ = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
    }

    #[test]
    fn grow() {
        let mut p = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
        p.grow();
    }

    #[test]
    fn create_change() {
        let change = Page::create_change(CellType::Axon, Gate::North, true);
        assert!(change.get_cell_type() == CellType::Axon);
        assert!(change.get_gate() == Gate::North);

        let change = Page::create_change(CellType::Dendrite, Gate::West, true);
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

        let change = Page::grow_local(&mut data, 0, 0, CellType::Axon, Gate::North, true);
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

        let change = Page::grow_local(&mut data, 1, 0, CellType::Dendrite, Gate::West, true);
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
        let _ = Page::grow_local(&mut data, 0, 63, CellType::Axon, Gate::North, true);
    }

    #[test]
    #[should_panic]
    fn grow_local_bad_east() {
        let mut data = vec![Cell::new(), Cell::new()];
        let _ = Page::grow_local(&mut data, 63, 0, CellType::Axon, Gate::East, true);
    }


    #[bench]
    fn bench_grow(b: &mut Bencher) {
        let mut page = Page::new(0.05, 0, 0, &[1, 2, 3, 4]);
        b.iter(|| {
            page.grow()
        });
    }

}
