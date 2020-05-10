use ggez::{Context, GameResult};
use ggez::graphics::{self, Text};

use crate::chip8::Chip8;
use crate::ui::{Assets, Point2};

/// Display the currently executing opcodes of a `Chip8` within a
/// 160x320 pixel window.
pub struct AssemblyDisplay {
    /// The horizontal position of this display relative to the main window
    x: f32,

    /// The vertical position of this display relative to the main window
    y: f32,

    /// The start address of the memory slice we are currently viewing
    window_start_address: u16,

    /// The end address of the memory slice we are currently viewing
    window_end_address: u16,

    lines: Vec<(Point2, Text)>,
}

impl AssemblyDisplay {
    pub fn new(x: f32, y: f32) -> AssemblyDisplay {
        AssemblyDisplay {
            x,
            y,
            window_start_address: Chip8::PROGRAM_START,
            window_end_address: Chip8::PROGRAM_START + (20 * 2),
            lines: Vec::new()
        }
    }

    pub fn update(&mut self, assets: &Assets, chip8: &Chip8) {
        // If the window is not viewing the current instruction we should shift the window and re-generate the text.
        if chip8.pc < self.window_start_address || chip8.pc > self.window_end_address  {
            self.window_start_address = chip8.pc - 2;
            self.window_end_address = chip8.pc + (20 * 2);

            self.lines.clear();

            let opcodes = chip8.opcodes(self.window_start_address, self.window_end_address);
            for (i, (address, opcode)) in opcodes.iter().enumerate() {
                let opcode_pos = Point2::new(
                    self.x + 10.0,
                    self.y + 10.0 + ((i as f32) * 17.0)
                );

                let opcode_text = format!("{:5X} - {:6}", address, opcode.to_assembly_name());
                let opcode_text = Text::new((opcode_text, assets.debug_font, 16.0));

                self.lines.push((opcode_pos, opcode_text));
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for (position, text) in &self.lines {
            graphics::draw(ctx, text, (*position, 0.0, graphics::WHITE))?
        }

        Ok(())
    }
}
