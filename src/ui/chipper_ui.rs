use arrayvec::ArrayVec;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Image, DrawParam, Rect, FilterMode};
use ggez::input::keyboard::{self, KeyCode, KeyMods};

use crate::chip8::{Chip8, Chip8Output, Opcode};

pub struct ChipperUI {
    chip8: Chip8,

    /// `frame_buffer` holds the texture derived from the Chip-8 graphics memory.
    ///
    /// `frame_buffer` is an 8-bit RGBA array. This means 8 bits for red, green, blue and alpha
    /// respectively.
    ///
    /// We need to refresh `frame_buffer` whenever `Chip8` executes `Opcode::Draw`. Otherwise
    /// we can just keep rendering this texture until something changes.
    frame_buffer: ArrayVec<[u8; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT * 4]>,
}

impl ChipperUI {
    pub fn run() {
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("chipper", "Jake Woods")
            .build()
            .expect("aieee, could not create ggez context!");

        // Create an instance of your event handler.
        // Usually, you should provide it with the Context object to
        // use when setting your game up.
        let mut chipper_ui = ChipperUI::new(&mut ctx);

        chipper_ui.chip8.load_rom_from_file("./roms/BRIX").expect("Failed to load ROM");

        // Run!
        match event::run(&mut ctx, &mut event_loop, &mut chipper_ui) {
            Ok(_) => println!("Exited cleanly."),
            Err(e) => println!("Error occured: {}", e)
        }
    }


    pub fn new(_ctx: &mut Context) -> ChipperUI {
        let mut chipper_ui = ChipperUI {
            chip8: Chip8::new(),
            frame_buffer: ArrayVec::<[_; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT * 4]>::new(),
        };

        chipper_ui.refresh_frame_buffer();

        chipper_ui
    }

    fn refresh_frame_buffer(&mut self) {
        self.frame_buffer = self.chip8.gfx.iter().flat_map(|pixel| {
            match pixel {
                0 => vec![0x0, 0x0, 0x0, 0x0],
                _ => vec![0xFF, 0xFF, 0xFF, 0xFF],
            }
        }).collect();
    }
}

impl EventHandler for ChipperUI {
    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, 64.0, 32.0))
            .expect("Failed to set screen coordinates");
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.chip8.key(0x1, keyboard::is_key_pressed(ctx, KeyCode::Key1));
        self.chip8.key(0x2, keyboard::is_key_pressed(ctx, KeyCode::Key2));
        self.chip8.key(0x3, keyboard::is_key_pressed(ctx, KeyCode::Key3));
        self.chip8.key(0xC, keyboard::is_key_pressed(ctx, KeyCode::Key4));

        self.chip8.key(0x4, keyboard::is_key_pressed(ctx, KeyCode::Q));
        self.chip8.key(0x5, keyboard::is_key_pressed(ctx, KeyCode::W));
        self.chip8.key(0x6, keyboard::is_key_pressed(ctx, KeyCode::E));
        self.chip8.key(0xD, keyboard::is_key_pressed(ctx, KeyCode::R));

        self.chip8.key(0x7, keyboard::is_key_pressed(ctx, KeyCode::A));
        self.chip8.key(0x8, keyboard::is_key_pressed(ctx, KeyCode::S));
        self.chip8.key(0x9, keyboard::is_key_pressed(ctx, KeyCode::D));
        self.chip8.key(0xE, keyboard::is_key_pressed(ctx, KeyCode::F));

        self.chip8.key(0xA, keyboard::is_key_pressed(ctx, KeyCode::Z));
        self.chip8.key(0x0, keyboard::is_key_pressed(ctx, KeyCode::X));
        self.chip8.key(0xB, keyboard::is_key_pressed(ctx, KeyCode::C));
        self.chip8.key(0xF, keyboard::is_key_pressed(ctx, KeyCode::V));

        let chip8Output = self.chip8.cycle();
        match chip8Output {
            Chip8Output::Redraw => self.refresh_frame_buffer(),
            Chip8Output::None => {}
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        graphics::set_default_filter(ctx, FilterMode::Nearest);

        let image = Image::from_rgba8(ctx, 64, 32, &self.frame_buffer)?;
        graphics::draw(ctx, &image, DrawParam::default())?;

        // Draw code here...
        graphics::present(ctx)
    }
}
