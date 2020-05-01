pub enum Opcode {
    /// Opcode: 0030
    ///
    /// Clear the display
    ClearScreen,

    /// Opcode: 1nnn
    ///
    /// Jump to address  nnn
    Jump(u16),

    /// 6xkk - LD Vx, byte
    ///
    /// Load the value kk into the register Vx.
    LoadConstant { register: u8, value: u8 }
}

impl Opcode {
    pub fn from_u8_bytes(bytes: &[u8; 2]) -> Opcode {
        Opcode::LoadConstant { register: 0, value: 250 }
    }
}
