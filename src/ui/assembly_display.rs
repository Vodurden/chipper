use ggez::{Context, GameResult};
use ggez::graphics::{self, Text, DrawParam, FilterMode};

use crate::chip8::Chip8;
use crate::ui::{Assets, Point2, Vector2};

/// Display the currently executing opcodes of a `Chip8` within a
/// 220x320 pixel window.
pub struct AssemblyDisplay {
    /// The horizontal position of this display relative to the main window
    x: f32,

    /// The vertical position of this display relative to the main window
    y: f32,

    /// The start address of the memory slice we are currently viewing
    window_start_address: u16,

    /// The end address of the memory slice we are currently viewing
    window_end_address: u16,

    text: Vec<(Point2, Text)>,
}

impl AssemblyDisplay {
    pub fn new(x: f32, y: f32) -> AssemblyDisplay {
        AssemblyDisplay {
            x,
            y,
            window_start_address: Chip8::PROGRAM_START,
            window_end_address: Chip8::PROGRAM_START + (25 * 2),
            text: Vec::new()
        }
    }

    pub fn update(&mut self, assets: &Assets, chip8: &Chip8) {
        // If the window is not viewing the current instruction we should shift the window
        // and re-generate the text.
        if chip8.pc < self.window_start_address || chip8.pc > self.window_end_address  {
            self.window_start_address = chip8.pc - 2;
            self.window_end_address = chip8.pc + (25 * 2);

            self.text.clear();

            let opcodes = chip8.opcodes(self.window_start_address, self.window_end_address);
            for (i, (address, opcode)) in opcodes.iter().enumerate() {
                let address_pos = Point2::new(self.x + 10.0, self.y + ((i as f32) * 12.0));
                let address_text = format!("{}", address);
                let address_text = Text::new((address_text, assets.debug_font, 16.0));
                self.text.push((address_pos, address_text));

                let opcode_pos = address_pos + Vector2::new(36.0, 0.0);
                let opcode_text = opcode.to_assembly_name();
                let opcode_text = Text::new((opcode_text, assets.debug_font, 16.0));
                self.text.push((opcode_pos, opcode_text));

                let opcode_arg_pos = opcode_pos + Vector2::new(80.0, 0.0);
                let opcode_arg_text = opcode.to_assembly_args().unwrap_or(String::new());
                let opcode_arg_text = Text::new((opcode_arg_text, assets.debug_font, 16.0));
                self.text.push((opcode_arg_pos, opcode_arg_text));
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for (position, text) in &self.text {
            graphics::queue_text(ctx, text, *position, Some(graphics::WHITE));
        }

        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Nearest)?;

        Ok(())
    }
}
