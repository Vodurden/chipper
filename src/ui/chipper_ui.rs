use arrayvec::ArrayVec;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Image, DrawParam, Rect};
use ggez::input::keyboard::{KeyCode, KeyMods};

use crate::chip8::Chip8;

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

        chipper_ui.chip8.load_rom_from_file("./roms/PONG").expect("Failed to load ROM");

        // Run!
        match event::run(&mut ctx, &mut event_loop, &mut chipper_ui) {
            Ok(_) => println!("Exited cleanly."),
            Err(e) => println!("Error occured: {}", e)
        }
    }


    pub fn new(_ctx: &mut Context) -> ChipperUI {
        ChipperUI {
            chip8: Chip8::new(),
            frame_buffer: ArrayVec::<[_; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT * 4]>::new(),
        }
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

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Key1 => self.chip8.press_key(0x1),
            KeyCode::Key2 => self.chip8.press_key(0x2),
            KeyCode::Key3 => self.chip8.press_key(0x3),
            KeyCode::Key4 => self.chip8.press_key(0xC),

            KeyCode::Q => self.chip8.press_key(0x4),
            KeyCode::W => self.chip8.press_key(0x5),
            KeyCode::E => self.chip8.press_key(0x6),
            KeyCode::R => self.chip8.press_key(0xD),

            KeyCode::A => self.chip8.press_key(0x7),
            KeyCode::S => self.chip8.press_key(0x8),
            KeyCode::D => self.chip8.press_key(0x9),
            KeyCode::F => self.chip8.press_key(0xE),

            KeyCode::Z => self.chip8.press_key(0xA),
            KeyCode::X => self.chip8.press_key(0x0),
            KeyCode::C => self.chip8.press_key(0xB),
            KeyCode::V => self.chip8.press_key(0xF),

            _ => {}
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Key1 => self.chip8.release_key(0x1),
            KeyCode::Key2 => self.chip8.release_key(0x2),
            KeyCode::Key3 => self.chip8.release_key(0x3),
            KeyCode::Key4 => self.chip8.release_key(0xC),

            KeyCode::Q => self.chip8.release_key(0x4),
            KeyCode::W => self.chip8.release_key(0x5),
            KeyCode::E => self.chip8.release_key(0x6),
            KeyCode::R => self.chip8.release_key(0xD),

            KeyCode::A => self.chip8.release_key(0x7),
            KeyCode::S => self.chip8.release_key(0x8),
            KeyCode::D => self.chip8.release_key(0x9),
            KeyCode::F => self.chip8.release_key(0xE),

            KeyCode::Z => self.chip8.release_key(0xA),
            KeyCode::X => self.chip8.release_key(0x0),
            KeyCode::C => self.chip8.release_key(0xB),
            KeyCode::V => self.chip8.release_key(0xF),

            _ => {}
        }
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        self.chip8.cycle();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        self.refresh_frame_buffer();
        let image = Image::from_rgba8(ctx, 64, 32, &self.frame_buffer)?;
        graphics::draw(ctx, &image, DrawParam::default())?;

        // Draw code here...
        graphics::present(ctx)
    }
}
