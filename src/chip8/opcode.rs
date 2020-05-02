use crate::chip8::{Register, Address};

#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    /// Opcode: `0030`
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
    SubtractXFromY { x: Register, y: Register },

    /// Opcode: `8xy6`
    ///
    /// - Set `Vx` to `Vy >> 1`.
    /// - Set `VF` to the least significant bit prior to the shift.
    ShiftRight { x: Register, y: Register },

    /// Opcode: `8xy7`
    ///
    /// - Set `Vx` to `Vy - Vx`.
    /// - Set `VF` to 00 if a borrow occurs, otherwise set `VF` to 00.
    SubtractYFromX { x: Register, y: Register },

    /// Opcode: `8xyE`
    ///
    /// - Set `Vx` to `Vy << 1`.
    /// - Set `VF` to the most significant bit prior to the shift.
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
    /// - Draw a sprite at position `Vx`, `Vy` with `n` bytes of sprite data starting from the address stored in `I`
    /// - Set `VF` to 01 if any set pixels are changed to unset, otherwise set `VF` to 00
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
    /// Store the value of `Vx` in the delay timer.
    StoreDelay { x: Register },

    /// Opcode: `Fx0A`
    ///
    /// Wait for a keypress and store the result in `Vx`.
    WaitForKeyPress { x: Register },

    /// Opcode: `Fx15`
    ///
    /// Set the delay timer to the value of `Vx.`
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
            (0x8, x, y, 0x5) => Opcode::SubtractXFromY { x, y},
            (0x8, x, y, 0x6) => Opcode::ShiftRight { x, y},
            (0x8, x, y, 0x7) => Opcode::SubtractYFromX { x, y},
            (0x8, x, y, 0xE) => Opcode::ShiftLeft { x, y},

            (0x9, x, y, 0x0) => Opcode::SkipNextIfRegisterNotEqual { x, y },

            (0xA, _, _, _) => Opcode::StoreAddress(word & 0x0FFF),
            (0xB, _, _, _) => Opcode::JumpWithOffset(word & 0x0FFF),
            (0xC, x, _, _) => Opcode::Random { x, mask: (word & 0x00FF) as u8 },
            (0xD, x, y, n) => Opcode::Draw { x, y, n },

            (0xE, x, 0x9, 0xE) => Opcode::SkipIfKeyPressed { x },
            (0xE, x, 0xA, 0x1) => Opcode::SkipIfKeyNotPressed { x },

            (0xF, x, 0x0, 0x7) => Opcode::StoreDelay { x },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes() {
        assert_eq!(Opcode::from_bytes(&[0x00, 0xE0]), Opcode::from_u16(0x00E0));
    }

    #[test]
    fn parse_clear_screen() {
        assert_eq!(Opcode::from_u16(0x00E0), Opcode::ClearScreen);
    }

    #[test]
    fn parse_return() {
        assert_eq!(Opcode::from_u16(0x00EE), Opcode::Return);
    }

    #[test]
    fn parse_jump() {
        assert_eq!(Opcode::from_u16(0x1ABC), Opcode::Jump(0xABC));
    }

    #[test]
    fn parse_call_subroutine() {
        assert_eq!(Opcode::from_u16(0x2ABC), Opcode::CallSubroutine(0xABC));
    }

    #[test]
    fn parse_skip_next_if_equal() {
        assert_eq!(Opcode::from_u16(0x3A15), Opcode::SkipNextIfEqual { x: 0xA, value: 0x15 });
    }

    #[test]
    fn parse_skip_next_if_not_equal() {
        assert_eq!(Opcode::from_u16(0x4A15), Opcode::SkipNextIfNotEqual { x: 0xA, value: 0x15 });
    }

    #[test]
    fn parse_skip_next_if_register_equal() {
        assert_eq!(Opcode::from_u16(0x5AB0), Opcode::SkipNextIfRegisterEqual { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_store_constant() {
        assert_eq!(Opcode::from_u16(0x6A10), Opcode::StoreConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn parse_add_constant() {
        assert_eq!(Opcode::from_u16(0x7A10), Opcode::AddConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn parse_store() {
        assert_eq!(Opcode::from_u16(0x8AB0), Opcode::Store { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_or() {
        assert_eq!(Opcode::from_u16(0x8AB1), Opcode::Or { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_and() {
        assert_eq!(Opcode::from_u16(0x8AB2), Opcode::And { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_xor() {
        assert_eq!(Opcode::from_u16(0x8AB3), Opcode::Xor { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_add() {
        assert_eq!(Opcode::from_u16(0x8AB4), Opcode::Add { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_subtract_x_from_y() {
        assert_eq!(Opcode::from_u16(0x8AB5), Opcode::SubtractXFromY { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_shift_right() {
        assert_eq!(Opcode::from_u16(0x8AB6), Opcode::ShiftRight { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_subtract_y_from_x() {
        assert_eq!(Opcode::from_u16(0x8AB7), Opcode::SubtractYFromX { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_shift_left() {
        assert_eq!(Opcode::from_u16(0x8ABE), Opcode::ShiftLeft { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_skip_next_if_register_not_equal() {
        assert_eq!(Opcode::from_u16(0x9AB0), Opcode::SkipNextIfRegisterNotEqual { x: 0xA, y: 0xB });
    }

    #[test]
    fn parse_store_address() {
        assert_eq!(Opcode::from_u16(0xAABC), Opcode::StoreAddress(0xABC));
    }

    #[test]
    fn parse_jump_with_offset() {
        assert_eq!(Opcode::from_u16(0xBABC), Opcode::JumpWithOffset(0xABC));
    }

    #[test]
    fn parse_random() {
        assert_eq!(Opcode::from_u16(0xC152), Opcode::Random { x: 0x1, mask: 0x52 });
    }

    #[test]
    fn parse_draw() {
        assert_eq!(Opcode::from_u16(0xDAB1), Opcode::Draw { x: 0xA, y: 0xB, n: 0x1 });
    }

    #[test]
    fn parse_skip_if_key_pressed() {
        assert_eq!(Opcode::from_u16(0xEA9E), Opcode::SkipIfKeyPressed { x: 0xA });
    }

    #[test]
    fn parse_skip_if_key_not_pressed() {
        assert_eq!(Opcode::from_u16(0xEAA1), Opcode::SkipIfKeyNotPressed { x: 0xA });
    }

    #[test]
    fn parse_store_delay() {
        assert_eq!(Opcode::from_u16(0xFA07), Opcode::StoreDelay { x: 0xA });
    }

    #[test]
    fn parse_wait_for_keypress() {
        assert_eq!(Opcode::from_u16(0xFA0A), Opcode::WaitForKeyPress { x: 0xA });
    }

    #[test]
    fn parse_set_delay() {
        assert_eq!(Opcode::from_u16(0xFA15), Opcode::SetDelay { x: 0xA });
    }

    #[test]
    fn parse_store_sound() {
        assert_eq!(Opcode::from_u16(0xFA18), Opcode::StoreSound { x: 0xA });
    }

    #[test]
    fn parse_add_index() {
        assert_eq!(Opcode::from_u16(0xFA1E), Opcode::AddIndex { x: 0xA });
    }

    #[test]
    fn parse_set_index_to_font_data() {
        assert_eq!(Opcode::from_u16(0xFA29), Opcode::SetIndexToFontData { x: 0xA });
    }

    #[test]
    fn parse_store_bcd() {
        assert_eq!(Opcode::from_u16(0xFA33), Opcode::StoreBCD { x: 0xA });
    }

    #[test]
    fn parse_write_memory() {
        assert_eq!(Opcode::from_u16(0xFA55), Opcode::WriteMemory { x: 0xA });
    }

    #[test]
    fn parse_read_memory() {
        assert_eq!(Opcode::from_u16(0xFA65), Opcode::ReadMemory { x: 0xA });
    }
}
