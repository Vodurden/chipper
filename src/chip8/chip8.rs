use std::fs;
use std::path::Path;

use crate::chip8::Opcode;

pub struct Chip8 {
    /// Chip-8 memory is segmented into two sections:
    ///
    /// - `0x000-0x1FF`: Reserved for the Chip 8 interpreter.
    /// - `0x200-0xFFF`: Program ROM and RAM
    ///
    /// We only use `0x050-0x0A0` in the reserved memory for the built in 4x5 pixel font set with digits (0-9) and letters (A-F)
    pub memory: [u8; 4096],

    /// Stack holds the addresses to return to when the current subroutine finishes.
    pub stack: [u16; 16],

    pub gfx: [u8; 64 * 32],

    pub key: [u8; 16],

    /// General Purpose Registers `V0`, `V1`, ..., `VF`
    ///
    /// `VF` should not be used by Chip-8 programs. We use it as a flag for some opcodes.
    pub v: [u8; 16],

    /// Index Register: Generally used to store memory addresses which means only the lowest (rightmost) 12 bits are usually used
    pub i: u16,

    /// Program Counter. Points to the currently executing address in `memory`
    pub pc: u16,

    /// Stack Pointer. Points to the topmost level of `stack`
    pub sp: u8,

    /// Delay Timer Register. When non-zero it decrements by 1 at the rate of 60hz.
    pub delay_timer: u8,

    /// Sound Timer Register. When non-zero it:
    ///
    /// - Decrements by 1 at a rate of 60hz
    /// - Sounds the Chip-8 buzzer.
    pub sound_timer: u8,
}

impl Chip8 {
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
        chip8.memory[0x50..0xA0].copy_from_slice(&Chip8::FONTSET);
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
            stack: [0; 16],
            gfx: [0; 64 * 32],
            key: [0; 16],

            v: [0; 16],
            i: 0,
            pc: 0,
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,
        }
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

    /// Execute one cycle of the chip8 interpreter.
    pub fn cycle(&mut self) {
        let opcode = self.read_opcode();
        self.pc += 2;

        println!("{:?}", opcode);

        self.execute_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn read_opcode(&self) -> Opcode {
        let pc = self.pc as usize;
        let opcode_bytes = [self.memory[pc], self.memory[pc+1]];
        Opcode::from_bytes(&opcode_bytes)
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::StoreConstant { x, value } => self.v[x as usize] = value,
            Opcode::AddConstant { x, value } => self.v[x as usize] += value,
            Opcode::Store { x, y } => self.v[x as usize] = self.v[y as usize],

            // TODO: Exhausive matching
            _ => panic!("Unsupported Opcode!"),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn program_counter_increases_after_cycle() {
        let mut chip8 = Chip8::new_with_rom(vec![0x60, 0x0F]);

        assert_eq!(chip8.pc, 0x200);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    pub fn store_constant() {
        let mut chip8 = Chip8::new_with_rom(vec![0x60, 0x0F]);
        chip8.cycle();

        assert_eq!(chip8.v[0], 0x0F);
    }

    #[test]
    pub fn add_constant() {
        let mut chip8 = Chip8::new_with_rom(vec![0x71, 0x0F]);
        chip8.cycle();

        assert_eq!(chip8.v[1], 0x0F);
    }

    #[test]
    pub fn store() {
        let rom = Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 1, value: 0x15 },
            Opcode::Store { x: 2, y: 1 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.v[2], 0x15);
    }
}
