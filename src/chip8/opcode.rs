use crate::chip8::{Register, Address};

#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Opcode: `00E0`
    ///
    /// Clear the display.
    ClearScreen,

    /// Opcode: `00EE`
    ///
    /// Return from a subroutine.
    Return,

    /// Opcode: `1nnn`
    ///
    /// Jump to address `nnn`.
    Jump(Address),

    /// Opcode: `2nnn`
    ///
    /// Call subroutine starting at address `nnn`.
    CallSubroutine(Address),

    /// Opcode: `3xnn`
    ///
    /// Skip the next instruction if register `Vx` equals `nn`.
    SkipNextIfEqual { x: Register, value: u8 },

    /// Opcode: `4xnn`
    ///
    /// Skip the next instruction if register `Vx` is _not_ equal to `nn`.
    SkipNextIfNotEqual { x: Register, value: u8 },

    /// Opcode: `5xy0`
    ///
    /// Skip the next instruction if register `Vx` is equal to register `Vy`.
    SkipNextIfRegisterEqual { x: Register, y: Register },

    /// Opcode: `6xnn`
    ///
    /// Store the value `nn` into register `Vx`.
    StoreConstant { x: Register, value: u8 },

    /// Opcode: `7xnn`
    ///
    /// Add the value nn into register `Vx`.
    AddConstant { x: Register, value: u8 },

    /// Opcode: `8xy0`
    ///
    /// Store the value of `Vy` in register `Vx`.
    Store { x: Register, y: Register },

    /// Opcode: `8xy1`
    ///
    /// Set `Vx` to `Vx OR Vy`.
    Or { x: Register, y: Register },

    /// Opcode: `8xy2`
    ///
    /// Set `Vx` to `Vx AND Vy`.
    And { x: Register, y: Register },

    /// Opcode: `8xy3`
    ///
    /// Set `Vx` to `Vx XOR Vy`.
    Xor { x: Register, y: Register },

    /// Opcode: `8xy4`
    ///
    /// - Set `Vx` to `Vx + Vy`.
    /// - Set `VF` to 01 if a carry occurs, otherwise set `VF` to 00.
    Add { x: Register, y: Register },

    /// Opcode: `8xy5`
    ///
    /// - Set `Vx` to `Vx - Vy`.
    /// - Set `VF` to 00 if a borrow occurs, otherwise set `VF` to 00.
    SubtractYFromX { x: Register, y: Register },

    /// Opcode: `8xy6`
    ///
    /// This opcode is controversial: For _old_ roms (pre-2005) they tend to assume this behavior:
    ///
    /// - Set `Vx` _and_ `Vy` to `Vy >> 1`.
    /// - Set `VF` to the least significant bit prior to the shift.
    ///
    /// For newer roms (post-2005) they tend to assume this behavior:
    ///
    /// - Set `Vx` to `Vx >> 1`
    /// - Set `VF` to the least significant bit prior to the shift
    ///
    /// Currently we implement the "new" behavior.
    ShiftRight { x: Register, y: Register },

    /// Opcode: `8xy7`
    ///
    /// - Set `Vx` to `Vy - Vx`.
    /// - Set `VF` to 00 if a borrow occurs, otherwise set `VF` to 00.
    SubtractXFromY { x: Register, y: Register },

    /// Opcode: `8xyE`
    ///
    /// This opcode is controversial: For _old_ roms (pre-2005) they tend to assume this behavior:
    ///
    /// - Set `Vx` _and_ `Vy` to `Vy << 1`.
    /// - Set `VF` to the most significant bit prior to the shift.
    ///
    /// For newer roms (post-2005) they tend to assume this behavior:
    ///
    /// - Set `Vx` to `Vx << 1`
    /// - Set `VF` to the most significant bit prior to the shift
    ///
    /// Currently we implement the "new" behavior.
    ShiftLeft { x: Register, y: Register },

    /// Opcode: `9xy0`
    ///
    /// Skip the next instruction if register `Vx` is not equal to register `Vy`.
    SkipNextIfRegisterNotEqual { x: Register, y: Register },

    /// Opcode: `Annn`
    ///
    /// Store address `nnn` in register `I`.
    StoreAddress(Address),

    /// Opcode: `Bnnn`
    ///
    /// Jump to address `nnn + V0`.
    JumpWithOffset(Address),

    /// Opcode: `Cxnn`
    ///
    /// Set `Vx` to a random number masked with `nn`.
    Random { x: Register, mask: u8 },

    /// Opcode: `Dxyn`
    ///
    /// - Draw a sprite at position `Vx`, `Vy`. The spirte has dimensions 8 by `n`
    /// - The sprite data is read starting from the address stored in `I`.
    /// - Set `VF` to 01 if any set pixels are changed to unset, otherwise set `VF` to 00
    ///
    /// When `Draw` is executed it also triggers a screen refresh
    Draw { x: Register, y: Register, n: u8 },

    /// Opcode: `Ex9E`
    ///
    /// Skip the next instruction if the key corresponding to the value of `Vx` is pressed.
    SkipIfKeyPressed { x: Register },

    /// Opcode: `ExA1`
    ///
    /// Skip the next instruction if the key corresponding to the value of `Vx` is not pressed.
    SkipIfKeyNotPressed { x: Register },

    /// Opcode: `Fx07`
    ///
    /// Read the value of the delay timer into `Vx`.
    ReadDelay { x: Register },

    /// Opcode: `Fx0A`
    ///
    /// Wait for a keypress and store the result in `Vx`.
    WaitForKeyPress { x: Register },

    /// Opcode: `Fx15`
    ///
    /// Set the delay timer to the value of `Vx`.
    SetDelay { x: Register },

    /// Opcode: `Fx18`
    ///
    /// Store the value of `Vx` in the sound timer.
    StoreSound { x: Register },

    /// Opcode: `Fx1E`
    ///
    /// Add the value of `Vx` to `I`
    AddIndex { x: Register },

    /// Opcode: `Fx29`
    ///
    /// Set `I` to the font data corresponding to the value of `Vx`.
    SetIndexToFontData { x: Register },

    /// Opcode: `Fx33`
    ///
    /// Store the binary-coded decimal equivalent of the value stored in `Vx` at addresses `I`, `I+1` and `I+2`.
    StoreBCD { x: Register },

    /// Opcode: `Fx55`
    ///
    /// - Store the values of `V0..Vx` (inclusive) in memory starting at address `I`.
    /// - Set `I` to `I + x + 1`.
    WriteMemory { x: Register },

    /// Opcode: `Fx65`
    ///
    /// - Fill registers `V0..Vx` (inclusive) with the values stored in memory starting at address `I`.
    /// - Set `I` to `I + x + 1`.
    ReadMemory { x: Register },
}

