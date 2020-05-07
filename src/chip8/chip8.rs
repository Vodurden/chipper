use std::fs;
use std::path::Path;

use crate::chip8::{Opcode, Register};

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

    pub gfx: [[u8; 64]; 32],

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
            stack: [0; 16],
            gfx: [[0; 64]; 32],
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
            Opcode::StoreAddress(address) => self.i = address,
            Opcode::Draw { x, y, n } => self.draw(x, y, n),

            Opcode::SetIndexToFontData { x } => self.i = Chip8::FONT_START + (x as u16 * 5),

            // TODO: Exhausive matching
            _ => panic!("Unsupported Opcode!"),
        }
    }

    fn draw(&mut self, x: Register, y: Register, n: u8) {
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
        let rom = Opcode::to_rom(vec![
            Opcode::StoreConstant { x: 1, value: 0x15 },
            Opcode::Store { x: 2, y: 1 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.v[2], 0x15);
    }

    #[test]
    pub fn op_draw() {
        let rom = Opcode::to_rom(vec![
            Opcode::SetIndexToFontData { x: 0x0A },
            Opcode::Draw { x: 0, y: 0, n: 0x5 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.gfx[0][0..8], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[1][0..8], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[2][0..8], [1,1,1,1,0,0,0,0]);
        assert_eq!(chip8.gfx[3][0..8], [1,0,0,1,0,0,0,0]);
        assert_eq!(chip8.gfx[4][0..8], [1,0,0,1,0,0,0,0]);
    }

    #[test]
    pub fn op_draw_at_offset() {
        let rom = Opcode::to_rom(vec![
            Opcode::SetIndexToFontData { x: 0x0A },
            Opcode::Draw { x: 10, y: 5, n: 0x5 }
        ]);
        let mut chip8 = Chip8::new_with_rom(rom);

        chip8.cycle();
        chip8.cycle();

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

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.gfx[0][0..8], [1, 1, 1, 1, 0, 0, 0, 0]);

        chip8.cycle();
        chip8.cycle();

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

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.v[0xF], 0);

        chip8.cycle();
        chip8.cycle();

        assert_eq!(chip8.v[0xF], 1);
    }
}
