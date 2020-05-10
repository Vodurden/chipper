use ggez::Context;
use ggez::graphics::Font;

pub struct Assets {
    pub debug_font: Font
}

impl Assets {
    pub fn load(ctx: &mut Context) -> Assets {
        let debug_font_bytes = std::include_bytes!("../../resources/font/source-code-pro.regular.ttf");

        let debug_font = Font::new_glyph_font_bytes(ctx, debug_font_bytes)
            .expect("Failed to load debug font");

        Assets {
            debug_font
        }
    }
}
