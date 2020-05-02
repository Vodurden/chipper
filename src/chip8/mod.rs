mod chip8;
mod opcode;

pub use self::chip8::Chip8;
pub use self::opcode::Opcode;

pub type Register = u8;
pub type Address = u16;
