use arrayvec::ArrayVec;
use std::fmt;

/// `Gpu` represents the Chip-8 display. The Chip-8 has a 64x32 display consisting of an
/// empty colour and a filled colour.
///
/// If `pixels[y * Chip8::SCREEN_WIDTH + x]` is `0x0` then the pixel at `(x, y)` should be empty,
/// otherwise it should be filled.
///
/// The specific colour of "filled" and "empty" should be defined by the rendering system.
pub struct Gpu {
    pixels: [u8; Gpu::SCREEN_PIXELS]
}

pub enum DrawResult {
    NoCollision,
    Collision
}

impl Gpu {
    pub const SCREEN_WIDTH: usize = 64;
    pub const SCREEN_HEIGHT: usize = 32;
    pub const SCREEN_PIXELS: usize = Gpu::SCREEN_WIDTH * Gpu::SCREEN_HEIGHT;

    pub const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    pub const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

    pub fn new() -> Gpu {
        Gpu {
            pixels: [0; Gpu::SCREEN_PIXELS]
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [0; Gpu::SCREEN_PIXELS];
    }

    pub fn pixel(&mut self, x: usize, y: usize) -> &mut u8 {
        &mut self.pixels[(y * Gpu::SCREEN_WIDTH) + x]
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: Vec<u8>) -> DrawResult {
        let mut draw_result: DrawResult = DrawResult::NoCollision;

        for (pixel_y, row_sprite) in sprite.iter().enumerate() {
            let y = (y + pixel_y) as usize % Gpu::SCREEN_HEIGHT;

            for pixel_x in 0..8 {
                let bit = (row_sprite >> (7 - pixel_x)) & 0x1;
                if bit != 0 {
                    let x = (x + pixel_x) as usize % Gpu::SCREEN_WIDTH;
                    let pixel = self.pixel(x, y);
                    if *pixel == 1 {
                        draw_result = DrawResult::Collision;
                    }

                    *pixel ^= 1;
                }
            }
        }

        draw_result
    }

    /// Convert the current display to a RGBA texture.
    ///
    /// Arguments:
    ///
    /// * `filled`: The RGBA value to use for filled pixels
    /// * `empty`: The RGBA value to use for empty pixels
    ///
    pub fn to_rgba(
        &self,
        empty: [u8; 4],
        filled: [u8; 4],
    ) -> ArrayVec<[u8; Gpu::SCREEN_PIXELS * 4]> {
        self.pixels.iter().flat_map(|pixel| {
            match pixel {
                0 => ArrayVec::from(empty),
                _ => ArrayVec::from(filled),
            }
        }).collect()
    }

    pub fn to_gfx_slice(&self, x_start: u8, columns: u8, y_start: u8, rows: u8) -> Vec<Vec<u8>> {
        let mut gfx_slice = Vec::new();

        for y in y_start..(y_start + rows) {
            let mut row = Vec::new();

            for x in x_start..(x_start + columns) {
                let y = y as usize;
                let x = x as usize;
                row.push(self.pixels[y * Gpu::SCREEN_WIDTH + x] as u8);
            }

            gfx_slice.push(row);
        }

        gfx_slice

    }
}

impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut row = 0;
        loop {
            if row > (2048 - 64) { break; }

            let s: String = self.pixels[row..row+64]
                .into_iter()
                .map(|x| ('0' as u8) + x)
                .map(|x| x as char)
                .collect();
            f.write_str(&s)?;
            f.write_str("\n")?;

            row += 64;
        }

        Ok(())
    }
}
