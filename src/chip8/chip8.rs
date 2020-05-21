use std::time::Duration;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::chip8::{Opcode, Register, Address, Chip8Result, Chip8Error};
use crate::chip8::quirks::{ReadWriteIncrementQuirk, BitShiftQuirk};
use crate::chip8::gpu::{self, Gpu};

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

    pub gpu: Gpu,

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

    /// `clock_speed` defines how often we `cycle` when calling `time`
    pub clock_speed: Duration,

    /// `timer_speed` defines how often we decrement `delay_timer` and `sound_timer`
    pub timer_speed: Duration,

    /// When `debug_mode` is true `tick` should do nothing. `step` needs to be used to advance the program.
    pub debug_mode: bool,

    read_write_increment_quirk: ReadWriteIncrementQuirk,

    bit_shift_quirk: BitShiftQuirk,

    /// Execution state, used to wait for keypresses
    state: Chip8State,

    /// Random Number Generator used for `Opcode::Random`
    rng: ChaCha8Rng,

    /// Stores how much time has elapsed since our last `cycle()`
    clock_tick_accumulator: Duration,

    /// Stores how much time has elapsed since we last decreased `delay_timer` and `sound_timer`
    timer_tick_accumulator: Duration,
}



#[derive(PartialEq)]
enum Chip8State {
    Running,
    WaitingForKey { target_register: Register }
}

#[derive(PartialEq)]
pub enum Chip8Output {
    None,
    Tick,
    Redraw
}

impl Chip8Output {
    fn combine(x: Chip8Output, y: Chip8Output) -> Chip8Output {
        match (x, y) {
            (Chip8Output::Redraw, _) => Chip8Output::Redraw,
            (_, Chip8Output::Redraw) => Chip8Output::Redraw,
            (Chip8Output::Tick, _) => Chip8Output::Tick,
            (_, Chip8Output::Tick) => Chip8Output::Tick,
            _ => Chip8Output::None,
        }
    }
}

impl Chip8 {
    pub const PROGRAM_START: u16 = 0x200;
    pub const MEMORY: u16 = 4096;

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
        chip8.pc = Chip8::PROGRAM_START;

        let font_start = Chip8::FONT_START as usize;
        let font_end = Chip8::FONT_END as usize;
        chip8.memory[font_start..font_end].copy_from_slice(&Chip8::FONTSET);

