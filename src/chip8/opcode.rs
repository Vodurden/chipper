use crate::chip8::{Register, Address};

/// `Opcode` represents a single instruction available on the Chip-8
///
/// Each opcode is derived from a `u16` with different values corresponding to different instructions.
///
/// Available instructions:
///
/// ```text
/// | Opcode | Shorthand         | Purpose               | Description                              |
/// |--------+-------------------+-----------------------+------------------------------------------|
/// | 2nnn   | CALL addr         | Flow Control          | Call Subroutine                          |
/// | 00EE   | RET               | Flow Control          | Return                                   |
/// | 1nnn   | JUMP addr         | Flow Control          | Jump to Address                          |
/// | Bnnn   | JUMP addr,V0      | Flow Control          | Jump to Address with Offset              |
/// | 3xnn   | SKIP.EQ Vx, value | Conditional Execution | Skip Next If Equal                       |
/// | 5xy0   | SKIP.EQ Vx, Vy    | Conditional Execution | Skip Next If Registers Equal             |
/// | 4xnn   | SKIP.NE Vx, value | Conditional Execution | Skip Next If Not Equal                   |
/// | 9xy0   | SKIP.NE Vx, Vy    | Conditional Execution | Skip Next If Registers Not Equal         |
/// | 6xnn   | LOAD Vx, value    | Manipulate Vx         | Load Value into Vx                       |
/// | 8xy0   | LOAD Vx, Vy       | Manipulate Vx         | Load Vy into Vx                          |
/// | 8xy1   | OR Vx, Vy         | Manipulate Vx         | Set Vx to Vx OR Vy                       |
/// | 8xy2   | AND Vx, Vy        | Manipulate Vx         | Set Vx to Vx AND Vy                      |
/// | 8xy3   | XOR Vx, Vy        | Manipulate Vx         | Set Vx to Vx XOR Vy                      |
/// | 8xy4   | ADD Vx, Vy        | Manipulate Vx         | Set Vx to Vx + Vy. Set VF to carry       |
/// | 7xnn   | ADD Vx, value     | Manipulate Vx         | Set Vx to Vx + value                     |
/// | 8xy5   | SUBYX Vx, Vy      | Manipulate Vx         | Set Vx to Vx - Vy. Set VF to carry       |
/// | 8xy7   | SUBXY Vx, Vy      | Manipulate Vx         | Set Vx to Vy - Vx. Set VF to carry       |
/// | 8xy6   | SHR Vx            | Manipulate Vx         | Set Vx to Vx >> 1. Set VF to LSB         |
/// | 8xyE   | SHL Vx            | Manipulate Vx         | Set Vx to Vx << 1. Set VF to MSB         |
/// | Annn   | IDX addr          | Manipulate I          | Set I to addr                            |
/// | Fx1E   | ADD I, Vx         | Manipulate I          | Set I to I + Vx                          |
/// | Fx29   | FONT Vx           | Manipulate I          | Set I to the font data representing Vx   |
/// | Fx55   | WRITE Vx          | Manipulate Memory     | Write values V0..Vx to memory at I       |
/// | Fx33   | BCD Vx            | Manipulate Memory     | Write BCD of Vx to memory at I,I+1,I+2   |
/// | Fx65   | READ Vx           | Manipulate Memory     | Read memory at I into V0..Vx             |
/// | Ex9E   | SKIP.KEQ Vx       | IO (Keyboard)         | Skip next instruction if key pressed     |
/// | ExA1   | SKIP.KNE Vx       | IO (Keyboard)         | Skip next instruction if key not pressed |
/// | Fx0A   | KEY Vx            | IO (Keyboard)         | Wait for key release. Store key in Vx    |
/// | Fx07   | LOAD Vx, DELAY    | IO (Time)             | Load DELAY register into Vx              |
/// | Fx15   | LOAD DELAY, Vx    | IO (Time)             | Load Vx into DELAY register              |
/// | Fx18   | LOAD SOUND, Vx    | IO (Sound)            | Load Vx into SOUND register              |
/// | Cxnn   | RAND Vx, value    | IO (Random)           | Load (random & value) into Vx            |
/// | 00E0   | CLEAR             | IO (Display)          | Clear the display                        |
/// | Dxyn   | DRAW x, y, n      | IO (Display)          | Draw sprite to display                   |
/// ```
///
/// For more info see the individual docs for each instruction.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Opcode {
    // =======================================================================
    // = Flow Control Opcodes - Opcodes to jump between parts of the program =
    // =======================================================================

    /// Assembly: `CALL addr`
    /// Opcode: `2nnn`
    ///
    /// Call subroutine starting at address `nnn`.
    CallSubroutine(Address),

    /// Opcode: `00EE`
    /// Assembly: `RET`
    ///
    /// Return from a subroutine.
    Return,

    /// Assembly: `JUMP addr`
    /// Opcode: `1nnn`
    ///
    /// Jump to address `nnn`.
    Jump(Address),

    /// Assembly: `JUMP addr,V0`
    /// Opcode: `Bnnn`
    ///
    /// Jump to address `nnn + V0`.
    JumpWithOffset(Address),

    // ====================================================================
    // = Conditional Opcodes - Conditionally execute parts of the program =
    // ====================================================================

    /// Assembly: `SKIP.EQ Vx, value`
    /// Opcode: `3xnn`
    ///
    /// Skip the next instruction if register `Vx` equals `value`.
    SkipNextIfEqual { x: Register, value: u8 },

    /// Assembly: `SKIP.EQ Vx,Vy`
    /// Opcode: `5xy0`
    ///
    /// Skip the next instruction if register `Vx` is equal to register `Vy`.
    SkipNextIfRegisterEqual { x: Register, y: Register },

    /// Assembly: `SKIP.NE Vx, value`
    /// Opcode: `4xnn`
    ///
    /// Skip the next instruction if register `Vx` is _not_ equal to `value`.
    SkipNextIfNotEqual { x: Register, value: u8 },

    /// Assembly: `SKIP.NE Vx, Vy`
    /// Opcode: `9xy0`
    ///
    /// Skip the next instruction if register `Vx` is not equal to register `Vy`.
    SkipNextIfRegisterNotEqual { x: Register, y: Register },

    // ========================================================================
    // = `Vx` Opcodes - Opcodes to manipulate the value of the `Vx` registers =
    // ========================================================================

    /// Assembly: `LOAD Vx, value`
    /// Opcode: `6xnn`
    ///
    /// Load `value` into register `Vx`
    LoadConstant { x: Register, value: u8 },

    /// Assembly: `LOAD Vx, Vy`
    /// Opcode: `8xy0`
    ///
    /// Store the value of `Vy` in register `Vx`.
    Load { x: Register, y: Register },

    /// Assembly: `OR Vx, Vy`
    /// Opcode: `8xy1`
    ///
    /// Set `Vx` to `Vx OR Vy`.
    Or { x: Register, y: Register },

    /// Assembly: `AND Vx, Vy`
    /// Opcode: `8xy2`
    ///
    /// Set `Vx` to `Vx AND Vy`.
    And { x: Register, y: Register },

    /// Assembly: `XOR Vx, Vy`
    /// Opcode: `8xy3`
    ///
    /// Set `Vx` to `Vx XOR Vy`.
    Xor { x: Register, y: Register },

    /// Assembly: `ADD Vx, Vy`
    /// Opcode: `8xy4`
    ///
    /// - Set `Vx` to `Vx + Vy`.
    /// - Set `VF` to 01 if a carry occurs, otherwise set `VF` to 00.
    Add { x: Register, y: Register },

    /// Assembly: `ADD Vx, value`
    /// Opcode: `7xnn`
    ///
    /// Add the value nn into register `Vx`.
    AddConstant { x: Register, value: u8 },

    /// Assembly: `SUBYX`
    /// Opcode: `8xy5`
    ///
    /// - Set `Vx` to `Vx - Vy`.
    /// - Set `VF` to 00 if a borrow occurs, otherwise set `VF` to 00.
    SubtractYFromX { x: Register, y: Register },

    /// Assembly: `SUBXY`
    /// Opcode: `8xy7`
    ///
    /// - Set `Vx` to `Vy - Vx`.
    /// - Set `VF` to 00 if a borrow occurs, otherwise set `VF` to 00.
    SubtractXFromY { x: Register, y: Register },

    /// Assembly: `SHR Vx`
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

    /// Assembly: `SHL Vx`
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


    // ========================================================
    // = `I` Opcodes - Opcodes to manipulate the `I` register =
    // ========================================================

    /// Assembly: `IDX addr`
    /// Opcode: `Annn`
    ///
    /// Store address `nnn` in register `I`.
    IndexAddress(Address),

    /// Assembly: `ADD I, Vx`
    /// Opcode: `Fx1E`
    ///
    /// Add the value of `Vx` to `I`
    AddAddress { x: Register },

    /// Assembly: `FONT Vx`
    /// Opcode: `Fx29`
    ///
    /// Set `I` to the font data corresponding to the value of `Vx`.
    IndexFont { x: Register },

    // =================================================
    // = Memory Opcodes - Opcodes to Read/Write memory =
    // =================================================

    /// Assembly: `WRITE Vx`
    /// Opcode: `Fx55`
    ///
    /// - Store the values of `V0..Vx` (inclusive) in memory starting at address `I`.
    /// - Set `I` to `I + x + 1`.
    WriteMemory { x: Register },

    /// Assembly: `BCD Vx`
    /// Opcode: `Fx33`
    ///
    /// Store the binary-coded decimal equivalent of the value stored in `Vx` at addresses `I`, `I+1` and `I+2`.
    WriteBCD { x: Register },

    /// Assembly: `READ Vx`
    /// Opcode: `Fx65`
    ///
    /// - Fill registers `V0..Vx` (inclusive) with the values stored in memory starting at address `I`.
    /// - Set `I` to `I + x + 1`.
    ReadMemory { x: Register },


    // ============================================================================================
    // = IO Opcodes - Opcodes for interacting with the real world (drawing, input, sound, etc...) =
    // ============================================================================================

    /// Assembly: `SKIP.KEQ Vx`
    /// Opcode: `Ex9E`
    ///
    /// Skip the next instruction if the key corresponding to the value of `Vx` is pressed.
    SkipIfKeyPressed { x: Register },

    /// Assembly: `SKIP.KNE Vx`
    /// Opcode: `ExA1`
    ///
    /// Skip the next instruction if the key corresponding to the value of `Vx` is not pressed.
    SkipIfKeyNotPressed { x: Register },

    /// Assembly: `KEY Vx`
    /// Opcode: `Fx0A`
    ///
    /// Halt the program until the specified key is released. Store the key that was released in `Vx`.
    ///
    /// See: [here](https://retrocomputing.stackexchange.com/a/361) for more information.
    WaitForKeyRelease { x: Register },

    /// Assembly: `LOAD Vx, DELAY`
    /// Opcode: `Fx07`
    ///
    /// Read the value of the delay timer into `Vx`.
    LoadDelayIntoRegister { x: Register },

    /// Assembly: `LOAD DELAY, Vx`
    /// Opcode: `Fx15`
    ///
    /// Set the delay timer to the value of `Vx`.
    LoadRegisterIntoDelay { x: Register },

    /// Assembly: `LOAD SOUND, Vx`
    /// Opcode: `Fx18`
    ///
    /// Store the value of `Vx` in the sound timer.
    LoadRegisterIntoSound { x: Register },

    /// Assembly: `RAND x, nn`
    /// Opcode: `Cxnn`
    ///
    /// Set `Vx` to a random number masked with `nn`.
    Random { x: Register, mask: u8 },

    /// Assembly: `CLEAR`
    /// Opcode: `00E0`
    ///
    /// Clear the display.
    ClearScreen,

    /// Assembly: `DRAW x, y, n`
    /// Opcode: `Dxyn`
    ///
    /// - Draw a sprite at position `Vx`, `Vy`. The spirte has dimensions 8 by `n`
    /// - The sprite data is read starting from the address stored in `I`.
    /// - Set `VF` to 01 if any set pixels are changed to unset, otherwise set `VF` to 00
    ///
    /// When `Draw` is executed it also triggers a screen refresh
    Draw { x: Register, y: Register, n: u8 },
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
            // Flow Control
            (0x2, _, _, _) => Opcode::CallSubroutine(word & 0x0FFF),
            (0x0, 0x0, 0xE, 0xE) => Opcode::Return,
            (0x1, _, _, _) => Opcode::Jump(word & 0x0FFF),
            (0xB, _, _, _) => Opcode::JumpWithOffset(word & 0x0FFF),

            // Conditional Execution
            (0x3, x, _, _) => Opcode::SkipNextIfEqual { x, value: (word & 0x00FF) as u8 },
            (0x4, x, _, _) => Opcode::SkipNextIfNotEqual { x, value: (word & 0x00FF) as u8 },
            (0x5, x, y, 0x0) => Opcode::SkipNextIfRegisterEqual { x, y },
            (0x9, x, y, 0x0) => Opcode::SkipNextIfRegisterNotEqual { x, y },

            // Manipulate Vx
            (0x6, x, _, _) => Opcode::LoadConstant { x, value: (word & 0x00FF) as u8 },
            (0x8, x, y, 0x0) => Opcode::Load { x, y },
            (0x8, x, y, 0x1) => Opcode::Or { x, y },
            (0x8, x, y, 0x2) => Opcode::And { x, y },
            (0x8, x, y, 0x3) => Opcode::Xor { x, y },
            (0x8, x, y, 0x4) => Opcode::Add { x, y },
            (0x7, x, _, _)   => Opcode::AddConstant { x, value: (word & 0x00FF) as u8 },
            (0x8, x, y, 0x5) => Opcode::SubtractXFromY { x, y },
            (0x8, x, y, 0x6) => Opcode::ShiftRight { x, y },
            (0x8, x, y, 0x7) => Opcode::SubtractYFromX { x, y },
            (0x8, x, y, 0xE) => Opcode::ShiftLeft { x, y },

            // Manipulate I
            (0xA, _, _, _) => Opcode::IndexAddress(word & 0x0FFF),
            (0xF, x, 0x1, 0xE) => Opcode::AddAddress { x },
            (0xF, x, 0x2, 0x9) => Opcode::IndexFont { x },

            // Manipulate Memory
            (0xF, x, 0x3, 0x3) => Opcode::WriteBCD { x },
            (0xF, x, 0x5, 0x5) => Opcode::WriteMemory { x },
            (0xF, x, 0x6, 0x5) => Opcode::ReadMemory { x },

            // IO
            (0xE, x, 0x9, 0xE) => Opcode::SkipIfKeyPressed { x },
            (0xE, x, 0xA, 0x1) => Opcode::SkipIfKeyNotPressed { x },
            (0xF, x, 0x0, 0xA) => Opcode::WaitForKeyRelease { x },
            (0xF, x, 0x0, 0x7) => Opcode::LoadDelayIntoRegister { x },
            (0xF, x, 0x1, 0x5) => Opcode::LoadRegisterIntoDelay { x },
            (0xF, x, 0x1, 0x8) => Opcode::LoadRegisterIntoSound { x },
            (0xC, x, _, _) => Opcode::Random { x, mask: (word & 0x00FF) as u8 },
            (0x0, 0x0, 0xE, 0x0) => Opcode::ClearScreen,
            (0xD, x, y, n) => Opcode::Draw { x, y, n },

            // TODO: Better error handling
            _ => panic!("Unsupported opcode: {:x?}", word),
        }
    }

    #[allow(dead_code)]
    pub fn to_u16(&self) -> u16 {
        match self {
            // Flow Control
            Opcode::CallSubroutine(address) => 0x2000 | address,
            Opcode::Return => 0x00EE,
            Opcode::Jump(address) => 0x1000 | address,
            Opcode::JumpWithOffset(address) => 0xB000 | address,

            // Conditional Execution
            Opcode::SkipNextIfEqual { x, value } => 0x3000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::SkipNextIfNotEqual { x, value } => 0x4000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::SkipNextIfRegisterEqual { x, y } => 0x5000 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::SkipNextIfRegisterNotEqual { x, y } => 0x9000 | ((*x as u16) << 8) | ((*y as u16) << 4),

            // Manipulate Vx
            Opcode::LoadConstant { x, value } => 0x6000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::Load { x, y } => 0x8000 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Or { x, y } => 0x8001 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::And { x, y } => 0x8002 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Xor { x, y } => 0x8003 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::Add { x, y } => 0x8004 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::AddConstant { x, value } => 0x7000 | ((*x as u16) << 8) | (*value as u16),
            Opcode::SubtractXFromY { x, y } => 0x8005 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::ShiftRight { x, y } => 0x8006 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::SubtractYFromX { x, y } => 0x8007 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::ShiftLeft { x, y } => 0x800E | ((*x as u16) << 8) | ((*y as u16) << 4),

            // Manipulate I
            Opcode::IndexAddress(address) => 0xA000 | address,
            Opcode::AddAddress { x } => 0xF01E | ((*x as u16) << 8),
            Opcode::IndexFont { x } => 0xF029 | ((*x as u16) << 8),

            // Manipulate Memory
            Opcode::WriteMemory { x } => 0xF055 | ((*x as u16) << 8),
            Opcode::WriteBCD { x } => 0xF033 | ((*x as u16) << 8),
            Opcode::ReadMemory { x } => 0xF065 | ((*x as u16) << 8),

            // IO
            Opcode::SkipIfKeyPressed { x } => 0xE09E | ((*x as u16) << 8),
            Opcode::SkipIfKeyNotPressed { x } => 0xE0A1 | ((*x as u16) << 8),
            Opcode::WaitForKeyRelease { x } => 0xF00A | ((*x as u16) << 8),
            Opcode::LoadDelayIntoRegister { x } => 0xF007 | ((*x as u16) << 8),
            Opcode::LoadRegisterIntoDelay { x } => 0x0F015 | ((*x as u16) << 8),
            Opcode::LoadRegisterIntoSound { x } => 0xF018 | ((*x as u16) << 8),
            Opcode::Random { x, mask } => 0xC000 | ((*x as u16) << 8) | (*mask as u16),
            Opcode::ClearScreen => 0x00E0,
            Opcode::Draw { x, y, n } => 0xD000 | ((*x as u16) << 8) | ((*y as u16) << 4) | (*n as u16),
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
        assert_eq!(Opcode::LoadConstant { x: 0xA, value: 0x10 }.to_u16(), 0x6A10);
    }

    #[test]
    fn to_u16_add_constant() {
        assert_eq!(Opcode::AddConstant { x: 0xA, value: 0x10 }.to_u16(), 0x7A10);
    }

    #[test]
    fn to_u16_store() {
        assert_eq!(Opcode::Load { x: 0xA, y: 0xB }.to_u16(), 0x8AB0);
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
        assert_eq!(Opcode::IndexAddress(0xABC).to_u16(), 0xAABC);
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
    fn to_u16_wait_for_key_release() {
        assert_eq!(Opcode::WaitForKeyRelease { x: 0xA }.to_u16(), 0xFA0A);
    }

    #[test]
    fn to_u16_store_delay() {
        assert_eq!(Opcode::LoadDelayIntoRegister { x: 0xA }.to_u16(), 0xFA07);
    }

    #[test]
    fn to_u16_set_delay() {
        assert_eq!(Opcode::LoadRegisterIntoDelay { x: 0xA }.to_u16(), 0xFA15);
    }

    #[test]
    fn to_u16_store_sound() {
        assert_eq!(Opcode::LoadRegisterIntoSound { x: 0xA }.to_u16(), 0xFA18);
    }

    #[test]
    fn to_u16_add_address() {
        assert_eq!(Opcode::AddAddress { x: 0xA }.to_u16(), 0xFA1E);
    }

    #[test]
    fn to_u16_set_index_to_font_data() {
        assert_eq!(Opcode::IndexFont { x: 0xA }.to_u16(), 0xFA29);
    }

    #[test]
    fn to_u16_store_bcd() {
        assert_eq!(Opcode::WriteBCD { x: 0xA }.to_u16(), 0xFA33);
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
        assert_eq!(Opcode::from_u16(0x6A10), Opcode::LoadConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn from_u16_add_constant() {
        assert_eq!(Opcode::from_u16(0x7A10), Opcode::AddConstant { x: 0xA, value: 0x10 });
    }

    #[test]
    fn from_u16_store() {
        assert_eq!(Opcode::from_u16(0x8AB0), Opcode::Load { x: 0xA, y: 0xB });
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
        assert_eq!(Opcode::from_u16(0xAABC), Opcode::IndexAddress(0xABC));
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
    fn from_u16_wait_for_key_release() {
        assert_eq!(Opcode::from_u16(0xFA0A), Opcode::WaitForKeyRelease { x: 0xA });
    }

    #[test]
    fn from_u16_store_delay() {
        assert_eq!(Opcode::from_u16(0xFA07), Opcode::LoadDelayIntoRegister { x: 0xA });
    }

    #[test]
    fn from_u16_set_delay() {
        assert_eq!(Opcode::from_u16(0xFA15), Opcode::LoadRegisterIntoDelay { x: 0xA });
    }

    #[test]
    fn from_u16_store_sound() {
        assert_eq!(Opcode::from_u16(0xFA18), Opcode::LoadRegisterIntoSound { x: 0xA });
    }

    #[test]
    fn from_u16_add_address() {
        assert_eq!(Opcode::from_u16(0xFA1E), Opcode::AddAddress { x: 0xA });
    }

    #[test]
    fn from_u16_set_index_to_font_data() {
        assert_eq!(Opcode::from_u16(0xFA29), Opcode::IndexFont { x: 0xA });
    }

    #[test]
    fn from_u16_store_bcd() {
        assert_eq!(Opcode::from_u16(0xFA33), Opcode::WriteBCD { x: 0xA });
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
