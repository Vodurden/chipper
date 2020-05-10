mod chipper_ui;
mod chip8_display;
mod assembly_display;
mod assets;

pub use self::chipper_ui::ChipperUI;
pub use self::chip8_display::Chip8Display;
pub use self::assembly_display::AssemblyDisplay;
pub use self::assets::Assets;

use nalgebra;

pub type Vector2 = nalgebra::Vector2<f32>;
pub type Point2 = nalgebra::Point2<f32>;
