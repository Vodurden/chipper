use ggez::{Context, GameResult};
use ggez::graphics::{self, Text, DrawParam, DrawMode, FilterMode, Rect, Mesh, Color};

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

    pc_highlight: Option<Mesh>,
}

impl AssemblyDisplay {
    pub const WIDTH: f32 = 220.0;
    pub const HEIGHT: f32 = 320.0;

    const NUM_LINES: u16 = (AssemblyDisplay::HEIGHT / AssemblyDisplay::LINE_HEIGHT) as u16 - 1;
    const LINE_HEIGHT: f32 = 12.0;
    const FONT_SIZE: f32 = 16.0;
    const PADDING_LEFT: f32 = 10.0;

    pub fn new(x: f32, y: f32) -> AssemblyDisplay {
        AssemblyDisplay {
            x,
            y,
            window_start_address: Chip8::PROGRAM_START,
            window_end_address: Chip8::PROGRAM_START + (AssemblyDisplay::NUM_LINES * 2),
            text: Vec::new(),
            pc_highlight: None,
        }
    }

    pub fn update(&mut self, ctx: &mut Context, assets: &Assets, chip8: &Chip8) -> GameResult<()> {
        // If the window is not viewing the current instruction we should shift the window
        // and re-generate the text.
        if self.text.is_empty() || chip8.pc < self.window_start_address || chip8.pc > self.window_end_address  {
            self.window_start_address = chip8.pc - 2;
            self.window_end_address = chip8.pc + (AssemblyDisplay::NUM_LINES * 2);

            self.text.clear();

            let opcodes = chip8.opcodes(self.window_start_address, self.window_end_address);
            for (i, (address, opcode)) in opcodes.iter().enumerate() {
                let origin = Point2::new(
                    self.x + AssemblyDisplay::PADDING_LEFT,
                    self.y + ((i as f32) * AssemblyDisplay::LINE_HEIGHT)
                );

                let address_pos = origin;
                let address_text = format!("{}", address);
                let address_text = Text::new((address_text, assets.debug_font, AssemblyDisplay::FONT_SIZE));
                self.text.push((address_pos, address_text));

                let opcode_pos = address_pos + Vector2::new(36.0, 0.0);
                let opcode_text = opcode.to_assembly_name();
                let opcode_text = Text::new((opcode_text, assets.debug_font, AssemblyDisplay::FONT_SIZE));
                self.text.push((opcode_pos, opcode_text));

                let opcode_arg_pos = opcode_pos + Vector2::new(80.0, 0.0);
                let opcode_arg_text = opcode.to_assembly_args().unwrap_or(String::new());
                let opcode_arg_text = Text::new((opcode_arg_text, assets.debug_font, AssemblyDisplay::FONT_SIZE));
                self.text.push((opcode_arg_pos, opcode_arg_text));
            }
        }

        let pc_window_index = (chip8.pc - self.window_start_address) / 2;
        let pc_pos = Point2::new(self.x + AssemblyDisplay::PADDING_LEFT, self.y + pc_window_index as f32 * AssemblyDisplay::LINE_HEIGHT);
        let rect = Rect::new(pc_pos.x, pc_pos.y, AssemblyDisplay::WIDTH, AssemblyDisplay::LINE_HEIGHT);
        let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from_rgb(0xFF, 0x00, 0x00))?;
        self.pc_highlight = Some(rect);

        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if let Some(pc_highlight) = &self.pc_highlight {
            graphics::draw(ctx, pc_highlight, DrawParam::default())?;
        }

        for (position, text) in &self.text {
            graphics::queue_text(ctx, text, *position, Some(graphics::WHITE));
        }

        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Nearest)?;

        Ok(())
    }
}
