use arrayvec::ArrayVec;
use ggez::{Context, GameResult};
use ggez::graphics::{self, Image, DrawParam, FilterMode};

use crate::chip8::Chip8;
use crate::ui::Vector2;


/// Displays a Chip8 device in a 640x320 area.
pub struct Chip8Display {

    /// `display_image` holds the texture derived from the Chip-8 graphics memory.
    ///
    /// We need to refresh `display_image` whenever `Chip8` executes `Opcode::Draw`.
    /// Otherwise we can just keep rendering this texture until something changes.
    display_image: Image
}

impl Chip8Display {
    pub fn new(ctx: &mut Context, chip8: &Chip8) -> Chip8Display {
        let display_image = Chip8Display::generate_display_image(ctx, chip8);

        Chip8Display { display_image }
    }

    pub fn on_chip8_draw(&mut self, ctx: &mut Context, chip8: &Chip8) {
        self.display_image = Chip8Display::generate_display_image(ctx, chip8);
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let draw_params = DrawParam::default().scale(Vector2::new(10.0, 10.0));
        graphics::draw(ctx, &self.display_image, draw_params)?;

        Ok(())
    }

    fn generate_display_image(ctx: &mut Context, chip8: &Chip8) -> Image {
        let frame_buffer: ArrayVec::<[_; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT * 4]> =
            chip8.gfx.iter().flat_map(|pixel| {
                match pixel {
                    0 => vec![0x0, 0x0, 0x0, 0x0],
                    _ => vec![0xFF, 0xFF, 0xFF, 0xFF],
                }
            }).collect();

        let mut image = Image::from_rgba8(ctx, 64, 32, &frame_buffer)
            .expect("Failed to generate frame buffer");

        image.set_filter(FilterMode::Nearest);

        image
    }
}