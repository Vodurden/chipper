use std::fmt;
use std::error;

#[derive(Debug, PartialEq)]
pub enum Chip8Error {
    UnsupportedOpcode(u16),
    StackUnderflow
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chip8Error::UnsupportedOpcode(value) => write!(f, "unsupported opcode: {:x}", value),
            Chip8Error::StackUnderflow => write!(f, "stack underflow!"),
        }
    }
}
impl error::Error for Chip8Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Chip8Error::UnsupportedOpcode(_) => None,
            Chip8Error::StackUnderflow => None,
        }
    }
}
