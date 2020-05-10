use ggez::Context;
use ggez::graphics::{self, Text};

use crate::chip8::Chip8;
use crate::ui::{Assets, Point2};

/// Display the currently executing opcodes of a `Chip8` within a
/// 160x320 pixel window.
pub struct AssemblyDisplay {
    /// The horizontal position of this display relative to the main window
    x: f32,

    /// The vertical position of this display relative to the main window
    y: f32
}

impl AssemblyDisplay {
    pub fn new(x: f32, y: f32) -> AssemblyDisplay {
        AssemblyDisplay { x, y }
    }

    pub fn draw(&self, ctx: &mut Context, assets: &Assets, chip8: &Chip8) {
        let opcodes = chip8.current_opcode_window(10);
        for (i, (address, opcode)) in opcodes.iter().enumerate() {
            let opcode_pos = Point2::new(
                self.x + 10.0,
                self.y + 10.0 + ((i as f32) * 17.0)
            );

            let opcode_text = format!("{:5X} - {:6}", address, opcode.to_assembly_name());
            let opcode_text = Text::new((opcode_text, assets.debug_font, 16.0));
            graphics::draw(ctx, &opcode_text, (opcode_pos, 0.0, graphics::WHITE))
                .expect("Failed to draw text");
        }
    }
}
