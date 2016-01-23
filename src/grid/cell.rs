
use num::FromPrimitive;
use std::fmt;
use std::ops::{BitAnd, BitOr, Shl};
use rand::{thread_rng, Rng};
use rand::Rand;
use rand::distributions::{IndependentSample, Range};

const CELL_TYPE_MASK: u16  = 0b000_0000_0000_00_111;
const GATE_MASK: u16      = 0b000_0000_0000_11_000;
const SIGNAL_MASK: u16    = 0b000_0000_1111_00_000;
const CHROMO_MASK: u16    = 0b000_1111_0000_00_000;

const CELL_TYPE_OFFSET: u8 = 0;
const GATE_OFFSET: u8      = 3;
const SIGNAL_OFFSET: u8    = 5;
const CHROMO_OFFSET: u8    = 9;

enum_from_primitive! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum CellType {
        Empty = 0b000,
        Body = 0b001,
        Axon = 0b010,
        Dendrite = 0b011
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Gate {
        North = 0b00,
        West = 0b01,
        South = 0b10,
        East = 0b11
    }
}

impl fmt::Binary for Gate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match *self {
            Gate::North => 0b00,
            Gate::West => 0b01,
            Gate::South => 0b10,
            Gate::East => 0b11
        };
        write!(f, "{:#b}", val)
    }
}

impl Rand for Gate {
    fn rand<R: Rng>(rng: &mut R) -> Gate {
        let range = Range::new(0, 3);
        match range.ind_sample(rng) {
            0 => Gate::North,
            1 => Gate::West,
            2 => Gate::South,
            3 => Gate::East,
            _ => unreachable!()
        }
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Chromosome {
        Block = 0b0000,
        North = 0b0001,
        West  = 0b0010,
        South = 0b0100,
        East  = 0b1000,

        North_West  = 0b0011,
        North_South = 0b0101,
        North_East  = 0b1001,
        West_South  = 0b0110,
        West_East   = 0b1010,
        South_East  = 0b1100,

        North_West_South  = 0b0111,
        North_East_South  = 0b1101,
        North_West_East   = 0b1011,
        West_South_East   = 0b1110,

        All = 0b1111

    }
}

impl fmt::Binary for Chromosome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match *self {
            Chromosome::Block => 0b0000,
            Chromosome::North => 0b0001,
            Chromosome::West => 0b0010,
            Chromosome::South => 0b0100,
            Chromosome::East => 0b1000,

            Chromosome::North_West  => 0b0011,
            Chromosome::North_South => 0b0101,
            Chromosome::North_East  => 0b1001,
            Chromosome::West_South  => 0b0110,
            Chromosome::West_East   => 0b1010,
            Chromosome::South_East  => 0b1100,

            Chromosome::North_West_South  => 0b0111,
            Chromosome::North_East_South  => 0b1101,
            Chromosome::North_West_East   => 0b1011,
            Chromosome::West_South_East   => 0b1110,

            Chromosome::All => 0b1111

        };
        write!(f, "{:#b}", val)
    }
}

impl Rand for Chromosome {
    fn rand<R: Rng>(rng: &mut R) -> Chromosome {
        let range = Range::new(0, 15);
        match range.ind_sample(rng) {
            0 => Chromosome::Block,
            1 => Chromosome::North,
            2 => Chromosome::West,
            3 => Chromosome::South,
            4 => Chromosome::East,

            5 => Chromosome::North_West,
            6 => Chromosome::North_South,
            7 => Chromosome::North_East,
            8 => Chromosome::West_South,
            9 => Chromosome::West_East,
            10 => Chromosome::South_East,

            11 => Chromosome::North_West_South,
            12 => Chromosome::North_East_South,
            13 => Chromosome::North_West_East,
            14 => Chromosome::West_South_East,

            15 => Chromosome::All,
            _ => unreachable!()
        }
    }
}

impl BitAnd for Chromosome {
    type Output = Chromosome;

    fn bitand(self, _rhs: Chromosome) -> Chromosome {
        match Chromosome::from_u16(self as u16 & _rhs as u16) {
            Some(c) => c,
            None => unreachable!()
        }
    }
}

impl BitOr for Chromosome {
    type Output = Chromosome;

    fn bitor(self, _rhs: Chromosome) -> Chromosome {
        match Chromosome::from_u16(self as u16 | _rhs as u16) {
            Some(c) => c,
            None => unreachable!()
        }
    }
}


