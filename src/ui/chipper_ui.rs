use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowSetup, WindowMode};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Rect, FilterMode};
use ggez::input::keyboard::{self, KeyCode};
use ggez::timer;

use crate::chip8::{Chip8, Chip8Output};
use crate::ui::{Assets, AssemblyDisplay, Chip8Display, RegisterDisplay};

pub struct ChipperUI {
    chip8: Chip8,
    assets: Assets,
    register_display: RegisterDisplay,
    chip8_display: Chip8Display,
    assembly_window: AssemblyDisplay,
}

impl ChipperUI {
    const WIDTH: f32 = RegisterDisplay::WIDTH + Chip8Display::WIDTH + AssemblyDisplay::WIDTH;
    const HEIGHT: f32 = Chip8Display::HEIGHT;

    pub fn run() {
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("chipper", "Jake Woods")
            .window_setup(WindowSetup::default().title("Chipper"))
            .window_mode(WindowMode::default().dimensions(ChipperUI::WIDTH, ChipperUI::HEIGHT))
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

    pub fn new(ctx: &mut Context) -> ChipperUI {
        graphics::set_default_filter(ctx, FilterMode::Nearest);

        let assets = Assets::load(ctx);
        let chip8 = Chip8::new();
        let register_display = RegisterDisplay::new(0.0, 170.0);
        let chip8_display = Chip8Display::new(ctx, &chip8, RegisterDisplay::WIDTH, 0.0);
        let assembly_window = AssemblyDisplay::new(RegisterDisplay::WIDTH + Chip8Display::WIDTH, 0.0);

        ChipperUI {
            assets,
            chip8,
            register_display,
            chip8_display,
            assembly_window
        }
    }
}

impl EventHandler for ChipperUI {
    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, ChipperUI::WIDTH, ChipperUI::HEIGHT))
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

        let delta_time = timer::delta(ctx);
        let chip8_output = self.chip8.tick(delta_time);

        if chip8_output == Chip8Output::Tick || chip8_output == Chip8Output::Redraw {
            self.register_display.update(ctx, &self.assets, &self.chip8)?;
            self.assembly_window.update(ctx, &self.assets, &self.chip8)?;
        }

        if chip8_output == Chip8Output::Redraw {
            self.chip8_display.update(ctx, &self.chip8)
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        self.chip8_display.draw(ctx)?;
        self.assembly_window.draw(ctx)?;

        self.register_display.draw(ctx)?;

        // Draw code here...
        graphics::present(ctx)
    }
}
