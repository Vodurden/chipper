mod chip8;
mod chip8_error;
mod opcode;
mod quirks;

pub use self::chip8::{Chip8, Chip8Output};
pub use self::opcode::Opcode;
pub use self::chip8_error::Chip8Error;

pub type Chip8Result<T> = Result<T, Chip8Error>;
pub type Register = u8;
pub type Address = u16;
