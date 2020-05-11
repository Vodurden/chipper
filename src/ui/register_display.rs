use ggez::{Context, GameResult};
use ggez::graphics::{self, Text, DrawParam, DrawMode, FilterMode, Rect, Mesh, Color};

use crate::chip8::Chip8;
use crate::ui::{Assets, Point2, Vector2};

pub struct RegisterDisplay {
    /// The horizontal position of this display relative to the main window
    x: f32,

    /// The vertical position of this display relative to the main window
    y: f32,

    text: Vec<(Point2, Text)>
}

impl RegisterDisplay {
    pub const WIDTH: f32 = 150.0;
    pub const HEIGHT: f32 = 320.0;

    const LINE_HEIGHT: f32 = 12.0;
    const FONT_SIZE: f32 = 16.0;

    pub fn new(x: f32, y: f32) -> RegisterDisplay {
        RegisterDisplay { x, y, text: Vec::new() }
    }

    pub fn update(&mut self, ctx: &mut Context, assets: &Assets, chip8: &Chip8) -> GameResult<()> {
        self.text.clear();

        // Show `PC` and `I`
        self.push_line_col(assets, 0, 0, "PC".to_string(), format!("{:03X}", chip8.pc));
        self.push_line_col(assets, 1, 0, "IX".to_string(), format!("{:03X}", chip8.i));

        // Generate `V` registers
        let v_line_offset = 2;
        for (i, x) in (0..8).enumerate() {
            self.push_line_col(assets, 0, v_line_offset + i as u8, format!("V{:X}", i), format!("{:02X}", chip8.v[x]));
        }
        for (i, x) in (8..16).enumerate() {
            self.push_line_col(assets, 1, v_line_offset + i as u8, format!("V{:X}", i + 8), format!("{:02X}", chip8.v[x]));
        }

        // Show `DT` and `ST`
        self.push_line_col(assets, 0, 11, "DT".to_string(), format!("{:02X}", chip8.delay_timer));
        self.push_line_col(assets, 1, 11, "ST".to_string(), format!("{:02X}", chip8.sound_timer));

        Ok(())
    }

    fn push_line_col(&mut self, assets: &Assets, col: u8, line: u8, key: String, value: String) {
        let key_x = self.x + (col as f32 * RegisterDisplay::WIDTH / 2.0);
        let sep_x = key_x + 25.0;
        let value_x = sep_x + 15.0;
        let line_y = self.y + (line as f32) * RegisterDisplay::LINE_HEIGHT;

        let key_pos = Point2::new(key_x, line_y);
        let sep_pos = Point2::new(sep_x, line_y);
        let value_pos = Point2::new(value_x, line_y);

        let key_text = Text::new((key, assets.debug_font, RegisterDisplay::FONT_SIZE));
        let sep_text = Text::new(("=", assets.debug_font, RegisterDisplay::FONT_SIZE));
        let value_text = Text::new((value, assets.debug_font, RegisterDisplay::FONT_SIZE));

        self.text.push((key_pos, key_text));
        self.text.push((sep_pos, sep_text));
        self.text.push((value_pos, value_text));
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for (position, text) in &self.text {
            graphics::queue_text(ctx, text, *position, Some(graphics::WHITE));
        }
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Nearest)?;

        Ok(())
    }
}
