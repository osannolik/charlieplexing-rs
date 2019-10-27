#![no_std]

pub use embedded_hal::digital::v2::PinState as PinState;

// const generics would be cool...
// look into generic_array
const N_COLS: usize = 5;
const N_ROWS: usize = 6;
const N_PINS: usize = N_ROWS;

type Matrix<T> = [[T; N_COLS]; N_ROWS];
pub type BoolPatternMatrix = Matrix<bool>;

pub const ALL_ON: BoolPatternMatrix = [[true; N_COLS]; N_ROWS];
pub const ALL_OFF: BoolPatternMatrix = [[false; N_COLS]; N_ROWS];

const ROW_COL_TO_PIN: Matrix<usize> = [
    [1, 2, 3, 4, 5],
    [0, 2, 3, 4, 5],
    [0, 1, 3, 4, 5],
    [0, 1, 2, 4, 5],
    [0, 1, 2, 3, 5],
    [0, 1, 2, 3, 4],
];

pub trait ApplyPinState {
    fn apply(&mut self, row_nr: usize, state: PinState);
}

pub trait Charlieplexer {
    fn new(item: BoolPatternMatrix) -> Self;

    fn update(&mut self, pattern: BoolPatternMatrix);

    fn apply<T>(&self, gpio: &mut T)
        where T: ApplyPinState;

    fn next(&mut self) -> Sequencer;
}

pub type PinStates = [PinState; N_PINS];
pub const PINS_HIZ: PinStates = [PinState::Floating; N_PINS];
pub const PINS_HIGH: PinStates = [PinState::High; N_PINS];
pub const PINS_LOW: PinStates = [PinState::Low; N_PINS];

//const PIN_SEQUENCE_HIZ: PinSequence = [PINS_HIZ; N_ROWS];

type PinSequence = [PinStates; N_ROWS];

#[derive(Copy, Clone)]
pub struct Sequencer {
    sequence: PinSequence,
    index: usize,
}

impl Sequencer {
    fn row_to_pin_states(row: usize, pattern: &[bool]) -> PinStates {
        let mut pin_states = PINS_HIZ;

        for (col, &is_on) in pattern.iter().enumerate() {
            let pin = ROW_COL_TO_PIN[row][col];
            pin_states[pin] = if is_on { PinState::High } else { PinState::Floating };
        }

        pin_states[row] = if pattern.contains(&true) {
            PinState::Low
        } else {
            PinState::Floating
        };

        return pin_states;
    }

    fn to_pin_sequence(pattern: BoolPatternMatrix) -> PinSequence {
        let mut seq = [PINS_HIZ; N_ROWS];

        for (row, pattern) in pattern.iter().enumerate() {
            seq[row] = Self::row_to_pin_states(row, pattern);
        }

        return seq;
    }
}

impl Charlieplexer for Sequencer {
    fn new(item: BoolPatternMatrix) -> Sequencer {
        Sequencer {
            sequence: Self::to_pin_sequence(item),
            index: 0,
        }
    }

    fn update(&mut self, pattern: BoolPatternMatrix) {
        self.sequence = Self::to_pin_sequence(pattern);
    }

    fn apply<T>(&self, io: &mut T)
        where T: ApplyPinState,
    {
        for (row_nr, &level) in self.sequence[self.index].iter().enumerate() {
            io.apply(row_nr, level);
        }
    }

    fn next(&mut self) -> Sequencer {
        self.index = (self.index + 1) % self.sequence.len();
        *self
    }
}
