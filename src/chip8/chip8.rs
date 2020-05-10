use std::fs;
use std::path::Path;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::chip8::{Opcode, Register, Address};

/// `Chip8` is the core emulation structure of this project. It implements the memory and opcodes
/// of the Chip-8 architecture.
pub struct Chip8 {
    /// Chip-8 memory is segmented into two sections:
    ///
    /// - `0x000-0x1FF`: Reserved for the Chip 8 interpreter.
    /// - `0x200-0xFFF`: Program ROM and RAM
    ///
    /// We only use `0x050-0x0A0` in the reserved memory for the built in 4x5 pixel font set with digits (0-9) and letters (A-F)
    pub memory: [u8; 4096],

    /// Stack holds the addresses to return to when the current subroutine finishes.
    pub stack: Vec<u16>,

    /// `gfx` represents the Chip-8 display. The Chip-8 has a 64x32 display consisting of an
    /// empty colour and a filled colour.
    ///
    /// If `gfx[y * Chip8::SCREEN_WIDTH + x]` is `0x0` then the pixel at `(x, y)` should be empty,
    /// otherwise it should be filled.
    ///
    /// The specific colour of "filled" and "empty" should be defined by the rendering system.
    pub gfx: [u8; Chip8::SCREEN_PIXELS],

    /// The Chip-8 has a 16 charcter keypad:
    ///
    /// ```text
    /// 1 2 3 C
    /// 4 5 6 D
    /// 7 8 9 E
    /// A 0 B F
    /// ```
    ///
    /// Keys are indexed by their hexadecimal number. For example: `keys[0xA]` gives the state of key `A`.
    ///
    /// Each key is either pressed (true) or released (false)
    pub keys: [bool; 16],

    /// General Purpose Registers `V0`, `V1`, ..., `VF`
    ///
    /// `VF` should not be used by Chip-8 programs. We use it as a flag for some opcodes.
    pub v: [u8; 16],

    /// Index Register: Generally used to store memory addresses which means only the lowest (rightmost) 12 bits are usually used
    pub i: u16,

    /// Program Counter. Points to the currently executing address in `memory`
    pub pc: u16,

    /// Delay Timer Register. When non-zero it decrements by 1 at the rate of 60hz.
    pub delay_timer: u8,

    /// Sound Timer Register. When non-zero it:
    ///
    /// - Decrements by 1 at a rate of 60hz
    /// - Sounds the Chip-8 buzzer.
    pub sound_timer: u8,

    /// Execution state, used to wait for keypresses
    state: Chip8State,

    /// Random Number Generator used for `Opcode::Random`
    rng: ChaCha8Rng,
}

#[derive(PartialEq)]
enum Chip8State {
    Running,
    WaitingForKey { target_register: Register }
}

#[derive(PartialEq)]
pub enum Chip8Output {
    None,
    Redraw
}

impl Chip8 {
    pub const SCREEN_WIDTH: usize = 64;
    pub const SCREEN_HEIGHT: usize = 32;
    pub const SCREEN_PIXELS: usize = Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT;

