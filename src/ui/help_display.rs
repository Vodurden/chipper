use ggez::{Context, GameResult};
use ggez::graphics::{self, Text, DrawParam, FilterMode};

use crate::ui::{Assets, Chip8Display, Point2};

pub struct HelpDisplay {
    text: Vec<(Point2, Text)>
}

impl HelpDisplay {
    pub const SCALE: f32 = Chip8Display::SCALE;
    #[allow(dead_code)]
    pub const WIDTH: f32 = 15.0 * HelpDisplay::SCALE;
    pub const HEIGHT: f32 = 15.6 * HelpDisplay::SCALE;

    const LINE_HEIGHT: f32 = 1.2 * HelpDisplay::SCALE;
    const FONT_SIZE: f32 = 1.6 * HelpDisplay::SCALE;

    pub fn new(assets: &Assets, x: f32, y: f32) -> HelpDisplay {
        // Horrible spacing to make things line up properly. For some reason
        // the font doesn't line up even though it _should_ be monospace.
        let lines = vec![
            "    Chipper by Jake Woods",
            "",
            "F2 = Load ROM",
            "F5 = Pause/Resume Game",
            "F6 = Step (When Paused)",
            "",
            "                 Controls",
            "       KEYBD                CHIP8",
            "       1  2 3 4    ==>    1  2 3 C",
            "       Q W E R    ==>    4 5 6 D",
            "       A S D F    ==>    7 8 9 E",
            "       Z X C V    ==>    A 0 B F",
        ];

        let mut text = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let line_y = y + (i as f32 * HelpDisplay::LINE_HEIGHT);
            let line_pos = Point2::new(x, line_y);

            let line_text = Text::new((line.to_string(), assets.debug_font, HelpDisplay::FONT_SIZE));

            text.push((line_pos, line_text));
        }

        HelpDisplay { text }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for (position, text) in &self.text {
            graphics::queue_text(ctx, text, *position, Some(graphics::WHITE));
        }
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Nearest)?;

        Ok(())
    }
}
