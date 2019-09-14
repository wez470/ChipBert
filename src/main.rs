use std::path::PathBuf;
use structopt::StructOpt;
use std::time::Duration;
use std::thread;
use rand::random;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const RAM: usize = 4096;
const ROM_START: usize = 0x200;
const FONT_LENGTH: usize = 80;
const FONT_START: usize = 0x50;
const FONTS: [u8; FONT_LENGTH] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    ram: Box<[u8]>,
    // Memory address register
    i: u16,
    pc: u16,
    screen: Box<[u8]>,
    input: Input,
    stack: Vec<u16>,
    regs: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
}

pub struct Input {}

impl Emulator {
    pub fn new(rom: Box<[u8]>) -> Emulator {
        let mut e = Emulator {
            ram: vec![0; RAM].into_boxed_slice(),
            i: 0,
            pc: ROM_START as u16,
            stack: Vec::new(),
            screen: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            input: Input{},
            regs: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
        };
        for i in 0..FONT_LENGTH {
            e.ram[i + FONT_START] = FONTS[i]
        }
        for i in 0..rom.len() {
            e.ram[i + ROM_START] = rom[i];
        }
        e
    }

    pub fn run(&mut self) {
        let inst = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;
        println!("PC=0x{:04X}: {:04X}", self.pc, inst);
        let nibbles =
            ((inst >> 12),
            (inst >> 8 & 0b1111),
            (inst >> 4 & 0b1111),
            (inst & 0b1111));
        match nibbles {
            (0, 0, 0xE, 0) => self.clear_screen(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (0, _, _, _) => unimplemented!("CALL RCA 1802 program"),
            (1, _, _, _) => self.jump(inst & 0b1111_1111_1111),
            (2, _, _, _) => self.call(inst & 0b1111_1111_1111),
            (3, vx, _, _) => self.cond(self.regs[vx as usize] == inst as u8),
            (4, vx, _, _) => self.cond(self.regs[vx as usize] != inst as u8),
            (5, vx, vy, 0) => self.cond(self.regs[vx as usize] == self.regs[vy as usize]),
            (6, vx, _, _) => self.regs[vx as usize] = inst as u8,
            (7, vx, _, _) => self.regs[vx as usize] = self.regs[vx as usize].overflowing_add(inst as u8).0,
            (8, vx, vy, 0) => self.regs[vx as usize] = self.regs[vy as usize],
            (8, vx, vy, 1) => self.regs[vx as usize] = self.regs[vx as usize] | self.regs[vy as usize],
            (8, vx, vy, 2) => self.regs[vx as usize] = self.regs[vx as usize] & self.regs[vy as usize],
            (8, vx, vy, 3) => self.regs[vx as usize] = self.regs[vx as usize] ^ self.regs[vy as usize],
            (8, vx, vy, 4) => {
                let (res, carry) = self.regs[vx as usize].overflowing_add(self.regs[vy as usize]);
                self.regs[vx as usize] = res;
                self.regs[0xF] = carry as u8;
            },
            (8, vx, vy, 5) => {
                let (res, carry) = self.regs[vx as usize].overflowing_sub(self.regs[vy as usize]);
                self.regs[vx as usize] = res;
                self.regs[0xF] = !carry as u8;
            },
            (8, vx, _, 6) => {
                self.regs[0xF] = self.regs[vx as usize] & 1;
                self.regs[vx as usize] >>= 1;
            },
            (8, vx, vy, 7) => {
                let (res, carry) = self.regs[vy as usize].overflowing_sub(self.regs[vx as usize]);
                self.regs[vx as usize] = res;
                self.regs[0xF] = !carry as u8;
            },
            (8, vx, _, 0xE) => {
                self.regs[0xF] = self.regs[vx as usize] >> 7;
                self.regs[vx as usize] <<= 1;
            },
            (9, vx, vy, 0) => self.cond(self.regs[vx as usize] != self.regs[vy as usize]),
            (0xA, _, _, _) => self.i = inst & 0b1111_1111_1111,
            (0xB, _, _, _) => self.pc = self.regs[0] as u16 + inst & 0b1111_1111_1111,
            (0xC, vx, _, _) => self.regs[vx as usize] = random::<u8>() & (inst as u8),
            (0xD, vx, vy, n) => self.draw_screen(self.regs[vx as usize], self.regs[vy as usize], n as u8),
            (0xE, vx, 9, 0xE) => unimplemented!("KEY EQUAL"),
            (0xE, vx, 0xA, 1) => unimplemented!("KEY NOT EQUAL"),
            (0xF, vx, 0, 7) => self.regs[vx as usize] = self.delay_timer,
            (0xF, vx, 0, 0xA) => self.regs[vx as usize] = 0, // TODO: KEY,
            (0xF, vx, 1, 5) => self.delay_timer = self.regs[vx as usize],
            (0xF, vx, 1, 8) => self.sound_timer = self.regs[vx as usize],
            (0xF, vx, 1, 0xE) => self.i += self.regs[vx as usize] as u16,
            (0xF, vx, 2, 9) => self.i = FONT_START as u16 + self.regs[vx as usize] as u16,
            (0xF, vx, 3, 3) => {
                self.ram[self.i as usize] = (self.regs[vx as usize] >> 2) & 1;
                self.ram[self.i as usize] = (self.regs[vx as usize] >> 1) & 1;
                self.ram[self.i as usize] = self.regs[vx as usize] & 1
            },
            (0xF, vx, 5, 5) => {
                for i in 0..(vx + 1) {
                    self.ram[self.i + i] + self.regs[i];
                }
            },
            (0xF, vx, 6, 5) => {
                for i in 0..(vx + 1) {
                    self.regs[i] = self.ram[self.i + i];
                }
            },
            _ => unreachable!("Invalid instruction reached"),
        }

        self.pc += 2;
    }

    fn clear_screen(&mut self) {
        for i in 0..self.screen.len() {
            self.screen[i] = 0;
        }
    }

    fn ret(&mut self) {
        self.pc = self.stack.pop().expect("Returning on empty stack!")
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr
    }

    fn call(&mut self, fn_addr: u16) {
        self.stack.push(self.pc);
        self.pc = fn_addr;
    }

    fn cond(&mut self, cond: bool) {
        if cond {
            self.pc += 2;
        }
    }

    fn draw_screen(&mut self, base_x: u8, base_y: u8, height: u8) {
        println!("drawing sprite at x: {}, y: {}, height: {}", base_x, base_y, height);
        for y_i in 0..height {
            let y = (base_y + y_i) % SCREEN_HEIGHT as u8;
            for x_i in 0..8 {
                let x = (base_x + x_i) % SCREEN_WIDTH as u8;
                let pixel_i = (self.ram[self.i as usize + y_i as usize] >> x_i) & 1;
                self.screen[y as usize * SCREEN_WIDTH as usize + x as usize] = pixel_i;
            }
        }
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
