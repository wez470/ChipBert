const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const RAM: usize = 4096;

pub struct Emulator {
    ram: Box<[u8]>,
    // Memory address register
    i: u16,
    screen: Box<[u8]>,
    input: Input
}

pub struct Input {}


impl Emulator {
    pub fn new() -> Emulator {
        Emulator{
            ram: vec![0; RAM].into_boxed_slice(),
            i: 0,
            screen: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            input: Input{}
        }
    }

    pub fn run(instr: u16) {
        unimplemented!("TODO: Run fn")
    }
}

fn main() {
    let program = "0111".as_bytes();
    let emulator = Emulator::new();

    while True {
        emulator.run((program[0] as u16) << 8 | program[1] as u16)
    }
}
