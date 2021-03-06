use ggez::{Context, GameResult};
use ggez::graphics::{self, Text, DrawParam, FilterMode};

use crate::chip8::Chip8;
use crate::ui::{Assets, Chip8Display, Point2};

pub struct RegisterDisplay {
    /// The horizontal position of this display relative to the main window
    x: f32,

    /// The vertical position of this display relative to the main window
    y: f32,

    text: Vec<(Point2, Text)>
}

impl RegisterDisplay {
    pub const SCALE: f32 = Chip8Display::SCALE;
    pub const WIDTH: f32 = 22.0 * RegisterDisplay::SCALE;

    #[allow(dead_code)]
    pub const HEIGHT: f32 = 32.0 * RegisterDisplay::SCALE;

    const LINE_HEIGHT: f32 = 1.2 * RegisterDisplay::SCALE;
    const FONT_SIZE: f32 = 1.6 * RegisterDisplay::SCALE;

    const KEY_X_OFFSET: f32 = 0.0 * RegisterDisplay::SCALE;
    const SEP_X_OFFSET: f32 = 2.5 * RegisterDisplay::SCALE;
    const VALUE_X_OFFSET: f32 = 1.5 * RegisterDisplay::SCALE;

    pub fn new(x: f32, y: f32) -> RegisterDisplay {
        RegisterDisplay { x, y, text: Vec::new() }
    }

    pub fn update(&mut self, assets: &Assets, chip8: &Chip8) -> GameResult<()> {
        self.text.clear();

        let header_pos = Point2::new(self.x + 50.0, self.y);
        let header_text = Text::new(("Registers".to_string(), assets.debug_font, RegisterDisplay::FONT_SIZE));
        self.text.push((header_pos, header_text));

        // Show `PC` and `I`
        self.push_line_col(assets, 0, 2, "PC".to_string(), format!("{:03X}", chip8.pc));
        self.push_line_col(assets, 1, 2, "IX".to_string(), format!("{:03X}", chip8.i));

        // Show `DT` and `ST`
        self.push_line_col(assets, 0, 3, "DT".to_string(), format!("{:02X}", chip8.delay_timer));
        self.push_line_col(assets, 1, 3, "ST".to_string(), format!("{:02X}", chip8.sound_timer));

        // Generate `V` registers
        let v_line_offset = 5;
        for (i, x) in (0..8).enumerate() {
            self.push_line_col(assets, 0, v_line_offset + i as u8, format!("V{:X}", i), format!("{:02X}", chip8.v[x]));
        }
        for (i, x) in (8..16).enumerate() {
            self.push_line_col(assets, 1, v_line_offset + i as u8, format!("V{:X}", i + 8), format!("{:02X}", chip8.v[x]));
        }

        Ok(())
    }

    fn push_line_col(&mut self, assets: &Assets, col: u8, line: u8, key: String, value: String) {
        let key_x = self.x + (col as f32 * RegisterDisplay::WIDTH / 2.0) + RegisterDisplay::KEY_X_OFFSET;
        let sep_x = key_x + RegisterDisplay::SEP_X_OFFSET;
        let value_x = sep_x + RegisterDisplay::VALUE_X_OFFSET;
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