#[derive(Clone, Copy)]
pub struct Cell {
    data: u16
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            data: 0
        }
    }

    pub fn set_cell_type(&mut self, cell_type: CellType) {
        self.data = (self.data & !CELL_TYPE_MASK) | ((cell_type as u16) << CELL_TYPE_OFFSET);
    }

    pub fn get_cell_type(&self) -> CellType {
        match CellType::from_u16((self.data & CELL_TYPE_MASK) >> CELL_TYPE_OFFSET) {
            Some(ct) => ct,
            None => unreachable!()
        }
    }

    pub fn set_gate(&mut self, gate: Gate) {
        self.data = (self.data & !GATE_MASK) | ((gate as u16) << GATE_OFFSET);
    }

    pub fn get_gate(&self) -> Gate {
        match Gate::from_u16((self.data & GATE_MASK) >> GATE_OFFSET) {
            Some(g) => g,
            None => unreachable!()
        }
    }

    pub fn set_chromosome(&mut self, chromo: Chromosome) {
        //println!("chromo: {:#b}", chromo);
        //println!("mask: {:#b}", CHROMO_MASK);
        //println!("self.data: {:#b}", self.data);
        //println!("shift and mask: {:#b}", (self.data & !CHROMO_MASK) | ((chromo as u16) << CHROMO_OFFSET));
        self.data = (self.data & !CHROMO_MASK) | ((chromo as u16) << CHROMO_OFFSET);
    }

    pub fn get_chromosome(&self) -> Chromosome {
        match Chromosome::from_u16((self.data & CHROMO_MASK) >> CHROMO_OFFSET) {
            Some(c) => c,
            None => unreachable!()
        }
    }

    pub fn chromosome_contains(&self, other: Chromosome) -> bool {
        match other {
            // Block is special since it can't co-exist with other flags
            Chromosome::Block => self.get_chromosome() == Chromosome::Block,
            _ => (self.get_chromosome() & other) == other
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Cell, Gate, Chromosome};

    #[test]
    fn toggle_gates() {
        let mut c = Cell::new();
        assert!(c.get_gate() == Gate::North);

        c.set_gate(Gate::South);
        assert!(c.get_gate() == Gate::South);

        c.set_gate(Gate::East);
        assert!(c.get_gate() == Gate::East);

        c.set_gate(Gate::West);
        assert!(c.get_gate() == Gate::West);

        let mut c = Cell::new();
        assert!(c.get_gate() == Gate::North);

        let mut c = Cell::new();
        assert!(c.get_gate() == Gate::North);
    }

    #[test]
    fn toggle_chromo() {
        let mut c = Cell::new();
        assert!(c.get_chromosome() == Chromosome::Block);

        c.set_chromosome(Chromosome::North);
        assert!(c.get_chromosome() == Chromosome::North);

        c.set_chromosome(Chromosome::South);
        assert!(c.get_chromosome() == Chromosome::South);

        c.set_chromosome(Chromosome::Block);
        assert!(c.get_chromosome() == Chromosome::Block);

        c.set_chromosome(Chromosome::West_South);
        assert!(c.get_chromosome() == Chromosome::West_South);

        c.set_chromosome(Chromosome::North);
        assert!(c.get_chromosome() == Chromosome::North);

        c.set_chromosome(Chromosome::East);
        assert!(c.get_chromosome() == Chromosome::East);

        c.set_chromosome(Chromosome::All);
        assert!(c.get_chromosome() == Chromosome::All);
    }

    #[test]
    fn toggle_chromo_and_gate() {
        let mut c = Cell::new();
        assert!(c.get_chromosome() == Chromosome::Block);
        assert!(c.get_gate() == Gate::North);

        c.set_chromosome(Chromosome::North);
        assert!(c.get_chromosome() == Chromosome::North);

        c.set_gate(Gate::South);
        assert!(c.get_gate() == Gate::South);
        assert!(c.get_chromosome() == Chromosome::North);
    }

    #[test]
    fn chromo_contains() {
        let mut c = Cell::new();
        assert!(c.get_chromosome() == Chromosome::Block);
        assert!(c.chromosome_contains(Chromosome::Block));
        assert!(!c.chromosome_contains(Chromosome::North));

        c.set_chromosome(Chromosome::North_South);
        assert!(c.get_chromosome() == Chromosome::North_South);
        assert!(!c.chromosome_contains(Chromosome::Block));
        assert!(c.chromosome_contains(Chromosome::North));
        assert!(c.chromosome_contains(Chromosome::South));
        assert!(c.chromosome_contains(Chromosome::North_South));
    }

}