    const FONT_START: u16 = 0x50;
    const FONT_END: u16 = 0xA0;
    const FONTSET: [u8; 80] = [
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

    pub fn new() -> Chip8 {
        let mut chip8 = Chip8::empty();
        chip8.pc = 0x200;

        let font_start = Chip8::FONT_START as usize;
        let font_end = Chip8::FONT_END as usize;
        chip8.memory[font_start..font_end].copy_from_slice(&Chip8::FONTSET);

        chip8
    }

    pub fn new_with_rom(rom_bytes: Vec<u8>) -> Chip8 {
        let mut chip8 = Chip8::new();
        chip8.load_rom(rom_bytes);
        chip8
    }

    /// Returns a Chip8 with _no initialized memory_
    pub fn empty() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            stack: Vec::new(),
            gfx: [0; Chip8::SCREEN_PIXELS],
            keys: [false; 16],

            v: [0; 16],
            i: 0,
            pc: 0,

            delay_timer: 0,
            sound_timer: 0,

            state: Chip8State::Running,
            rng: ChaCha8Rng::from_entropy()
        }
    }

    pub fn with_seed(&mut self, seed: u64) -> &mut Self {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self
    }

    pub fn load_rom(&mut self, rom_bytes: Vec<u8>) {
        let rom_start = 0x200;
        let rom_end = rom_start + rom_bytes.len();
        self.memory[rom_start..rom_end].copy_from_slice(&rom_bytes[..]);
    }

    pub fn load_rom_from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let rom_bytes = fs::read(path)?;
        self.load_rom(rom_bytes);

        Ok(())
    }

    pub fn key(&mut self, key: u8, pressed: bool) {
        // Transition out of `WaitingForKey` when the correct key is released.
        if let Chip8State::WaitingForKey { target_register } = self.state {
            if pressed == false && self.keys[key as usize] == true {
                self.v[target_register as usize] = key;
                self.state = Chip8State::Running;
            }
        }

        self.keys[key as usize] = pressed;
    }

    pub fn press_key(&mut self, key: u8) {
        self.key(key, true);
    }

    pub fn release_key(&mut self, key: u8) {
        self.key(key, false);
    }

    pub fn current_opcode(&self) -> Opcode {
        self.read_opcode()
    }

    /// Execute one cycle of the chip8 interpreter.
    pub fn cycle(&mut self) -> Chip8Output {
        if self.state != Chip8State::Running {
            return Chip8Output::None;
        }

        let opcode = self.read_opcode();
        self.pc += 2;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        self.execute_opcode(opcode.clone());

        match opcode {
            Opcode::Draw { x: _, y: _, n: _ } => Chip8Output::Redraw,
            _ => Chip8Output::None,
        }
    }

    pub fn cycle_n(&mut self, times: u32) {
        for _ in 0..times {
            self.cycle();
        }
    }

    pub fn gfx_slice(&self, x_start: u8, x_end: u8, y_start: u8, y_end: u8) -> Vec<Vec<u8>> {
        let mut gfx_slice = Vec::new();

        for y in y_start..y_end {
            let mut row = Vec::new();

            for x in x_start..x_end {
                let y = y as usize;
                let x = x as usize;
                row.push(self.gfx[y * Chip8::SCREEN_WIDTH + x] as u8);
            }

            gfx_slice.push(row);
        }

        gfx_slice

    }

    pub fn gfx_to_string(&self) -> String {
        let mut gfx_string = String::new();

        let mut row = 0;
        loop {
            if row > (2048 - 64) { break; }

            gfx_string.push_str(&format!("{:?}\n", &self.gfx[row..row+64]));

            row += 64;
        }

        gfx_string
    }

    fn read_opcode(&self) -> Opcode {
        let pc = self.pc as usize;
        let opcode_bytes = [self.memory[pc], self.memory[pc+1]];
        Opcode::from_bytes(&opcode_bytes)
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode {
            // Flow Control
            Opcode::CallSubroutine(address) => self.op_call_subroutine(address),
            Opcode::Return => self.op_return(),
            Opcode::Jump(address) => self.pc = address,
            Opcode::JumpWithOffset(address) => self.pc = address + (self.v[0] as u16),

            // Conditional Execution
            Opcode::SkipNextIfEqual { x, value } => self.op_skip_next_if(self.v[x as usize] == value),
            Opcode::SkipNextIfNotEqual { x, value } => self.op_skip_next_if(self.v[x as usize] != value),
            Opcode::SkipNextIfRegisterEqual { x, y } => self.op_skip_next_if(self.v[x as usize] == self.v[y as usize]),
            Opcode::SkipNextIfRegisterNotEqual { x, y } => self.op_skip_next_if(self.v[x as usize] != self.v[y as usize]),

            // Manipulate `Vx`
            Opcode::LoadConstant { x, value } => self.v[x as usize] = value,
            Opcode::Load { x, y } => self.v[x as usize] = self.v[y as usize],
            Opcode::Or { x, y } => self.v[x as usize] = self.v[x as usize] | self.v[y as usize],
            Opcode::And { x, y } => self.v[x as usize] = self.v[x as usize] & self.v[y as usize],
            Opcode::Xor { x, y } => self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize],
            Opcode::Add { x, y } => self.op_add(x, y),
            Opcode::AddConstant { x, value } => self.v[x as usize] = self.v[x as usize].wrapping_add(value),
            Opcode::SubtractYFromX { x, y } => self.op_subtract_y_from_x(x, y),
            Opcode::SubtractXFromY { x, y } => self.op_subtract_x_from_y(x, y),
            Opcode::ShiftRight { x, y } => self.op_shift_right(x, y),
            Opcode::ShiftLeft { x, y } => self.op_shift_left(x, y),

            // Manipulate `I`
            Opcode::IndexAddress(address) => self.i = address,
            Opcode::AddAddress { x } => self.i += self.v[x as usize] as u16,
            Opcode::IndexFont { x } => self.i = Chip8::FONT_START + (x as u16 * 5),

            // Manipulate Memory
            Opcode::WriteMemory { x } => self.op_write_memory(x),
            Opcode::ReadMemory { x } => self.op_read_memory(x),
            Opcode::WriteBCD { x } => self.op_store_bcd(x),

            // IO Opcodes
            Opcode::SkipIfKeyPressed { x } => self.op_skip_if_key_pressed(x),
            Opcode::SkipIfKeyNotPressed { x } => self.op_skip_if_key_not_pressed(x),
            Opcode::WaitForKeyRelease { x } => self.state = Chip8State::WaitingForKey { target_register: x },
            Opcode::LoadDelayIntoRegister { x } => self.v[x as usize] = self.delay_timer,
            Opcode::LoadRegisterIntoDelay { x } => self.delay_timer = self.v[x as usize],
            Opcode::LoadRegisterIntoSound { x } => self.sound_timer = self.v[x as usize],
            Opcode::Random { x, mask } => self.op_rand(x, mask),
            Opcode::ClearScreen => self.gfx = [0; Chip8::SCREEN_PIXELS],
            Opcode::Draw { x, y, n } => self.op_draw(x, y, n),
        }
    }

    fn op_call_subroutine(&mut self, address: Address) {
        self.stack.push(self.pc);
        self.pc = address;
    }

    fn op_return(&mut self) {
        // TODO: Better error handling
        self.pc = self.stack.pop().expect("Stack Underflow!");
    }

    fn op_skip_next_if(&mut self, expression: bool) {
        if expression {
            self.pc += 2
        }
    }

    fn op_skip_if_key_pressed(&mut self, x: Register) {
        let key = self.v[x as usize];
        self.op_skip_next_if(self.keys[key as usize] == true)
    }

    fn op_skip_if_key_not_pressed(&mut self, x: Register) {
        let key = self.v[x as usize];
        self.op_skip_next_if(self.keys[key as usize] == false)
    }

    fn op_store_bcd(&mut self, x: Register) {
        let x = x as usize;
        let i = self.i as usize;

        self.memory[i] = self.v[x] / 100; // Value of the first digit
        self.memory[i + 1] = (self.v[x] / 10) % 10; // Value of the second digit
        self.memory[i + 2] = self.v[x] % 10; // Value of the third digit
    }

    fn op_rand(&mut self, x: Register, mask: u8) {
        let value: u8 = self.rng.gen();

        self.v[x as usize] = value & mask;
    }

    fn op_add(&mut self, x: Register, y: Register) {
        let (result, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = carry as u8;
    }

    fn op_subtract_y_from_x(&mut self, x: Register, y: Register) {
        let (result, carry) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = carry as u8;
    }

    fn op_subtract_x_from_y(&mut self, x: Register, y: Register) {
        let (result, carry) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = carry as u8;
    }

    fn op_shift_right(&mut self, x: Register, _: Register) {
        let least_significant_bit = self.v[x as usize] & 0b00000001;
        self.v[0xF] = least_significant_bit;
        self.v[x as usize] = self.v[x as usize].wrapping_shr(1);
    }

    fn op_shift_left(&mut self, x: Register, _: Register) {
        let most_significant_bit = (self.v[x as usize] >> 7) & 1;
        self.v[0xF] = most_significant_bit;
        self.v[x as usize] = self.v[x as usize].wrapping_shl(1);
    }

    fn op_draw(&mut self, x: Register, y: Register, n: u8) {
        self.v[0xF] = 0;
        let x = self.v[x as usize];
        let y = self.v[y as usize];

        for pixel_y in 0..n {
            let row_sprite: u8 = self.memory[(self.i + pixel_y as u16) as usize];
            let y = (y + pixel_y) as usize % Chip8::SCREEN_HEIGHT;

            for pixel_x in 0..8 {
                let bit = (row_sprite >> (7 - pixel_x)) & 0x1;
                if bit != 0 {
                    let x = (x + pixel_x) as usize % Chip8::SCREEN_WIDTH;
                    let pixel: &mut u8 = &mut self.gfx[(y * Chip8::SCREEN_WIDTH) + x];
                    if *pixel == 1 {
                        self.v[0xF] = 1;
                    }

                    *pixel ^= 1;
                }
            }
        }
    }

    fn op_write_memory(&mut self, x: Register) {
        for register in 0..=(x as usize) {
            self.memory[self.i as usize] = self.v[register];
            self.i += 1;
        }
    }

    fn op_read_memory(&mut self, x: Register) {
        for register in 0..=(x as usize) {
            self.v[register] = self.memory[self.i as usize];
            self.i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn program_counter_increases_after_cycle() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xF }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    pub fn op_call_subroutine_and_return() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            // Jump to Main
            Opcode::Jump(0x200 + 6),

            // Subroutine
            Opcode::LoadConstant { x: 0xA, value: 0xAA },
            Opcode::Return,

            // Main
            Opcode::LoadConstant { x: 0x1, value: 0xFF },
            Opcode::CallSubroutine(0x200 + 2),
            Opcode::LoadConstant { x: 0x2, value: 0xBB }
        ]));

        chip8.cycle_n(6);

        assert_eq!(chip8.v[0x1], 0xFF);
        assert_eq!(chip8.v[0xA], 0xAA);
        assert_eq!(chip8.v[0x2], 0xBB);
    }

    #[test]
    pub fn op_jump() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::Jump(0x200 + 4),
            Opcode::LoadConstant { x: 0x0, value: 0xAA },
            Opcode::LoadConstant { x: 0x1, value: 0xFF }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0x0);
        assert_eq!(chip8.v[0x1], 0xFF);
    }

    #[test]
    pub fn op_jump_with_offset() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x6 },
            Opcode::JumpWithOffset(0x200),
            Opcode::LoadConstant { x: 0x1, value: 0xAA },
            Opcode::LoadConstant { x: 0x2, value: 0xFF }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xFF);
    }

    #[test]
    pub fn op_skip_next_if_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfEqual { x: 0x0, value: 0x0 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    pub fn op_skip_next_if_equal_dont_skip_if_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfEqual { x: 0x0, value: 0xA }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    pub fn op_skip_next_if_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfNotEqual { x: 0x0, value: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    pub fn op_skip_next_if_register_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xA },
            Opcode::LoadConstant { x: 0x1, value: 0xA },
            Opcode::SkipNextIfRegisterEqual { x: 0x0, y: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle_n(3);
        assert_eq!(chip8.pc, 0x208);
    }

    #[test]
    pub fn op_skip_next_if_register_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xA },
            Opcode::LoadConstant { x: 0x1, value: 0xB },
            Opcode::SkipNextIfRegisterNotEqual { x: 0x0, y: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle_n(3);
        assert_eq!(chip8.pc, 0x208);
    }


    #[test]
    pub fn op_skip_if_key_pressed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xA },
            Opcode::SkipIfKeyPressed { x: 0x0 },
            Opcode::LoadConstant { x: 0x1, value: 0xA },
            Opcode::LoadConstant { x: 0x2, value: 0xB }
        ]));

        chip8.press_key(0xA);
        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xB);
    }

    #[test]
    pub fn op_skip_if_key_not_pressed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xA },
            Opcode::SkipIfKeyNotPressed { x: 0x0 },
            Opcode::LoadConstant { x: 0x1, value: 0xA },
            Opcode::LoadConstant { x: 0x2, value: 0xB }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xB);
    }

    #[test]
    pub fn op_wait_for_key_release() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::WaitForKeyRelease { x: 0xA },
            Opcode::LoadConstant { x: 0x1, value: 0xA }
        ]));

        chip8.press_key(0xA);
        chip8.cycle_n(10);
        assert_eq!(chip8.v[0x1], 0x0);

        chip8.press_key(0x3);
        chip8.release_key(0x3);
        chip8.cycle_n(1);
        assert_eq!(chip8.v[0x1], 0xA);
        assert_eq!(chip8.v[0xA], 0x3);
    }

    #[test]
    pub fn op_store_constant() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xF }
        ]));
        chip8.cycle();

        assert_eq!(chip8.v[0], 0x0F);
    }

    #[test]
    pub fn op_add_constant() {
        let mut chip8 = Chip8::new_with_rom(vec![0x71, 0x0F]);
        chip8.cycle();

        assert_eq!(chip8.v[1], 0x0F);
    }

    #[test]
    pub fn op_store() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 1, value: 0x15 },
            Opcode::Load { x: 2, y: 1 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[2], 0x15);
    }

    #[test]
    pub fn op_store_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0xFFF)
        ]));

        chip8.cycle();

        assert_eq!(chip8.i, 0xFFF);
    }

    #[test]
    pub fn op_add_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0x1),
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::AddAddress { x: 0x0 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.i, 0x2);
    }

    #[test]
    pub fn op_store_bcd_one_digit() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(address),
            Opcode::LoadConstant { x: 0, value: 3 },
            Opcode::WriteBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [0, 0, 3]);
    }

    #[test]
    pub fn op_store_bcd_two_digits() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(address),
            Opcode::LoadConstant { x: 0, value: 47 },
            Opcode::WriteBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [0, 4, 7]);
    }

    #[test]
    pub fn op_store_bcd_three_digits() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(address),
            Opcode::LoadConstant { x: 0, value: 255 },
            Opcode::WriteBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [2, 5, 5]);
    }

    #[test]
    pub fn op_store_sound() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoSound { x: 0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.sound_timer, 0x5);
    }

    #[test]
    pub fn op_store_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoDelay { x: 0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.delay_timer, 0x5);
    }

    #[test]
    pub fn op_read_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoDelay { x: 0 },
            Opcode::LoadDelayIntoRegister { x: 1 },
        ]));

        chip8.cycle_n(3);

        // Delay decreases by 1 per cycle so we expect our
        // original delay -1
        assert_eq!(chip8.v[0x1], 0x4);
    }

    #[test]
    pub fn op_random_can_be_deterministicly_seeded() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::Random { x: 0, mask: 0xFF },
            Opcode::Random { x: 1, mask: 0xFF }
        ]));
        chip8.with_seed(0);

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0], 0x6C);
        assert_eq!(chip8.v[1], 0x67);
    }

    #[test]
    pub fn op_random_masks_result() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::Random { x: 0, mask: 0x0F }
        ]));

        chip8.cycle();

        assert_eq!(chip8.v[0], chip8.v[0] & 0x0F);
    }

    #[test]
    pub fn op_or() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11110000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00001111 },
            Opcode::Or { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b11111111);
    }

    #[test]
    pub fn op_and() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11110000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00001111 },
            Opcode::And { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b00000000);
    }

    #[test]
    pub fn op_xor() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11111000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00011111 },
            Opcode::Xor { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b11100111);
    }

    #[test]
    pub fn op_add() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x2 },
            Opcode::Add { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x3);
    }

    #[test]
    pub fn op_add_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xFF },
            Opcode::LoadConstant { x: 0x1, value: 0xFF },
            Opcode::Add { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        println!("{:x?}", chip8.v[0x0]);
        assert_eq!(chip8.v[0x0], 0xFE);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_y_from_x() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x5 },
            Opcode::LoadConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractYFromX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x4);
    }

    #[test]
    pub fn op_subtract_y_from_x_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x0 },
            Opcode::LoadConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractYFromX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_x_from_y() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x5 },
            Opcode::SubtractXFromY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x4);
    }

    #[test]
    pub fn op_subtract_x_from_y_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x0 },
            Opcode::SubtractXFromY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_right() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000010 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000001);
    }

    #[test]
    pub fn op_shift_right_capture_msb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000001);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_left() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000110);
    }

    #[test]
    pub fn op_shift_left_capture_lsb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b10000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000110);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_clear_screen() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + (2 * 3)), // Store the address of the first byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
            Opcode::ClearScreen
        ]);
        rom.extend(vec![0b11110000]);

        let mut chip8 = Chip8::new_with_rom(rom);
        chip8.cycle_n(3);

        assert_eq!(chip8.gfx[0..8], [0,0,0,0,0,0,0,0]);
    }

    #[test]
    pub fn op_draw() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexFont { x: 0x0A },
            Opcode::LoadConstant { x: 0x0, value: 0 },
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x5 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.gfx_slice(0, 8, 0, 5), [
            [1,1,1,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
            [1,1,1,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
        ]);
    }

    #[test]
    pub fn op_draw_at_offset() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexFont { x: 0x0A },
            Opcode::LoadConstant { x: 0x0, value: 38 },
            Opcode::LoadConstant { x: 0x1, value: 20 },
            Opcode::Draw { x: 0x0, y: 0x1, n: 0x5 }
        ]));

        chip8.cycle_n(4);

        assert_eq!(chip8.gfx_slice(38, 46, 20, 25), [
            [1,1,1,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
            [1,1,1,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
            [1,0,0,1,0,0,0,0],
        ]);
    }

    #[test]
    pub fn op_draw_xors_overlapping_pixels() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + (2 * 5)), // Store the address of the first byte below
            Opcode::LoadConstant { x: 0x0, value: 0 },
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x1 },
            Opcode::IndexAddress(0x200 + (2 * 5) + 1), // Store the address of the second byte below
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x1 },
        ]);
        rom.extend(vec![0b11110000, 0b01101111]);

        let mut chip8 = Chip8::new_with_rom(rom);
        chip8.cycle_n(3);
        assert_eq!(chip8.gfx_slice(0, 8, 0, 1), [[1, 1, 1, 1, 0, 0, 0, 0]]);
        chip8.cycle_n(2);
        assert_eq!(chip8.gfx_slice(0, 8, 0, 1), [[1, 0, 0, 1, 1, 1, 1, 1]]);
    }

    /// When `draw` overlaps a sprite we expect it to delete the existing pixels and sets `VF` to `1`.
    ///
    /// This behavior is commonly used for collision detection
    #[test]
    pub fn op_draw_collision_detection() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + (2 * 5)), // Store the address of the first byte below
            Opcode::LoadConstant { x: 0x0, value: 0 },
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x1 },
            Opcode::IndexAddress(0x200 + (2 * 5) + 1), // Store the address of the second byte below
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x1 },
        ]);
        rom.extend(vec![0b11110000, 0b01101111]);

        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle_n(3);
        assert_eq!(chip8.v[0xF], 0);
        chip8.cycle_n(2);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    pub fn op_write_memory() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 100),
            Opcode::LoadConstant { x: 0x0, value: 0xFF },
            Opcode::LoadConstant { x: 0x1, value: 0xAA },
            Opcode::LoadConstant { x: 0x2, value: 0xBB },
            Opcode::WriteMemory { x: 0x2 }
        ]));

        chip8.cycle_n(5);

        assert_eq!(chip8.memory[0x200 + 100], 0xFF);
        assert_eq!(chip8.memory[0x200 + 101], 0xAA);
        assert_eq!(chip8.memory[0x200 + 102], 0xBB);
        assert_eq!(chip8.i, 0x200 + 102 + 1);
    }

    /// When using multiple `Opcode::WriteMemory`'s sequentually we expect it to start writing from
    /// where the previous write stopped.
    #[test]
    pub fn op_write_memory_consecutive() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 100),
            Opcode::LoadConstant { x: 0x0, value: 0xFF },
            Opcode::LoadConstant { x: 0x1, value: 0xAA },
            Opcode::WriteMemory { x: 0x1 },
            Opcode::LoadConstant { x: 0x0, value: 0x11 },
            Opcode::LoadConstant { x: 0x1, value: 0x21 },
            Opcode::WriteMemory { x: 0x1 }
        ]));

        chip8.cycle_n(7);

        assert_eq!(chip8.memory[0x200 + 100 .. 0x200 + 100 + 4], [0xFF, 0xAA, 0x11, 0x21]);
    }

    #[test]
    pub fn op_read_memory() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 4), // Store the address of the first byte below our opcodes
            Opcode::ReadMemory { x: 0x1 }
        ]);
        rom.extend(vec![0xAA, 0xFA]);

        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0xAA);
        assert_eq!(chip8.v[0x1], 0xFA);
        assert_eq!(chip8.i, 0x200 + 4 + 2)
    }


    /// When using multiple `Opcode::ReadMemory`'s sequentually we expect it to start reading from
    /// where the previous read stopped.
    #[test]
    pub fn op_read_memory_consecutive() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 6), // Store the address of the first byte below our opcodes
            Opcode::ReadMemory { x: 0x1 },
            Opcode::ReadMemory { x: 0x1 }
        ]);
        rom.extend(vec![0xAA, 0xFA, 0x01, 0x02]);

        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x01);
        assert_eq!(chip8.v[0x1], 0x02);
    }
}
