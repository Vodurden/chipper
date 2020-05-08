use std::fs;
use std::path::Path;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::chip8::{Opcode, Register, Address};

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

    pub gfx: [[u8; 64]; 32],

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
    WaitingForKey { key: u8 }
}

impl Chip8 {
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
            gfx: [[0; 64]; 32],
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

    pub fn press_key(&mut self, key: u8) {
        self.keys[key as usize] = true;
    }

    pub fn release_key(&mut self, key: u8) {
        self.keys[key as usize] = false;

        if let Chip8State::WaitingForKey { key: wait_key } = self.state {
            if key == wait_key {
                self.state = Chip8State::Running;
            }
        }
    }

    /// Execute one cycle of the chip8 interpreter.
    pub fn cycle(&mut self) {
        if self.state != Chip8State::Running {
            return;
        }

        let opcode = self.read_opcode();
        self.pc += 2;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        self.execute_opcode(opcode);
    }

    pub fn cycle_n(&mut self, times: u32) {
        for _ in 0..times {
            self.cycle();
        }
    }

    fn read_opcode(&self) -> Opcode {
        let pc = self.pc as usize;
        let opcode_bytes = [self.memory[pc], self.memory[pc+1]];
        Opcode::from_bytes(&opcode_bytes)
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode {
            // Flow Control Opcodes - Opcodes for manipulating the program counter
            Opcode::CallSubroutine(address) => self.op_call_subroutine(address),
            Opcode::Return => self.op_return(),
            Opcode::Jump(address) => self.pc = address,
            Opcode::JumpWithOffset(address) => self.pc = address + (self.v[0] as u16),

            // Conditional Opcodes - Opcodes for conditionally executing parts of the program
            Opcode::SkipNextIfEqual { x, value } => self.op_skip_next_if(self.v[x as usize] == value),
            Opcode::SkipNextIfNotEqual { x, value } => self.op_skip_next_if(self.v[x as usize] != value),
            Opcode::SkipNextIfRegisterEqual { x, y } => self.op_skip_next_if(self.v[x as usize] == self.v[y as usize]),
            Opcode::SkipNextIfRegisterNotEqual { x, y } => self.op_skip_next_if(self.v[x as usize] != self.v[y as usize]),

            // Keypad Opcodes - Opcodes for capturing user input
            Opcode::SkipIfKeyPressed { key } => self.op_skip_next_if(self.keys[key as usize] == true),
            Opcode::SkipIfKeyNotPressed { key } => self.op_skip_next_if(self.keys[key as usize] == false),
            Opcode::WaitForKeyRelease { key } => self.state = Chip8State::WaitingForKey { key },

            // Register Opcodes - Opcodes to manipulate the value of the `V` registers
            Opcode::StoreConstant { x, value } => self.v[x as usize] = value,
            Opcode::AddConstant { x, value } => self.v[x as usize] += value,
            Opcode::Store { x, y } => self.v[x as usize] = self.v[y as usize],

            // Index Opcodes - Opcodes to manipulate the value of `I`
            Opcode::StoreAddress(address) => self.i = address,
            Opcode::AddAddress { x } => self.i += self.v[x as usize] as u16,
            Opcode::StoreBCD { x } => self.op_store_bcd(x),

            // Sound Opcodes - Opcodes for manipulating sound
            Opcode::StoreSound { x } => self.sound_timer = self.v[x as usize],

            // Time Opcodes
            Opcode::SetDelay { x } => self.delay_timer = self.v[x as usize],
            Opcode::ReadDelay { x } => self.v[x as usize] = self.delay_timer,

            // Random Opcode
            Opcode::Random { x, mask } => self.op_rand(x, mask),

            // Math Opcodes
            Opcode::Or { x, y } => self.v[x as usize] = self.v[x as usize] | self.v[y as usize],
            Opcode::And { x, y } => self.v[x as usize] = self.v[x as usize] & self.v[y as usize],
            Opcode::Xor { x, y } => self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize],
            Opcode::Add { x, y } => self.op_add(x, y),
            Opcode::SubtractYFromX { x, y } => self.op_subtract_y_from_x(x, y),
            Opcode::SubtractXFromY { x, y } => self.op_subtract_x_from_y(x, y),
            Opcode::ShiftRight { x, y } => self.op_shift_right(x, y),
            Opcode::ShiftLeft { x, y } => self.op_shift_left(x, y),

            // Drawing Opcodes - Opcodes to draw to the screen
            Opcode::ClearScreen => self.gfx = [[0; 64]; 32],
            Opcode::Draw { x, y, n } => self.op_draw(x, y, n),
            Opcode::SetIndexToFontData { x } => self.i = Chip8::FONT_START + (x as u16 * 5),

            // Memory Opcodes - Opcodes to read & write memory
            Opcode::WriteMemory { x } => self.op_write_memory(x),
            Opcode::ReadMemory { x } => self.op_read_memory(x),
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

        for pixel_y in 0..n {
            let row_sprite: u8 = self.memory[(self.i + pixel_y as u16) as usize];

            for pixel_x in 0..8 {
                let bit = (row_sprite >> (7 - pixel_x)) & 0x1;
                if bit != 0 {
                    let pixel: &mut u8 = &mut self.gfx[(y + pixel_y) as usize][(x + pixel_x) as usize];
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
            Opcode::StoreConstant { x: 0x0, value: 0xF }
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
            Opcode::StoreConstant { x: 0xA, value: 0xAA },
            Opcode::Return,

            // Main
            Opcode::StoreConstant { x: 0x1, value: 0xFF },
            Opcode::CallSubroutine(0x200 + 2),
            Opcode::StoreConstant { x: 0x2, value: 0xBB }
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
            Opcode::StoreConstant { x: 0x0, value: 0xAA },
            Opcode::StoreConstant { x: 0x1, value: 0xFF }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0x0);
        assert_eq!(chip8.v[0x1], 0xFF);
    }

    #[test]
    pub fn op_jump_with_offset() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0x6 },
            Opcode::JumpWithOffset(0x200),
            Opcode::StoreConstant { x: 0x1, value: 0xAA },
            Opcode::StoreConstant { x: 0x2, value: 0xFF }
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
            Opcode::StoreConstant { x: 0x0, value: 0xA },
            Opcode::StoreConstant { x: 0x1, value: 0xA },
            Opcode::SkipNextIfRegisterEqual { x: 0x0, y: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle_n(3);
        assert_eq!(chip8.pc, 0x208);
    }

    #[test]
    pub fn op_skip_next_if_register_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0xA },
            Opcode::StoreConstant { x: 0x1, value: 0xB },
            Opcode::SkipNextIfRegisterNotEqual { x: 0x0, y: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle_n(3);
        assert_eq!(chip8.pc, 0x208);
    }


