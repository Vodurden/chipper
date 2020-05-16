/// The original Chip-8 would increment `I` after executing `READ` or `WRITE`.
///
/// Most modern games assume that `I` is _not_ incremented as that's what Super Chip-8 1.1 does.
#[derive(PartialEq, Debug)]
pub enum ReadWriteIncrementQuirk {
    /// Do nothing to `I` after executing `READ` or `WRITE`
    InvariantIndex,

    /// Increment `I` to `I + x + 1`, i.e. the position after the memory was written to
    IncrementIndex
}

impl Default for ReadWriteIncrementQuirk {
    fn default() -> ReadWriteIncrementQuirk {
        ReadWriteIncrementQuirk::InvariantIndex
    }
}

/// The behavior of `SHL` and `SHR` would shift `Vx` and `Vy` on the original Chip-8.
///
/// Most modern games assume that only `Vx` is shifted.
///
/// - Original Chip-8: SHL: `Vx = Vy << 1`, SHR: `Vx = Vy >> 1`
/// - Super Chip-8: SHL: `Vx = Vx << 1`, SHR: `Vx >> 1`
#[derive(PartialEq, Debug)]
pub enum BitShiftQuirk {
    ShiftX,

    ShiftYIntoX
}

impl Default for BitShiftQuirk {
    fn default() -> BitShiftQuirk {
        BitShiftQuirk::ShiftX
    }
}
