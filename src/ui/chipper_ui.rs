use std::thread;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowSetup, WindowMode};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Rect, FilterMode};
use ggez::input::keyboard::{self, KeyCode, KeyMods};
use ggez::timer;
use tinyfiledialogs;

use crate::chip8::{Chip8, Chip8Output};
use crate::ui::{Assets, AssemblyDisplay, Chip8Display, HelpDisplay, RegisterDisplay};

pub struct ChipperUI {
    chip8: Chip8,
    assets: Assets,
    help_display: HelpDisplay,
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

        // Run!
        match event::run(&mut ctx, &mut event_loop, &mut chipper_ui) {
            Ok(_) => println!("Exited cleanly."),
            Err(e) => println!("Error occured: {}", e)
        }
    }

    pub fn new(ctx: &mut Context) -> ChipperUI {
        graphics::set_default_filter(ctx, FilterMode::Nearest);

        let assets = Assets::load(ctx);
        let chip8 = Chip8::new_with_default_rom();
        let help_display = HelpDisplay::new(&assets, 20.0, 0.0);
        let register_display = RegisterDisplay::new(20.0, HelpDisplay::HEIGHT);
        let chip8_display = Chip8Display::new(ctx, &chip8, RegisterDisplay::WIDTH, 0.0);
        let assembly_window = AssemblyDisplay::new(RegisterDisplay::WIDTH + Chip8Display::WIDTH, 0.0);

        ChipperUI {
            assets,
            chip8,
            help_display,
            register_display,
            chip8_display,
            assembly_window
        }
    }

    fn load_rom_from_dialog(&mut self) {
        let current_dir = std::env::current_dir()
            .ok()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or(String::new().into());

        if let Some(file_path) = tinyfiledialogs::open_file_dialog("Choose a Chip 8 ROM", &current_dir, None) {
            self.chip8 = Chip8::new_with_rom_from_file(file_path).expect("Failed to load ROM");
            self.assembly_window.refresh(&self.assets, &self.chip8);
        }
    }

    fn refresh_chip8(&mut self, ctx: &mut Context, chip8_output: Chip8Output) -> GameResult<()> {
        if chip8_output == Chip8Output::Tick || chip8_output == Chip8Output::Redraw {
            self.register_display.update(&self.assets, &self.chip8)?;
            self.assembly_window.update(ctx, &self.assets, &self.chip8)?;
        }

        if chip8_output == Chip8Output::Redraw {
            self.chip8_display.update(ctx, &self.chip8)
        }

        Ok(())
    }

}

impl EventHandler for ChipperUI {
    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, ChipperUI::WIDTH, ChipperUI::HEIGHT))
            .expect("Failed to set screen coordinates");
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::F2 => self.load_rom_from_dialog(),
            KeyCode::F3 => {
                self.load_rom_from_dialog();
                self.chip8.debug_mode = true;
            }
            KeyCode::F5 => self.chip8.debug_mode = !self.chip8.debug_mode,
            KeyCode::F6 => {
                let chip8_output = self.chip8.step();
                self.refresh_chip8(ctx, chip8_output)
                    .expect("Failed to refresh chip8");
            },
            _ => {}
        }

        match (keymods, keycode) {
            (KeyMods::SHIFT, KeyCode::F1) => println!("{}", self.chip8.gfx_to_string()),
            _ => {}
        }
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
        self.refresh_chip8(ctx, chip8_output)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        self.chip8_display.draw(ctx)?;
        self.assembly_window.draw(ctx)?;
        self.help_display.draw(ctx)?;
        self.register_display.draw(ctx)?;

        graphics::present(ctx)?;

        // We don't need to run faster then the chip8 clock speed and
        // we can tolerate longer sleeps by simulating multiple cycles
        // in the same step.
        //
        // This means we can rely on sleep to help avoid hammering the CPU
        thread::sleep(self.chip8.clock_speed);

        Ok(())
    }
}