    #[test]
    pub fn op_skip_if_key_pressed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipIfKeyPressed { key: 0xA },
            Opcode::StoreConstant { x: 0x1, value: 0xA },
            Opcode::StoreConstant { x: 0x2, value: 0xB }
        ]));

        chip8.press_key(0xA);
        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xB);
    }

    #[test]
    pub fn op_skip_if_key_not_pressed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipIfKeyNotPressed { key: 0xA },
            Opcode::StoreConstant { x: 0x1, value: 0xA },
            Opcode::StoreConstant { x: 0x2, value: 0xB }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xB);
    }

    #[test]
    pub fn op_wait_for_key_release() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::WaitForKeyRelease { key: 0xA },
            Opcode::StoreConstant { x: 0x1, value: 0xA }
        ]));

        chip8.press_key(0xA);
        chip8.cycle_n(10);
        assert_eq!(chip8.v[0x1], 0x0);

        chip8.release_key(0xA);
        chip8.cycle_n(1);
        assert_eq!(chip8.v[0x1], 0xA);
    }

    #[test]
    pub fn op_store_constant() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0xF }
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
            Opcode::StoreConstant { x: 1, value: 0x15 },
            Opcode::Store { x: 2, y: 1 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[2], 0x15);
    }

    #[test]
    pub fn op_store_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(0xFFF)
        ]));

        chip8.cycle();

        assert_eq!(chip8.i, 0xFFF);
    }

    #[test]
    pub fn op_add_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(0x1),
            Opcode::StoreConstant { x: 0x0, value: 0x1 },
            Opcode::AddAddress { x: 0x0 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.i, 0x2);
    }

    #[test]
    pub fn op_store_bcd_one_digit() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(address),
            Opcode::StoreConstant { x: 0, value: 3 },
            Opcode::StoreBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [0, 0, 3]);
    }

    #[test]
    pub fn op_store_bcd_two_digits() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(address),
            Opcode::StoreConstant { x: 0, value: 47 },
            Opcode::StoreBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [0, 4, 7]);
    }

    #[test]
    pub fn op_store_bcd_three_digits() {
        let address = 0x200 + 100;
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(address),
            Opcode::StoreConstant { x: 0, value: 255 },
            Opcode::StoreBCD { x: 0 },
        ]));

        chip8.cycle_n(3);

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [2, 5, 5]);
    }

    #[test]
    pub fn op_store_sound() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0, value: 0x5 },
            Opcode::StoreSound { x: 0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.sound_timer, 0x5);
    }

    #[test]
    pub fn op_store_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0, value: 0x5 },
            Opcode::SetDelay { x: 0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.delay_timer, 0x5);
    }

    #[test]
    pub fn op_read_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0, value: 0x5 },
            Opcode::SetDelay { x: 0 },
            Opcode::ReadDelay { x: 1 },
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
            Opcode::StoreConstant { x: 0x0, value: 0b11110000 },
            Opcode::StoreConstant { x: 0x1, value: 0b00001111 },
            Opcode::Or { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b11111111);
    }

    #[test]
    pub fn op_and() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b11110000 },
            Opcode::StoreConstant { x: 0x1, value: 0b00001111 },
            Opcode::And { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b00000000);
    }

    #[test]
    pub fn op_xor() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b11111000 },
            Opcode::StoreConstant { x: 0x1, value: 0b00011111 },
            Opcode::Xor { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0b11100111);
    }

    #[test]
    pub fn op_add() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0x1 },
            Opcode::StoreConstant { x: 0x1, value: 0x2 },
            Opcode::Add { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x3);
    }

    #[test]
    pub fn op_add_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0xFF },
            Opcode::StoreConstant { x: 0x1, value: 0xFF },
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
            Opcode::StoreConstant { x: 0x0, value: 0x5 },
            Opcode::StoreConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractYFromX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x4);
    }

    #[test]
    pub fn op_subtract_y_from_x_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0x0 },
            Opcode::StoreConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractYFromX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_x_from_y() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0x1 },
            Opcode::StoreConstant { x: 0x1, value: 0x5 },
            Opcode::SubtractXFromY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0x4);
    }

    #[test]
    pub fn op_subtract_x_from_y_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0x1 },
            Opcode::StoreConstant { x: 0x1, value: 0x0 },
            Opcode::SubtractXFromY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3);

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_right() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b00000010 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000001);
    }

    #[test]
    pub fn op_shift_right_capture_msb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000001);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_left() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000110);
    }

    #[test]
    pub fn op_shift_left_capture_lsb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 0x0, value: 0b10000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.v[0x0], 0b00000110);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_clear_screen() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::StoreAddress(0x200 + (2 * 3)), // Store the address of the first byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
            Opcode::ClearScreen
        ]);
        rom.extend(vec![0b11110000]);

        let mut chip8 = Chip8::new_with_rom(rom);
        chip8.cycle_n(3);

        assert_eq!(chip8.gfx[0][0..8], [0,0,0,0,0,0,0,0]);
    }

    #[test]
    pub fn op_draw() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SetIndexToFontData { x: 0x0A },
            Opcode::Draw { x: 0, y: 0, n: 0x5 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.gfx[0][0..8], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[1][0..8], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[2][0..8], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[3][0..8], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[4][0..8], [1,0,0,1,0,0,0,0]);
    }

    #[test]
    pub fn op_draw_at_offset() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SetIndexToFontData { x: 0x0A },
            Opcode::Draw { x: 10, y: 5, n: 0x5 }
        ]));

        chip8.cycle_n(2);

        assert_eq!(chip8.gfx[5][10..18], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[6][10..18], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[7][10..18], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[8][10..18], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[9][10..18], [1,0,0,1,0,0,0,0]);
    }

    #[test]
    pub fn op_draw_xors_overlapping_pixels() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::StoreAddress(0x200 + (2 * 4)), // Store the address of the first byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
            Opcode::StoreAddress(0x200 + (2 * 4) + 1), // Store the address of the second byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
        ]);
        rom.extend(vec![0b11110000, 0b01101111]);

        let mut chip8 = Chip8::new_with_rom(rom);
        chip8.cycle_n(2);
        assert_eq!(chip8.gfx[0][0..8], [1, 1, 1, 1, 0, 0, 0, 0]);
        chip8.cycle_n(2);
        assert_eq!(chip8.gfx[0][0..8], [1, 0, 0, 1, 1, 1, 1, 1]);
    }

    /// When `draw` overlaps a sprite we expect it to delete the existing pixels and sets `VF` to `1`.
    ///
    /// This behavior is commonly used for collision detection
    #[test]
    pub fn op_draw_collision_detection() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::StoreAddress(0x200 + (2 * 4)), // Store the address of the first byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
            Opcode::StoreAddress(0x200 + (2 * 4) + 1), // Store the address of the second byte below
            Opcode::Draw { x: 0, y: 0, n: 0x1 },
        ]);
        rom.extend(vec![0b11110000, 0b01101111]);

        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle_n(2);
        assert_eq!(chip8.v[0xF], 0);
        chip8.cycle_n(2);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    pub fn op_write_memory() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::StoreAddress(0x200 + 100),
            Opcode::StoreConstant { x: 0x0, value: 0xFF },
            Opcode::StoreConstant { x: 0x1, value: 0xAA },
            Opcode::StoreConstant { x: 0x2, value: 0xBB },
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
            Opcode::StoreAddress(0x200 + 100),
            Opcode::StoreConstant { x: 0x0, value: 0xFF },
            Opcode::StoreConstant { x: 0x1, value: 0xAA },
            Opcode::WriteMemory { x: 0x1 },
            Opcode::StoreConstant { x: 0x0, value: 0x11 },
            Opcode::StoreConstant { x: 0x1, value: 0x21 },
            Opcode::WriteMemory { x: 0x1 }
        ]));

        chip8.cycle_n(7);

        assert_eq!(chip8.memory[0x200 + 100 .. 0x200 + 100 + 4], [0xFF, 0xAA, 0x11, 0x21]);
    }

    #[test]
    pub fn op_read_memory() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::StoreAddress(0x200 + 4), // Store the address of the first byte below our opcodes
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
            Opcode::StoreAddress(0x200 + 6), // Store the address of the first byte below our opcodes
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