        chip8
    }

    pub fn new_with_rom(rom_bytes: Vec<u8>) -> Chip8 {
        let mut chip8 = Chip8::new();
        let rom_start = Chip8::PROGRAM_START as usize;
        let rom_end = rom_start + rom_bytes.len();
        chip8.memory[rom_start..rom_end].copy_from_slice(&rom_bytes[..]);
        chip8
    }

    pub fn new_with_default_rom() -> Chip8 {
        // Default ROM: Just loop forever
        let default_rom = Opcode::to_rom(vec![
            Opcode::Jump(Chip8::PROGRAM_START)
        ]);

        Chip8::new_with_rom(default_rom)
    }
    /// Returns a Chip8 with _no initialized memory_
    pub fn empty() -> Chip8 {
        Chip8 {
            memory: [0; Chip8::MEMORY as usize],
            stack: Vec::new(),
            gpu: Gpu::new(),
            keys: [false; 16],

            v: [0; 16],
            i: 0,
            pc: 0,

            delay_timer: 0,
            sound_timer: 0,

            clock_speed: Duration::from_secs_f64(1.0 / 500.0),
            timer_speed: Duration::from_secs_f64(1.0 / 60.0),

            debug_mode: false,
            read_write_increment_quirk: ReadWriteIncrementQuirk::default(),
            bit_shift_quirk: BitShiftQuirk::default(),

            state: Chip8State::Running,
            rng: ChaCha8Rng::from_entropy(),
            clock_tick_accumulator: Duration::new(0, 0),
            timer_tick_accumulator: Duration::new(0, 0),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self
    }

    pub fn with_read_write_increment_quirk(mut self, quirk: ReadWriteIncrementQuirk) -> Self {
        self.read_write_increment_quirk = quirk;
        self
    }

    pub fn with_bit_shift_quirk(mut self, quirk: BitShiftQuirk) -> Self {
        self.bit_shift_quirk = quirk;
        self
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

    /// Return (Address, Opcode) from the chip8 memory for all opcodes that fall
    /// within `start_addr..end_addr`
    pub fn opcodes(&self, start_addr: Address, end_addr: Address) -> Vec<(Address, Opcode)> {
        let start_addr = start_addr as usize;
        let end_addr = end_addr as usize;

        let mut result = Vec::new();
        for opcode_addr in (start_addr..end_addr).step_by(2) {
            let bytes = [self.memory[opcode_addr], self.memory[opcode_addr + 1]];

            if let Ok(opcode) = Opcode::from_bytes(&bytes) {
                result.push((opcode_addr as u16, opcode));
            }
        }

        result
    }

    /// Tick the CPU forward by `delta` time. Depending on how much time
    /// has elapsed this may:
    ///
    /// - `cycle` some number of times based on the clock speed
    /// - decrement `sound_timer`
    /// - decrement `delay_timer`
    pub fn tick(&mut self, delta: Duration) -> Chip8Result<Chip8Output> {
        if self.debug_mode {
            return Ok(Chip8Output::None)
        }

        self.tick_internal(delta)
    }

    /// Step the CPU forward by a fixed amount of time.
    pub fn step(&mut self) -> Chip8Result<Chip8Output> {
        self.tick_internal(self.clock_speed)
    }

    // Internal implementation of `tick` that ignores `debug_mode`
    fn tick_internal(&mut self, delta: Duration) -> Chip8Result<Chip8Output> {
        self.clock_tick_accumulator += delta;

        let mut output = Chip8Output::None;
        while self.clock_tick_accumulator >= self.clock_speed {
            self.clock_tick_accumulator -= self.clock_speed;
            self.timer_tick_accumulator += self.clock_speed;
            if self.timer_tick_accumulator > self.timer_speed {
                self.delay_timer = self.delay_timer.saturating_sub(1);
                self.sound_timer = self.sound_timer.saturating_sub(1);

                self.timer_tick_accumulator -= self.timer_speed;
            }

            let cycle_output = self.cycle()?;
            output = Chip8Output::combine(output, Chip8Output::Tick);
            output = Chip8Output::combine(output, cycle_output);
        }

        Ok(output)
    }


    /// Execute one cycle of the chip8 interpreter.
    pub fn cycle(&mut self) -> Chip8Result<Chip8Output> {
        if self.state != Chip8State::Running {
            return Ok(Chip8Output::None);
        }

        let opcode = self.read_opcode()?;
        self.pc += 2;

        self.execute_opcode(opcode.clone())?;

        match opcode {
            Opcode::Draw { x: _, y: _, n: _ } => Ok(Chip8Output::Redraw),
            _ => Ok(Chip8Output::None),
        }
    }

    pub fn cycle_n(&mut self, times: u32) -> Chip8Result<()> {
        for _ in 0..times {
            self.cycle()?;
        }

        Ok(())
    }

    fn read_opcode(&self) -> Chip8Result<Opcode> {
        let pc = self.pc as usize;
        let opcode_bytes = [self.memory[pc], self.memory[pc+1]];
        Opcode::from_bytes(&opcode_bytes)
    }

    fn execute_opcode(&mut self, opcode: Opcode) -> Chip8Result<()> {
        match opcode {
            // Flow Control
            Opcode::CallSubroutine(address) => self.op_call_subroutine(address),
            Opcode::Return => self.op_return()?,
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
            Opcode::SubtractXY { x, y } => self.op_subtract(x, x, y),
            Opcode::SubtractYX { x, y } => self.op_subtract(x, y, x),
            Opcode::ShiftRight { x, y } => self.op_shift_right(x, y),
            Opcode::ShiftLeft { x, y } => self.op_shift_left(x, y),

            // Manipulate `I`
            Opcode::IndexAddress(address) => self.i = address,
            Opcode::AddAddress { x } => self.i += self.v[x as usize] as u16,
            Opcode::IndexFont { x } => self.i = Chip8::FONT_START + (self.v[x as usize] as u16 * 5),

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
            Opcode::ClearScreen => self.gpu.clear(),
            Opcode::Draw { x, y, n } => self.op_draw(x, y, n),
        }

        Ok(())
    }

    fn op_call_subroutine(&mut self, address: Address) {
        self.stack.push(self.pc);
        self.pc = address;
    }

    fn op_return(&mut self) -> Chip8Result<()> {
        self.pc = self.stack.pop().ok_or(Chip8Error::StackUnderflow)?;

        Ok(())
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

    fn op_subtract(&mut self, target: Register, x: Register, y: Register) {
        let (result, carry) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[target as usize] = result;
        self.v[0xF] = !carry as u8;
    }

    fn op_shift_right(&mut self, x: Register, y: Register) {
        let source: &mut u8 = match self.bit_shift_quirk {
            BitShiftQuirk::ShiftYIntoX => &mut self.v[y as usize],
            BitShiftQuirk::ShiftX => &mut self.v[x as usize],
        };

        let least_significant_bit = *source & 0b00000001;
        self.v[x as usize] = source.wrapping_shr(1);
        self.v[0xF] = least_significant_bit;
    }

    fn op_shift_left(&mut self, x: Register, y: Register) {
        let source: &mut u8 = match self.bit_shift_quirk {
            BitShiftQuirk::ShiftYIntoX => &mut self.v[y as usize],
            BitShiftQuirk::ShiftX => &mut self.v[x as usize],
        };

        let most_significant_bit = (*source >> 7) & 1;
        self.v[x as usize] = source.wrapping_shl(1);
        self.v[0xF] = most_significant_bit;
    }

    fn op_draw(&mut self, x: Register, y: Register, n: u8) {

        let x = self.v[x as usize] as usize;
        let y = self.v[y as usize] as usize;
        let sprite: Vec<u8> = (0..n).map(|y| self.memory[(self.i + y as u16) as usize]).collect();

        match self.gpu.draw(x, y, sprite) {
            gpu::DrawResult::NoCollision => self.v[0xF] = 0,
            gpu::DrawResult::Collision => self.v[0xF] = 1
        }
    }

    fn op_write_memory(&mut self, x: Register) {
        for register in 0..=(x as usize) {
            self.memory[self.i as usize + register] = self.v[register];
        }

        if self.read_write_increment_quirk == ReadWriteIncrementQuirk::IncrementIndex {
            self.i += (x + 1) as u16;
        }
    }

    fn op_read_memory(&mut self, x: Register) {
        for register in 0..=(x as usize) {
            self.v[register] = self.memory[self.i as usize + register];
        }

        if self.read_write_increment_quirk == ReadWriteIncrementQuirk::IncrementIndex {
            self.i += (x + 1) as u16;
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
        chip8.cycle().unwrap();
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    pub fn tick_cycles_cpu_after_enough_time_has_passed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xF }
        ]));

        assert_eq!(chip8.v[0x0], 0x0);
        chip8.tick(chip8.clock_speed).unwrap();
        assert_eq!(chip8.v[0x0], 0xF);
    }

    #[test]
    pub fn tick_does_not_cycle_if_not_enough_time_has_passed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xF }
        ]));

        chip8.tick(Duration::new(0, 0)).unwrap();
        assert_eq!(chip8.v[0x0], 0x0);
    }

    #[test]
    pub fn tick_cycles_multiple_times_if_a_lot_of_time_has_passed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x05 },
            Opcode::LoadConstant { x: 0x1, value: 0xAA },
            Opcode::LoadConstant { x: 0x2, value: 0xBB },
        ]));

        chip8.tick(chip8.clock_speed * 3).unwrap();
        assert_eq!(chip8.v[0x0], 0x05);
        assert_eq!(chip8.v[0x1], 0xAA);
        assert_eq!(chip8.v[0x2], 0xBB);
    }

    #[test]
    pub fn tick_decreases_sound_timer_if_enough_time_has_passed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x8 },
            Opcode::LoadRegisterIntoSound { x: 0x0 },

            // Infinite Loop because decreasing sound takes many cycles
            Opcode::LoadConstant { x: 0x1, value: 0xFA },
            Opcode::Jump(Chip8::PROGRAM_START + 3 * 2)
        ]));

        chip8.tick(chip8.clock_speed * 2).unwrap();
        assert_eq!(chip8.sound_timer, 0x8);

        chip8.tick(chip8.timer_speed).unwrap();
        assert_eq!(chip8.sound_timer, 0x7);
    }

    #[test]
    pub fn tick_decreases_delay_timer_if_enough_time_has_passed() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x8 },
            Opcode::LoadRegisterIntoDelay { x: 0x0 },

            // Infinite Loop because decreasing sound takes many cycles
            Opcode::LoadConstant { x: 0x1, value: 0xFA },
            Opcode::Jump(Chip8::PROGRAM_START + 3 * 2)
        ]));

        chip8.tick(chip8.clock_speed * 2).unwrap();
        assert_eq!(chip8.delay_timer, 0x8);

        chip8.tick(chip8.timer_speed).unwrap();
        assert_eq!(chip8.delay_timer, 0x7);
    }

    /// When we call `tick` we may execute several cycles and decrease the timer several times.
    ///
    /// We need to ensure the operations are correctly interleaved.
    #[test]
    pub fn tick_interleaves_cycles_and_timers_correctly() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x2 },
            Opcode::LoadRegisterIntoDelay { x: 0x0 },
            Opcode::LoadDelayIntoRegister { x: 0x0 },
            Opcode::SkipNextIfEqual { x: 0x0, value: 0x0 },
            Opcode::Jump(Chip8::PROGRAM_START + 2 * 2),
            Opcode::LoadConstant { x: 0xA, value: 0xFF },
        ]));

        chip8.tick(chip8.clock_speed * 2 + chip8.timer_speed * 2 + chip8.clock_speed * 2).unwrap();
        assert_eq!(chip8.v[0xA], 0xFF);
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

        chip8.cycle_n(6).unwrap();

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

        chip8.cycle_n(2).unwrap();

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

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x1], 0x0);
        assert_eq!(chip8.v[0x2], 0xFF);
    }

    #[test]
    pub fn op_skip_next_if_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfEqual { x: 0x0, value: 0x0 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle().unwrap();
        assert_eq!(chip8.pc, 0x204);
    }

    #[test]
    pub fn op_skip_next_if_equal_dont_skip_if_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfEqual { x: 0x0, value: 0xA }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle().unwrap();
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    pub fn op_skip_next_if_not_equal() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::SkipNextIfNotEqual { x: 0x0, value: 0x1 }
        ]));

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle().unwrap();
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
        chip8.cycle_n(3).unwrap();
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
        chip8.cycle_n(3).unwrap();
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
        chip8.cycle_n(3).unwrap();

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

        chip8.cycle_n(3).unwrap();

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
        chip8.cycle_n(10).unwrap();
        assert_eq!(chip8.v[0x1], 0x0);

        chip8.press_key(0x3);
        chip8.release_key(0x3);
        chip8.cycle_n(1).unwrap();
        assert_eq!(chip8.v[0x1], 0xA);
        assert_eq!(chip8.v[0xA], 0x3);
    }

    #[test]
    pub fn op_store_constant() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xF }
        ]));
        chip8.cycle().unwrap();

        assert_eq!(chip8.v[0], 0x0F);
    }

    #[test]
    pub fn op_add_constant() {
        let mut chip8 = Chip8::new_with_rom(vec![0x71, 0x0F]);
        chip8.cycle().unwrap();

        assert_eq!(chip8.v[1], 0x0F);
    }

    #[test]
    pub fn op_store() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 1, value: 0x15 },
            Opcode::Load { x: 2, y: 1 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[2], 0x15);
    }

    #[test]
    pub fn op_store_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0xFFF)
        ]));

        chip8.cycle().unwrap();

        assert_eq!(chip8.i, 0xFFF);
    }

    #[test]
    pub fn op_add_address() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::IndexAddress(0x1),
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::AddAddress { x: 0x0 }
        ]));

        chip8.cycle_n(3).unwrap();

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

        chip8.cycle_n(3).unwrap();

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

        chip8.cycle_n(3).unwrap();

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

        chip8.cycle_n(3).unwrap();

        let address = address as usize;
        assert_eq!(chip8.memory[address..address+3], [2, 5, 5]);
    }

    #[test]
    pub fn op_store_sound() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoSound { x: 0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.sound_timer, 0x5);
    }

    #[test]
    pub fn op_store_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoDelay { x: 0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.delay_timer, 0x5);
    }

    #[test]
    pub fn op_read_delay() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0, value: 0x5 },
            Opcode::LoadRegisterIntoDelay { x: 0 },
            Opcode::LoadDelayIntoRegister { x: 1 },
        ]));

        chip8.cycle_n(3).unwrap();

        // Delay only decreases when we use `tick` so it should
        // be what we set
        assert_eq!(chip8.v[0x1], 0x5);
    }

    #[test]
    pub fn op_random_can_be_deterministicly_seeded() {
        let rom = Opcode::to_rom(vec![
            Opcode::Random { x: 0, mask: 0xFF },
            Opcode::Random { x: 1, mask: 0xFF }
        ]);

        let mut chip8 = Chip8::new_with_rom(rom)
            .with_seed(0);

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0], 0x6C);
        assert_eq!(chip8.v[1], 0x67);
    }

    #[test]
    pub fn op_random_masks_result() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::Random { x: 0, mask: 0x0F }
        ]));

        chip8.cycle().unwrap();

        assert_eq!(chip8.v[0], chip8.v[0] & 0x0F);
    }

    #[test]
    pub fn op_or() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11110000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00001111 },
            Opcode::Or { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0b11111111);
    }

    #[test]
    pub fn op_and() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11110000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00001111 },
            Opcode::And { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000000);
    }

    #[test]
    pub fn op_xor() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b11111000 },
            Opcode::LoadConstant { x: 0x1, value: 0b00011111 },
            Opcode::Xor { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0b11100111);
    }

    #[test]
    pub fn op_add() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x2 },
            Opcode::Add { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0x3);
    }

    #[test]
    pub fn op_add_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0xFF },
            Opcode::LoadConstant { x: 0x1, value: 0xFF },
            Opcode::Add { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        println!("{:x?}", chip8.v[0x0]);
        assert_eq!(chip8.v[0x0], 0xFE);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_x_y() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x5 },
            Opcode::LoadConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractXY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0x4);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_x_y_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x0 },
            Opcode::LoadConstant { x: 0x1, value: 0x1 },
            Opcode::SubtractXY { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    pub fn op_subtract_y_x() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x5 },
            Opcode::SubtractYX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0x4);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_subtract_y_x_overflow() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0x1 },
            Opcode::LoadConstant { x: 0x1, value: 0x0 },
            Opcode::SubtractYX { x: 0x0, y: 0x1 }
        ]));

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0xFF);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    pub fn op_shift_right() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000010 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000001);
    }

    #[test]
    pub fn op_shift_right_capture_msb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftRight { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000001);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_right_shift_y_into_x_quirk() {
        let rom = Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x1, value: 0b00000011 },
            Opcode::ShiftRight { x: 0x0, y: 0x1 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom)
            .with_bit_shift_quirk(BitShiftQuirk::ShiftYIntoX);

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000001);
        assert_eq!(chip8.v[0x1], 0b00000011);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_left() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b00000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000110);
    }

    #[test]
    pub fn op_shift_left_capture_lsb() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x0, value: 0b10000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x0 }
        ]));

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000110);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    pub fn op_shift_left_shift_y_into_x_quirk() {
        let rom = Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x1, value: 0b10000011 },
            Opcode::ShiftLeft { x: 0x0, y: 0x1 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom)
            .with_bit_shift_quirk(BitShiftQuirk::ShiftYIntoX);

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0b00000110);
        assert_eq!(chip8.v[0x1], 0b10000011);
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
        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.gpu.to_gfx_slice(0, 8, 0, 1), [[0,0,0,0,0,0,0,0]]);
    }

    #[test]
    pub fn op_draw() {
        let mut chip8 = Chip8::new_with_rom(Opcode::to_rom(vec![
            Opcode::LoadConstant { x: 0x1, value: 0xA },
            Opcode::IndexFont { x: 0x1 },
            Opcode::LoadConstant { x: 0x0, value: 0 },
            Opcode::Draw { x: 0x0, y: 0x0, n: 0x5 }
        ]));

        chip8.cycle_n(4).unwrap();

        assert_eq!(chip8.gpu.to_gfx_slice(0, 8, 0, 5), [
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
            Opcode::LoadConstant { x: 0x0, value: 0xA },
            Opcode::IndexFont { x: 0x0 },
            Opcode::LoadConstant { x: 0x0, value: 38 },
            Opcode::LoadConstant { x: 0x1, value: 20 },
            Opcode::Draw { x: 0x0, y: 0x1, n: 0x5 }
        ]));

        chip8.cycle_n(5).unwrap();

        assert_eq!(chip8.gpu.to_gfx_slice(38, 8, 20, 5), [
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
        chip8.cycle_n(3).unwrap();
        assert_eq!(chip8.gpu.to_gfx_slice(0, 8, 0, 1), [[1, 1, 1, 1, 0, 0, 0, 0]]);
        chip8.cycle_n(2).unwrap();
        assert_eq!(chip8.gpu.to_gfx_slice(0, 8, 0, 1), [[1, 0, 0, 1, 1, 1, 1, 1]]);
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

        chip8.cycle_n(3).unwrap();
        assert_eq!(chip8.v[0xF], 0);
        chip8.cycle_n(2).unwrap();
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

        chip8.cycle_n(5).unwrap();

        assert_eq!(chip8.memory[0x200 + 100], 0xFF);
        assert_eq!(chip8.memory[0x200 + 101], 0xAA);
        assert_eq!(chip8.memory[0x200 + 102], 0xBB);
        assert_eq!(chip8.i, 0x200 + 100);
    }

    /// When using multiple `Opcode::WriteMemory`'s sequentually we expect it to start writing from
    /// where the previous write stopped.
    #[test]
    pub fn op_write_memory_consecutive() {
        let rom = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 100),
            Opcode::LoadConstant { x: 0x0, value: 0xFF },
            Opcode::LoadConstant { x: 0x1, value: 0xAA },
            Opcode::WriteMemory { x: 0x1 },
            Opcode::LoadConstant { x: 0x0, value: 0x11 },
            Opcode::LoadConstant { x: 0x1, value: 0x21 },
            Opcode::WriteMemory { x: 0x1 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom)
            .with_read_write_increment_quirk(ReadWriteIncrementQuirk::IncrementIndex);

        chip8.cycle_n(7).unwrap();

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

        chip8.cycle_n(2).unwrap();

        assert_eq!(chip8.v[0x0], 0xAA);
        assert_eq!(chip8.v[0x1], 0xFA);
        assert_eq!(chip8.i, 0x200 + 4);
    }


    /// When using multiple `Opcode::ReadMemory`'s sequentually we expect it to start reading from
    /// where the previous read stopped.
    #[test]
    pub fn op_read_memory_consecutive_with_quirk() {
        let mut rom: Vec<u8> = Opcode::to_rom(vec![
            Opcode::IndexAddress(0x200 + 6), // Store the address of the first byte below our opcodes
            Opcode::ReadMemory { x: 0x1 },
            Opcode::ReadMemory { x: 0x1 }
        ]);
        rom.extend(vec![0xAA, 0xFA, 0x01, 0x02]);

        let mut chip8 = Chip8::new_with_rom(rom)
            .with_read_write_increment_quirk(ReadWriteIncrementQuirk::IncrementIndex);

        chip8.cycle_n(3).unwrap();

        assert_eq!(chip8.v[0x0], 0x01);
        assert_eq!(chip8.v[0x1], 0x02);
    }
}
