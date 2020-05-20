use crate::chip8::{Chip8Error, Chip8Result, Register, Address};

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
/// | 8xy5   | SUBXY Vx, Vy      | Manipulate Vx         | Set Vx to Vx - Vy. Set VF to carry       |
/// | 8xy7   | SUBYX Vx, Vy      | Manipulate Vx         | Set Vx to Vy - Vx. Set VF to carry       |
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
    /// - Set `VF` to 01 if a borrow _does not_ occur, otherwise set `VF` to 00.
    SubtractXY { x: Register, y: Register },

    /// Assembly: `SUBXY`
    /// Opcode: `8xy7`
    ///
    /// - Set `Vx` to `Vy - Vx`.
    /// - Set `VF` to 01 if a borrow _does not_ occur, otherwise set `VF` to 00.
    SubtractYX { x: Register, y: Register },

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
    pub fn from_bytes(bytes: &[u8; 2]) -> Chip8Result<Opcode> {
        let opcode = u16::from_be_bytes(*bytes);
        Opcode::from_u16(opcode)
    }

    /// Return the byte representation of this opcode.
    pub fn to_bytes(&self) -> [u8; 2] {
        self.to_u16().to_be_bytes()
    }

    pub fn to_rom(opcodes: Vec<Opcode>) -> Vec<u8> {
        opcodes.iter()
            .flat_map(|op| op.to_bytes().to_vec())
            .collect()
    }

    pub fn from_u16(word: u16) -> Chip8Result<Opcode> {
        let nibbles = (
            ((word & 0xF000) >> 12) as u8,
            ((word & 0x0F00) >> 8) as u8,
            ((word & 0x00F0) >> 4) as u8,
            (word & 0x000F) as u8
        );

        match nibbles {
            // Flow Control
            (0x2, _, _, _) => Ok(Opcode::CallSubroutine(word & 0x0FFF)),
            (0x0, 0x0, 0xE, 0xE) => Ok(Opcode::Return),
            (0x1, _, _, _) => Ok(Opcode::Jump(word & 0x0FFF)),
            (0xB, _, _, _) => Ok(Opcode::JumpWithOffset(word & 0x0FFF)),

            // Conditional Execution
            (0x3, x, _, _) => Ok(Opcode::SkipNextIfEqual { x, value: (word & 0x00FF) as u8 }),
            (0x4, x, _, _) => Ok(Opcode::SkipNextIfNotEqual { x, value: (word & 0x00FF) as u8 }),
            (0x5, x, y, 0x0) => Ok(Opcode::SkipNextIfRegisterEqual { x, y }),
            (0x9, x, y, 0x0) => Ok(Opcode::SkipNextIfRegisterNotEqual { x, y }),

            // Manipulate Vx
            (0x6, x, _, _) => Ok(Opcode::LoadConstant { x, value: (word & 0x00FF) as u8 }),
            (0x8, x, y, 0x0) => Ok(Opcode::Load { x, y }),
            (0x8, x, y, 0x1) => Ok(Opcode::Or { x, y }),
            (0x8, x, y, 0x2) => Ok(Opcode::And { x, y }),
            (0x8, x, y, 0x3) => Ok(Opcode::Xor { x, y }),
            (0x8, x, y, 0x4) => Ok(Opcode::Add { x, y }),
            (0x7, x, _, _)   => Ok(Opcode::AddConstant { x, value: (word & 0x00FF) as u8 }),
            (0x8, x, y, 0x5) => Ok(Opcode::SubtractXY { x, y }),
            (0x8, x, y, 0x7) => Ok(Opcode::SubtractYX { x, y }),
            (0x8, x, y, 0x6) => Ok(Opcode::ShiftRight { x, y }),
            (0x8, x, y, 0xE) => Ok(Opcode::ShiftLeft { x, y }),

            // Manipulate I
            (0xA, _, _, _) => Ok(Opcode::IndexAddress(word & 0x0FFF)),
            (0xF, x, 0x1, 0xE) => Ok(Opcode::AddAddress { x }),
            (0xF, x, 0x2, 0x9) => Ok(Opcode::IndexFont { x }),

            // Manipulate Memory
            (0xF, x, 0x3, 0x3) => Ok(Opcode::WriteBCD { x }),
            (0xF, x, 0x5, 0x5) => Ok(Opcode::WriteMemory { x }),
            (0xF, x, 0x6, 0x5) => Ok(Opcode::ReadMemory { x }),

            // IO
            (0xE, x, 0x9, 0xE) => Ok(Opcode::SkipIfKeyPressed { x }),
            (0xE, x, 0xA, 0x1) => Ok(Opcode::SkipIfKeyNotPressed { x }),
            (0xF, x, 0x0, 0xA) => Ok(Opcode::WaitForKeyRelease { x }),
            (0xF, x, 0x0, 0x7) => Ok(Opcode::LoadDelayIntoRegister { x }),
            (0xF, x, 0x1, 0x5) => Ok(Opcode::LoadRegisterIntoDelay { x }),
            (0xF, x, 0x1, 0x8) => Ok(Opcode::LoadRegisterIntoSound { x }),
            (0xC, x, _, _) => Ok(Opcode::Random { x, mask: (word & 0x00FF) as u8 }),
            (0x0, 0x0, 0xE, 0x0) => Ok(Opcode::ClearScreen),
            (0xD, x, y, n) => Ok(Opcode::Draw { x, y, n }),

            _ => Err(Chip8Error::UnsupportedOpcode(word)),
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
            Opcode::SubtractXY { x, y } => 0x8005 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::SubtractYX { x, y } => 0x8007 | ((*x as u16) << 8) | ((*y as u16) << 4),
            Opcode::ShiftRight { x, y } => 0x8006 | ((*x as u16) << 8) | ((*y as u16) << 4),
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

    /// Return the Assembly name of this opcode
    pub fn to_assembly_name(&self) -> &str {
        match self {
            // Flow Control
            Opcode::CallSubroutine(_) => "CALL",
            Opcode::Return => "RET",
            Opcode::Jump(_) => "JUMP",
            Opcode::JumpWithOffset(_) => "JUMP",

            // Conditional Execution
            Opcode::SkipNextIfEqual { x: _, value: _ } => "SKIP.EQ",
            Opcode::SkipNextIfNotEqual { x: _, value: _ } => "SKIP.NE",
            Opcode::SkipNextIfRegisterEqual { x: _, y: _ } => "SKIP.EQ",
            Opcode::SkipNextIfRegisterNotEqual { x: _, y: _ } => "SKIP.NE",

            // Manipulate Vx
            Opcode::LoadConstant { x: _, value: _ } => "LOAD",
            Opcode::Load { x: _, y: _ } => "LOAD",
            Opcode::Or { x: _, y: _ } => "OR",
            Opcode::And { x: _, y: _ } => "AND",
            Opcode::Xor { x: _, y: _ } => "XOR",
            Opcode::Add { x: _, y: _ } => "ADD",
            Opcode::AddConstant { x: _, value: _ } => "ADD",
            Opcode::SubtractXY { x: _, y: _ } => "SUBXY",
            Opcode::SubtractYX { x: _, y: _ } => "SUBYX",
            Opcode::ShiftRight { x: _, y: _ } => "SHR",
            Opcode::ShiftLeft { x: _, y: _ } => "SHL",

            // Manipulate I
            Opcode::IndexAddress(_) => "IDX",
            Opcode::AddAddress { x: _ } => "ADD",
            Opcode::IndexFont { x: _ } => "FONT",

            // Manipulate Memory
            Opcode::WriteMemory { x: _ } => "WRITE",
            Opcode::WriteBCD { x: _ } => "BCD",
            Opcode::ReadMemory { x: _ } => "READ",

            // IO
            Opcode::SkipIfKeyPressed { x: _ } => "SKIP.KEQ",
            Opcode::SkipIfKeyNotPressed { x: _ } => "SKIP.KNE",
            Opcode::WaitForKeyRelease { x: _ } => "KEY",
            Opcode::LoadDelayIntoRegister { x: _ } => "LOAD",
            Opcode::LoadRegisterIntoDelay { x: _ } => "LOAD",
            Opcode::LoadRegisterIntoSound { x: _ } => "LOAD",
            Opcode::Random { x: _, mask: _ } => "RAND",
            Opcode::ClearScreen => "CLEAR",
            Opcode::Draw { x: _, y: _, n: _ } => "DRAW",
        }
    }

    pub fn to_assembly_args(&self) -> Option<String> {
        let fmt_addr = |addr| Some(format!("{:03X}", addr));
        let fmt_reg_value = |x, value| Some(format!("V{:X}, {:02X}", x, value));
        let fmt_reg_reg = |x, y| Some(format!("V{:X}, V{:X}", x, y));
        let fmt_reg = |x| Some(format!("V{:X}", x));

        match self {
            // Flow Control
            Opcode::CallSubroutine(addr) => fmt_addr(addr),
            Opcode::Return => None,
            Opcode::Jump(addr) => fmt_addr(addr),
            Opcode::JumpWithOffset(addr) => fmt_addr(addr),

            // Conditional Execution
            Opcode::SkipNextIfEqual { x, value } => fmt_reg_value(x, value),
            Opcode::SkipNextIfNotEqual { x, value } => fmt_reg_value(x, value),
            Opcode::SkipNextIfRegisterEqual { x, y } => fmt_reg_reg(x, y),
            Opcode::SkipNextIfRegisterNotEqual { x, y } => fmt_reg_reg(x, y),

            // Manipulate Vx
            Opcode::LoadConstant { x, value } => fmt_reg_value(x, value),
            Opcode::Load { x, y } => fmt_reg_reg(x, y),
            Opcode::Or { x, y } => fmt_reg_reg(x, y),
            Opcode::And { x, y } => fmt_reg_reg(x, y),
            Opcode::Xor { x, y } => fmt_reg_reg(x, y),
            Opcode::Add { x, y } => fmt_reg_reg(x, y),
            Opcode::AddConstant { x, value } => fmt_reg_value(x, value),
            Opcode::SubtractXY { x, y } => fmt_reg_reg(x, y),
            Opcode::SubtractYX { x, y } => fmt_reg_reg(x, y),
            Opcode::ShiftRight { x, y } => fmt_reg_reg(x, y),
            Opcode::ShiftLeft { x, y } => fmt_reg_reg(x, y),

            // // Manipulate I
            Opcode::IndexAddress(addr) => fmt_addr(addr),
            Opcode::AddAddress { x } => Some(format!("I, V{:X}", x)),
            Opcode::IndexFont { x } => fmt_reg(x),

            // // Manipulate Memory
            Opcode::WriteMemory { x } => fmt_reg(x),
            Opcode::WriteBCD { x } => fmt_reg(x),
            Opcode::ReadMemory { x } => fmt_reg(x),

            // // IO
            Opcode::SkipIfKeyPressed { x } => fmt_reg(x),
            Opcode::SkipIfKeyNotPressed { x } => fmt_reg(x),
            Opcode::WaitForKeyRelease { x } => fmt_reg(x),
            Opcode::LoadDelayIntoRegister { x } => Some(format!("V{:X}, DELAY", x)),
            Opcode::LoadRegisterIntoDelay { x } => Some(format!("DELAY, V{:X}", x)),
            Opcode::LoadRegisterIntoSound { x } => Some(format!("SOUND, V{:X}", x)),
            Opcode::Random { x, mask } => fmt_reg_value(x, mask),
            Opcode::ClearScreen => None,
            Opcode::Draw { x, y, n } => Some(format!("V{:X}, V{:X}, V{:X}", x, y, n)),
        }
    }

    pub fn to_assembly(&self) -> String {
        let mut assembly = self.to_assembly_name().to_string();

        if let Some(mut args) = self.to_assembly_args() {
            args.retain(|c| !c.is_whitespace());

            assembly += " ";
            assembly += &args;
        }

        assembly
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

    /// `opcode_test` generates data-driven tests for all opcodes covering:
    ///
    /// - `Opcode::from_u16`
    /// - `Opcode::to_u16`
    /// - `Opcode::to_assembly_name`
    /// - `Opcode::to_assembly_args`
    ///
    macro_rules! opcode_tests {
        ($opcode_name:ident, $opcode:expr, $u16_value:expr, $assembly:expr) => {
            paste::item! {
                #[test]
                fn [<$opcode_name:snake _to_u16>]() {
                    assert_eq!($opcode.to_u16(), $u16_value);
                }
            }

            paste::item! {
                #[test]
                fn [<$opcode_name:snake _from_u16>]() {
                    assert_eq!(Opcode::from_u16($u16_value), Ok($opcode));
                }
            }

            paste::item! {
                #[test]
                fn [<$opcode_name:snake _to_assembly>]() {
                    let assembly = $opcode.to_assembly();
                    assert_eq!(assembly, $assembly);
                }
            }
        }
    }

    // Flow Control
    opcode_tests!(CallSubroutine, Opcode::CallSubroutine(0xABC), 0x2ABC, "CALL ABC");
    opcode_tests!(Return, Opcode::Return, 0x00EE, "RET");
    opcode_tests!(Jump, Opcode::Jump(0xABC), 0x1ABC, "JUMP ABC");
    opcode_tests!(JumpWithOffset, Opcode::JumpWithOffset(0xABC), 0xBABC, "JUMP ABC");

    // Conditioonal Execution
    opcode_tests!(SkipNextIfEqual, Opcode::SkipNextIfEqual { x: 0xA, value: 0x15 }, 0x3A15, "SKIP.EQ VA,15");
    opcode_tests!(SkipNextIfNotEqual, Opcode::SkipNextIfNotEqual { x: 0xA, value: 0x15 }, 0x4A15, "SKIP.NE VA,15");
    opcode_tests!(SkipNextIfRegisterEqual, Opcode::SkipNextIfRegisterEqual { x: 0xA, y: 0xB }, 0x5AB0, "SKIP.EQ VA,VB");
    opcode_tests!(SkipNextIfRegisterNotEqual, Opcode::SkipNextIfRegisterNotEqual { x: 0xA, y: 0xB }, 0x9AB0, "SKIP.NE VA,VB");

    // Manipulate Vx
    opcode_tests!(LoadConstant, Opcode::LoadConstant { x: 0xA, value: 0x10 }, 0x6A10, "LOAD VA,10");
    opcode_tests!(Load, Opcode::Load { x: 0xA, y: 0xB }, 0x8AB0, "LOAD VA,VB");
    opcode_tests!(Or, Opcode::Or { x: 0xA, y: 0xB }, 0x8AB1, "OR VA,VB");
    opcode_tests!(And, Opcode::And { x: 0xA, y: 0xB }, 0x8AB2, "AND VA,VB");
    opcode_tests!(Xor, Opcode::Xor { x: 0xA, y: 0xB }, 0x8AB3, "XOR VA,VB");
    opcode_tests!(Add, Opcode::Add { x: 0xA, y: 0xB }, 0x8AB4, "ADD VA,VB");
    opcode_tests!(AddConstant, Opcode::AddConstant { x: 0xA, value: 0x10 }, 0x7A10, "ADD VA,10");
    opcode_tests!(SubtractXY, Opcode::SubtractXY { x: 0xA, y: 0xB }, 0x8AB5, "SUBXY VA,VB");
    opcode_tests!(SubtractYX, Opcode::SubtractYX { x: 0xA, y: 0xB }, 0x8AB7, "SUBYX VA,VB");
    opcode_tests!(ShiftRight, Opcode::ShiftRight { x: 0xA, y: 0xB }, 0x8AB6, "SHR VA,VB");
    opcode_tests!(ShiftLeft, Opcode::ShiftLeft { x: 0xA, y: 0xB }, 0x8ABE, "SHL VA,VB");

    // Manipulate I
    opcode_tests!(IndexAddress, Opcode::IndexAddress(0xABC), 0xAABC, "IDX ABC");
    opcode_tests!(AddAddress, Opcode::AddAddress { x: 0xA }, 0xFA1E, "ADD I,VA");
    opcode_tests!(IndexFont, Opcode::IndexFont { x: 0xA }, 0xFA29, "FONT VA");

    // Manipulate Memory
    opcode_tests!(WriteBCD, Opcode::WriteBCD { x: 0xA }, 0xFA33, "BCD VA");
    opcode_tests!(WriteMemory, Opcode::WriteMemory { x: 0xA }, 0xFA55, "WRITE VA");
    opcode_tests!(ReadMemory, Opcode::ReadMemory { x: 0xA }, 0xFA65, "READ VA");

    // IO
    opcode_tests!(SkipIfKeyPressed, Opcode::SkipIfKeyPressed { x: 0xA }, 0xEA9E, "SKIP.KEQ VA");
    opcode_tests!(SkipIfKeyNotPressed, Opcode::SkipIfKeyNotPressed { x: 0xA }, 0xEAA1, "SKIP.KNE VA");
    opcode_tests!(WaitForKeyRelease, Opcode::WaitForKeyRelease { x: 0xA }, 0xFA0A, "KEY VA");
    opcode_tests!(LoadDelayIntoRegister, Opcode::LoadDelayIntoRegister { x: 0xA }, 0xFA07, "LOAD VA,DELAY");
    opcode_tests!(LoadRegisterIntoDelay, Opcode::LoadRegisterIntoDelay { x: 0xA }, 0xFA15, "LOAD DELAY,VA");
    opcode_tests!(LoadRegisterIntoSound, Opcode::LoadRegisterIntoSound { x: 0xA }, 0xFA18, "LOAD SOUND,VA");
    opcode_tests!(Random, Opcode::Random { x: 0x1, mask: 0x52 }, 0xC152, "RAND V1,52");
    opcode_tests!(ClearScreen, Opcode::ClearScreen, 0x00E0, "CLEAR");
    opcode_tests!(Draw, Opcode::Draw { x: 0xA, y: 0xB, n: 0x1 }, 0xDAB1, "DRAW VA,VB,V1");
}