impl Opcode {
    pub fn from_bytes(bytes: &[u8; 2]) -> Opcode {
        let opcode = u16::from_be_bytes(*bytes);
        Opcode::from_u16(opcode)
    }

    /// Return the byte representation of this opcode.
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> [u8; 2] {
        self.to_u16().to_be_bytes()
    }

    #[allow(dead_code)]
    pub fn to_rom(opcodes: Vec<Opcode>) -> Vec<u8> {
        opcodes.iter()
            .flat_map(|op| op.to_bytes().to_vec())
            .collect()
    }

    pub fn from_u16(word: u16) -> Opcode {
        let nibbles = (
            ((word & 0xF000) >> 12) as u8,
            ((word & 0x0F00) >> 8) as u8,
            ((word & 0x00F0) >> 4) as u8,
            (word & 0x000F) as u8
        );

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => Opcode::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => Opcode::Return,

            (0x1, _, _, _) => Opcode::Jump(word & 0x0FFF),
            (0x2, _, _, _) => Opcode::CallSubroutine(word & 0x0FFF),
            (0x3, x, _, _) => Opcode::SkipNextIfEqual { x, value: (word & 0x00FF) as u8 },
            (0x4, x, _, _) => Opcode::SkipNextIfNotEqual { x, value: (word & 0x00FF) as u8 },
            (0x5, x, y, 0x0) => Opcode::SkipNextIfRegisterEqual { x, y },
            (0x6, x, _, _) => Opcode::StoreConstant { x, value: (word & 0x00FF) as u8 },
            (0x7, x, _, _) => Opcode::AddConstant { x, value: (word & 0x00FF) as u8 },

            (0x8, x, y, 0x0) => Opcode::Store { x, y },
            (0x8, x, y, 0x1) => Opcode::Or { x, y },
            (0x8, x, y, 0x2) => Opcode::And { x, y },
            (0x8, x, y, 0x3) => Opcode::Xor { x, y },
            (0x8, x, y, 0x4) => Opcode::Add { x, y },
            (0x8, x, y, 0x5) => Opcode::SubtractXFromY { x, y },
            (0x8, x, y, 0x6) => Opcode::ShiftRight { x, y },
            (0x8, x, y, 0x7) => Opcode::SubtractYFromX { x, y },
            (0x8, x, y, 0xE) => Opcode::ShiftLeft { x, y },

            (0x9, x, y, 0x0) => Opcode::SkipNextIfRegisterNotEqual { x, y },

            (0xA, _, _, _) => Opcode::StoreAddress(word & 0x0FFF),
            (0xB, _, _, _) => Opcode::JumpWithOffset(word & 0x0FFF),
            (0xC, x, _, _) => Opcode::Random { x, mask: (word & 0x00FF) as u8 },
            (0xD, x, y, n) => Opcode::Draw { x, y, n },

            (0xE, x, 0x9, 0xE) => Opcode::SkipIfKeyPressed { x },
            (0xE, x, 0xA, 0x1) => Opcode::SkipIfKeyNotPressed { x },

            (0xF, x, 0x0, 0x7) => Opcode::ReadDelay { x },
            (0xF, x, 0x0, 0xA) => Opcode::WaitForKeyPress { x },
            (0xF, x, 0x1, 0x5) => Opcode::SetDelay { x },
            (0xF, x, 0x1, 0x8) => Opcode::StoreSound { x },
            (0xF, x, 0x1, 0xE) => Opcode::AddIndex { x },
            (0xF, x, 0x2, 0x9) => Opcode::SetIndexToFontData { x },
            (0xF, x, 0x3, 0x3) => Opcode::StoreBCD { x },
            (0xF, x, 0x5, 0x5) => Opcode::WriteMemory { x },
            (0xF, x, 0x6, 0x5) => Opcode::ReadMemory { x },

            // TODO: Graceful error handling
            _ => panic!("Unknown opcode: {:x}", word)
        }
    }

    #[allow(dead_code)]
    pub fn to_u16(&self) -> u16 {
        match self {
            Opcode::ClearScreen => 0x00E0,
            Opcode::Return => 0x00EE,

            Opcode::Jump(address) => 0x1000 | address,
            Opcode::CallSubroutine(address) => 0x2000 | address,
            Opcode::SkipNextIfEqual { x, value } => 0x3000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::SkipNextIfNotEqual { x, value } => 0x4000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::SkipNextIfRegisterEqual { x, y } => 0x5000 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::StoreConstant { x, value } => 0x6000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::AddConstant { x, value } => 0x7000 | ((*x as u16) << 8) | (*value as u16),

            Opcode::Store { x, y } => 0x8000 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Or { x, y } => 0x8001 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::And { x, y } => 0x8002 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Xor { x, y } => 0x8003 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Add { x, y } => 0x8004 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::SubtractXFromY { x, y } => 0x8005 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::ShiftRight { x, y } => 0x8006 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::SubtractYFromX { x, y } => 0x8007 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::ShiftLeft { x, y } => 0x800E | ((*x as u16) << 8) | ((*y as u16) << 4),

            Opcode::SkipNextIfRegisterNotEqual { x, y } => 0x9000 | ((*x as u16) << 8) | ((*y as u16) << 4),

            Opcode::StoreAddress(address) => 0xA000 | address,
            Opcode::JumpWithOffset(address) => 0xB000 | address,
            Opcode::Random { x, mask } => 0xC000 | ((*x as u16) << 8) | (*mask as u16),
            Opcode::Draw { x, y, n } => 0xD000 | ((*x as u16) << 8) | ((*y as u16) << 4) | (*n as u16),

            Opcode::SkipIfKeyPressed { x } => 0xE09E | ((*x as u16) << 8),
            Opcode::SkipIfKeyNotPressed { x } => 0xE0A1 | ((*x as u16) << 8),

            Opcode::ReadDelay { x } => 0xF007 | ((*x as u16) << 8),
            Opcode::WaitForKeyPress { x } => 0xF00A | ((*x as u16) << 8),
            Opcode::SetDelay { x } => 0x0F015 | ((*x as u16) << 8),
            Opcode::StoreSound { x } => 0xF018 | ((*x as u16) << 8),
            Opcode::AddIndex { x } => 0xF01E | ((*x as u16) << 8),
            Opcode::SetIndexToFontData { x } => 0xF029 | ((*x as u16) << 8),
            Opcode::StoreBCD { x } => 0xF033 | ((*x as u16) << 8),
            Opcode::WriteMemory { x } => 0xF055 | ((*x as u16) << 8),
            Opcode::ReadMemory { x } => 0xF065 | ((*x as u16) << 8),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bytes() {
        assert_eq!(Opcode::Jump(0xABC).to_bytes(), [0x1A, 0xBC])
    }

    #[test]
    fn from_bytes() {
        assert_eq!(Opcode::from_bytes(&[0x00, 0xE0]), Opcode::from_u16(0x00E0));
    }

    #[test]
    fn to_rom() {
        let rom = Opcode::to_rom(vec![
            Opcode::ClearScreen,
            Opcode::Add { x: 0xA, y: 0xB }
        ]);

        assert_eq!(rom, [0x00, 0xE0, 0x8A, 0xB4])
    }

    // ======================
    // = Tests for to_u16 =
    // ======================
    #[test]
    fn to_u16_clear_screen() {
        assert_eq!(Opcode::ClearScreen.to_u16(), 0x00E0);
    }

    #[test]
    fn to_u16_return() {
        assert_eq!(Opcode::Return.to_u16(), 0x00EE);
    }

    #[test]
    fn to_u16_jump() {
        assert_eq!(Opcode::Jump(0xABC).to_u16(), 0x1ABC);
    }

    #[test]
    fn to_u16_call_subroutine() {
        assert_eq!(Opcode::CallSubroutine(0xABC).to_u16(), 0x2ABC);
    }

    #[test]
    fn to_u16_skip_next_if_equal() {
        assert_eq!(Opcode::SkipNextIfEqual { x: 0xA, value: 0x15 }.to_u16(), 0x3A15);
    }

    #[test]
    fn to_u16_skip_next_if_not_equal() {
        assert_eq!(Opcode::SkipNextIfNotEqual { x: 0xA, value: 0x15 }.to_u16(), 0x4A15);
    }

    #[test]
    fn to_u16_skip_next_if_register_equal() {
        assert_eq!(Opcode::SkipNextIfRegisterEqual { x: 0xA, y: 0xB }.to_u16(), 0x5AB0);
    }

    #[test]
    fn to_u16_store_constant() {
        assert_eq!(Opcode::StoreConstant { x: 0xA, value: 0x10 }.to_u16(), 0x6A10);
    }

    #[test]
    fn to_u16_add_constant() {
        assert_eq!(Opcode::AddConstant { x: 0xA, value: 0x10 }.to_u16(), 0x7A10);
    }

    #[test]
    fn to_u16_store() {
        assert_eq!(Opcode::Store { x: 0xA, y: 0xB }.to_u16(), 0x8AB0);
    }

    #[test]
    fn to_u16_or() {
        assert_eq!(Opcode::Or { x: 0xA, y: 0xB }.to_u16(), 0x8AB1);
    }

    #[test]
    fn to_u16_and() {
        assert_eq!(Opcode::And { x: 0xA, y: 0xB }.to_u16(), 0x8AB2);
    }

    #[test]
    fn to_u16_xor() {
        assert_eq!(Opcode::Xor { x: 0xA, y: 0xB }.to_u16(), 0x8AB3);
    }

    #[test]
    fn to_u16_add() {
        assert_eq!(Opcode::Add { x: 0xA, y: 0xB }.to_u16(), 0x8AB4);
    }

    #[test]
    fn to_u16_subtract_x_from_y() {
        assert_eq!(Opcode::SubtractXFromY { x: 0xA, y: 0xB }.to_u16(), 0x8AB5);
    }

    #[test]
    fn to_u16_shift_right() {
        assert_eq!(Opcode::ShiftRight { x: 0xA, y: 0xB }.to_u16(), 0x8AB6);
    }

    #[test]
    fn to_u16_subtract_y_from_x() {
        assert_eq!(Opcode::SubtractYFromX { x: 0xA, y: 0xB }.to_u16(), 0x8AB7);
    }

    #[test]
    fn to_u16_shift_left() {
        assert_eq!(Opcode::ShiftLeft { x: 0xA, y: 0xB }.to_u16(), 0x8ABE);
    }

    #[test]
    fn to_u16_skip_next_if_register_not_equal() {
        assert_eq!(Opcode::SkipNextIfRegisterNotEqual { x: 0xA, y: 0xB }.to_u16(), 0x9AB0);
    }

    #[test]
    fn to_u16_store_address() {
        assert_eq!(Opcode::StoreAddress(0xABC).to_u16(), 0xAABC);
    }

    #[test]
    fn to_u16_jump_with_offset() {
        assert_eq!(Opcode::JumpWithOffset(0xABC).to_u16(), 0xBABC);
    }

    #[test]
    fn to_u16_random() {
        assert_eq!(Opcode::Random { x: 0x1, mask: 0x52 }.to_u16(), 0xC152);
    }

    #[test]
    fn to_u16_draw() {
        assert_eq!(Opcode::Draw { x: 0xA, y: 0xB, n: 0x1 }.to_u16(), 0xDAB1);
    }

    #[test]
    fn to_u16_skip_if_key_pressed() {
        assert_eq!(Opcode::SkipIfKeyPressed { x: 0xA }.to_u16(), 0xEA9E);
    }

    #[test]
    fn to_u16_skip_if_key_not_pressed() {
        assert_eq!(Opcode::SkipIfKeyNotPressed { x: 0xA }.to_u16(), 0xEAA1);
    }

    #[test]
    fn to_u16_store_delay() {
        assert_eq!(Opcode::ReadDelay { x: 0xA }.to_u16(), 0xFA07);
    }

    #[test]
    fn to_u16_wait_for_keypress() {
        assert_eq!(Opcode::WaitForKeyPress { x: 0xA }.to_u16(), 0xFA0A);
    }

    #[test]
    fn to_u16_set_delay() {
        assert_eq!(Opcode::SetDelay { x: 0xA }.to_u16(), 0xFA15);
    }

    #[test]
    fn to_u16_store_sound() {
        assert_eq!(Opcode::StoreSound { x: 0xA }.to_u16(), 0xFA18);
    }

    #[test]
    fn to_u16_add_index() {
        assert_eq!(Opcode::AddIndex { x: 0xA }.to_u16(), 0xFA1E);
    }

    #[test]
    fn to_u16_set_index_to_font_data() {
        assert_eq!(Opcode::SetIndexToFontData { x: 0xA }.to_u16(), 0xFA29);
    }

    #[test]
    fn to_u16_store_bcd() {
        assert_eq!(Opcode::StoreBCD { x: 0xA }.to_u16(), 0xFA33);
    }

    #[test]
    fn to_u16_write_memory() {
        assert_eq!(Opcode::WriteMemory { x: 0xA }.to_u16(), 0xFA55);
    }

    #[test]
    fn to_u16_read_memory() {
        assert_eq!(Opcode::ReadMemory { x: 0xA }.to_u16(), 0xFA65);
    }

    // ======================
    // = Tests for from_u16 =
    // ======================
    #[test]
    fn from_u16_clear_screen() {
        assert_eq!(Opcode::from_u16(0x00E0), Opcode::ClearScreen);
    }

    #[test]
    fn from_u16_return() {
        assert_eq!(Opcode::from_u16(0x00EE), Opcode::Return);
    }

    #[test]
    fn from_u16_jump() {
        assert_eq!(Opcode::from_u16(0x1ABC), Opcode::Jump(0xABC));
    }

    #[test]
    fn from_u16_call_subroutine() {
        assert_eq!(Opcode::from_u16(0x2ABC), Opcode::CallSubroutine(0xABC));
    }

    #[test]
    fn from_u16_skip_next_if_equal() {
        assert_eq!(Opcode::from_u16(0x3A15), Opcode::SkipNextIfEqual { x: 0xA, value: 0x15 });
    }

    #[test]
    fn from_u16_skip_next_if_not_equal() {
        assert_eq!(Opcode::from_u16(0x4A15), Opcode::SkipNextIfNotEqual { x: 0xA, value: 0x15 });
    }

    #[test]
    fn from_u16_skip_next_if_register_equal() {
        assert_eq!(Opcode::from_u16(0x5AB0), Opcode::SkipNextIfRegisterEqual { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_store_constant() {
        assert_eq!(Opcode::from_u16(0x6A10), Opcode::StoreConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn from_u16_add_constant() {
        assert_eq!(Opcode::from_u16(0x7A10), Opcode::AddConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn from_u16_store() {
        assert_eq!(Opcode::from_u16(0x8AB0), Opcode::Store { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_or() {
        assert_eq!(Opcode::from_u16(0x8AB1), Opcode::Or { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_and() {
        assert_eq!(Opcode::from_u16(0x8AB2), Opcode::And { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_xor() {
        assert_eq!(Opcode::from_u16(0x8AB3), Opcode::Xor { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_add() {
        assert_eq!(Opcode::from_u16(0x8AB4), Opcode::Add { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_subtract_x_from_y() {
        assert_eq!(Opcode::from_u16(0x8AB5), Opcode::SubtractXFromY { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_shift_right() {
        assert_eq!(Opcode::from_u16(0x8AB6), Opcode::ShiftRight { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_subtract_y_from_x() {
        assert_eq!(Opcode::from_u16(0x8AB7), Opcode::SubtractYFromX { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_shift_left() {
        assert_eq!(Opcode::from_u16(0x8ABE), Opcode::ShiftLeft { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_skip_next_if_register_not_equal() {
        assert_eq!(Opcode::from_u16(0x9AB0), Opcode::SkipNextIfRegisterNotEqual { x: 0xA, y: 0xB });
    }

    #[test]
    fn from_u16_store_address() {
        assert_eq!(Opcode::from_u16(0xAABC), Opcode::StoreAddress(0xABC));
    }

    #[test]
    fn from_u16_jump_with_offset() {
        assert_eq!(Opcode::from_u16(0xBABC), Opcode::JumpWithOffset(0xABC));
    }

    #[test]
    fn from_u16_random() {
        assert_eq!(Opcode::from_u16(0xC152), Opcode::Random { x: 0x1, mask: 0x52 });
    }

    #[test]
    fn from_u16_draw() {
        assert_eq!(Opcode::from_u16(0xDAB1), Opcode::Draw { x: 0xA, y: 0xB, n: 0x1 });
    }

    #[test]
    fn from_u16_skip_if_key_pressed() {
        assert_eq!(Opcode::from_u16(0xEA9E), Opcode::SkipIfKeyPressed { x: 0xA });
    }

    #[test]
    fn from_u16_skip_if_key_not_pressed() {
        assert_eq!(Opcode::from_u16(0xEAA1), Opcode::SkipIfKeyNotPressed { x: 0xA });
    }

    #[test]
    fn from_u16_store_delay() {
        assert_eq!(Opcode::from_u16(0xFA07), Opcode::ReadDelay { x: 0xA });
    }

    #[test]
    fn from_u16_wait_for_keypress() {
        assert_eq!(Opcode::from_u16(0xFA0A), Opcode::WaitForKeyPress { x: 0xA });
    }

    #[test]
    fn from_u16_set_delay() {
        assert_eq!(Opcode::from_u16(0xFA15), Opcode::SetDelay { x: 0xA });
    }

    #[test]
    fn from_u16_store_sound() {
        assert_eq!(Opcode::from_u16(0xFA18), Opcode::StoreSound { x: 0xA });
    }

    #[test]
    fn from_u16_add_index() {
        assert_eq!(Opcode::from_u16(0xFA1E), Opcode::AddIndex { x: 0xA });
    }

    #[test]
    fn from_u16_set_index_to_font_data() {
        assert_eq!(Opcode::from_u16(0xFA29), Opcode::SetIndexToFontData { x: 0xA });
    }

    #[test]
    fn from_u16_store_bcd() {
        assert_eq!(Opcode::from_u16(0xFA33), Opcode::StoreBCD { x: 0xA });
    }

    #[test]
    fn from_u16_write_memory() {
        assert_eq!(Opcode::from_u16(0xFA55), Opcode::WriteMemory { x: 0xA });
    }

    #[test]
    fn from_u16_read_memory() {
        assert_eq!(Opcode::from_u16(0xFA65), Opcode::ReadMemory { x: 0xA });
    }
}
