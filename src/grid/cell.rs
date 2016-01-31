
use num::FromPrimitive;
use std::fmt;
use std::ops::{BitAnd, BitOr, Not};
use rand::{Rng};
use rand::Rand;
use rand::distributions::{IndependentSample, Range};

const CELL_TYPE_MASK: u32  = 0b0000000000_00_00_00_000000_0000_0_00_111;  // ---
const GATE_MASK: u32       = 0b0000000000_00_00_00_000000_0000_0_11_000;  // | Growth Phase
const STIM_MASK: u32       = 0b0000000000_00_00_00_000000_0000_1_00_000;  // |
const CHROMO_MASK: u32     = 0b0000000000_00_00_00_000000_1111_0_00_000;  // ---

const STRENGTH_MASK: u32   = 0b0000000000_00_00_00_000000_1111_0_00_000;  // --
const THRESHOLD_MASK: u32  = 0b0000000000_00_00_00_111111_0000_0_00_000;  // | Signal Phase
const POT_1_MASK: u32      = 0b0000000000_00_00_11_000000_0000_0_00_000;  // |
const POT_2_MASK: u32      = 0b0000000000_00_11_00_000000_0000_0_00_000;  // |
const POT_3_MASK: u32      = 0b0000000000_11_00_00_000000_0000_0_00_000;  // ---

const CELL_TYPE_OFFSET: u8  = 0;
const GATE_OFFSET: u8       = 3;
const STIM_OFFSET: u8       = 5;
const CHROMO_OFFSET: u8     = 6;

const STRENGTH_OFFSET: u8   = 6;
const THRESHOLD_OFFSET: u8  = 10;
const POT_1_OFFSET: u8      = 16;
const POT_2_OFFSET: u8      = 18;
const POT_3_OFFSET: u8      = 20;


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

impl Not for Gate {
    type Output = Gate;

    fn not(self) -> Gate {
        match self {
           Gate::North => Gate::South,
           Gate::South => Gate::North,
           Gate::East => Gate::West,
           Gate::West => Gate::East
        }
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

        NorthWest  = 0b0011,
        NorthSouth = 0b0101,
        NorthEast  = 0b1001,
        WestSouth  = 0b0110,
        WestEast   = 0b1010,
        SouthEast  = 0b1100,

        NorthWestSouth  = 0b0111,
        NorthEastSouth  = 0b1101,
        NorthWestEast   = 0b1011,
        WestSouthEast   = 0b1110,

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

            Chromosome::NorthWest  => 0b0011,
            Chromosome::NorthSouth => 0b0101,
            Chromosome::NorthEast  => 0b1001,
            Chromosome::WestSouth  => 0b0110,
            Chromosome::WestEast   => 0b1010,
            Chromosome::SouthEast  => 0b1100,

            Chromosome::NorthWestSouth  => 0b0111,
            Chromosome::NorthEastSouth  => 0b1101,
            Chromosome::NorthWestEast   => 0b1011,
            Chromosome::WestSouthEast   => 0b1110,

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

            5 => Chromosome::NorthWest,
            6 => Chromosome::NorthSouth,
            7 => Chromosome::NorthEast,
            8 => Chromosome::WestSouth,
            9 => Chromosome::WestEast,
            10 => Chromosome::SouthEast,

            11 => Chromosome::NorthWestSouth,
            12 => Chromosome::NorthEastSouth,
            13 => Chromosome::NorthWestEast,
            14 => Chromosome::WestSouthEast,

            15 => Chromosome::All,
            _ => unreachable!()
        }
    }
}

impl BitAnd for Chromosome {
    type Output = Chromosome;

    fn bitand(self, rhs: Chromosome) -> Chromosome {
        match Chromosome::from_u32(self as u32 & rhs as u32) {
            Some(c) => c,
            None => unreachable!()
        }
    }
}

impl BitOr for Chromosome {
    type Output = Chromosome;

    fn bitor(self, rhs: Chromosome) -> Chromosome {
        match Chromosome::from_u32(self as u32 | rhs as u32) {
            Some(c) => c,
            None => unreachable!()
        }
    }
}

impl From<Gate> for Chromosome {
    fn from(gate: Gate) -> Chromosome {
        match gate {
            Gate::North => Chromosome::North,
            Gate::South => Chromosome::South,
            Gate::East  => Chromosome::East,
            Gate::West  => Chromosome::West
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Cell {
    data: u32
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            data: 0
        }
    }

    pub fn set_cell_type(&mut self, cell_type: CellType) {
        self.data = (self.data & !CELL_TYPE_MASK) | ((cell_type as u32) << CELL_TYPE_OFFSET);
    }

    pub fn get_cell_type(&self) -> CellType {
        match CellType::from_u32((self.data & CELL_TYPE_MASK) >> CELL_TYPE_OFFSET) {
            Some(ct) => ct,
            None => unreachable!()
        }
    }

    pub fn set_gate(&mut self, gate: Gate) {
        self.data = (self.data & !GATE_MASK) | ((gate as u32) << GATE_OFFSET);
    }

    pub fn get_gate(&self) -> Gate {
        match Gate::from_u32((self.data & GATE_MASK) >> GATE_OFFSET) {
            Some(g) => g,
            None => unreachable!()
        }
    }

    pub fn set_chromosome(&mut self, chromo: Chromosome) {
        self.data = (self.data & !CHROMO_MASK) | ((chromo as u32) << CHROMO_OFFSET);
    }

    pub fn get_chromosome(&self) -> Chromosome {
        match Chromosome::from_u32((self.data & CHROMO_MASK) >> CHROMO_OFFSET) {
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

    pub fn set_threshold(&mut self, threshold: u8) {
        self.data = match threshold {
            0...63 => (self.data & !THRESHOLD_MASK) | ((threshold as u32) << THRESHOLD_OFFSET),
            _ => (self.data & !THRESHOLD_MASK) | (63 << THRESHOLD_OFFSET)
        };
    }

    pub fn get_threshold(&self) -> u8 {
        ((self.data & THRESHOLD_MASK) >> THRESHOLD_OFFSET) as u8
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

        let c = Cell::new();
        assert!(c.get_gate() == Gate::North);

        let c = Cell::new();
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

        c.set_chromosome(Chromosome::WestSouth);
        assert!(c.get_chromosome() == Chromosome::WestSouth);

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

        c.set_chromosome(Chromosome::NorthSouth);
        assert!(c.get_chromosome() == Chromosome::NorthSouth);
        assert!(!c.chromosome_contains(Chromosome::Block));
        assert!(c.chromosome_contains(Chromosome::North));
        assert!(c.chromosome_contains(Chromosome::South));
        assert!(c.chromosome_contains(Chromosome::NorthSouth));
    }

    #[test]
    fn set_threshold() {
        let mut c = Cell::new();
        assert!(c.get_threshold() == 0u8);

        c.set_threshold(16);
        assert!(c.get_threshold() == 16u8);

        // overflow
        c.set_threshold(90);
        assert!(c.get_threshold() == 63u8);
    }

}
