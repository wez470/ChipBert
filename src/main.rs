use std::path::PathBuf;
use structopt::StructOpt;
use std::time::Duration;
use std::thread;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const RAM: usize = 4096;
const ROM_START: usize = 0x200;

pub struct Emulator {
    ram: Box<[u8]>,
    // Memory address register
    i: u16,
    pc: u16,
    screen: Box<[u8]>,
    input: Input,
}

pub struct Input {}

impl Emulator {
    pub fn new(rom: Box<[u8]>) -> Emulator {
        let mut e = Emulator {
            ram: vec![0; RAM].into_boxed_slice(),
            i: 0,
            pc: ROM_START as u16,
            screen: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            input: Input{}
        };
        for i in 0..rom.len() {
            e.ram[i + ROM_START] = rom[i];
        }
        e
    }

    pub fn run(&mut self) {
        let inst = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;
        println!("PC=0x{:04X}: {:04X}", self.pc, inst);
        self.pc += 2;
    }
}

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    rom_path: PathBuf,
}

fn main() {
    let cli_args = CliArgs::from_args();
    let rom = std::fs::read(cli_args.rom_path)
        .expect("Failed to read ROM file")
        .into_boxed_slice();
    let mut emulator = Emulator::new(rom);

    while true {
        emulator.run();
        thread::sleep(Duration::from_millis(250))
    }
}
