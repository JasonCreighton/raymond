use num_complex::Complex;

use crate::math::{linear_interpolation, mandelbrot_escape_time, RGB};

/// A Texture maps a (u, v) coordinate on a Surface into a color
pub trait Texture: Sync {
    fn color(&self, u: f32, v: f32) -> RGB;
}

/// Infinite checkerboard pattern, alternating between two "sub Textures"
pub struct Checkerboard {
    texture1: Box<dyn Texture>,
    texture2: Box<dyn Texture>,
    square_size: f32,
}

/// Texture representing the Mandelbrot set
pub struct MandelbrotSet {
    pub colormap: Vec<RGB>,
}

/// A color can be used as a Texture
impl Texture for RGB {
    fn color(&self, _u: f32, _v: f32) -> RGB {
        *self
    }
}

impl Checkerboard {
    pub fn new(
        texture1: Box<dyn Texture>,
        texture2: Box<dyn Texture>,
        square_size: f32,
    ) -> Checkerboard {
        Checkerboard {
            texture1,
            texture2,
            square_size,
        }
    }
}

impl Texture for Checkerboard {
    fn color(&self, u: f32, v: f32) -> RGB {
        let scaled_u = u / self.square_size;
        let scaled_v = v / self.square_size;
        let square_number = (scaled_u.floor() + scaled_v.floor()) as i32;
        let square_u = scaled_u - u.floor();
        let square_v = scaled_v - v.floor();

        match (square_number + 1_000_000) % 2 {
            0 => self.texture1.color(square_u, square_v),
            1 => self.texture2.color(square_u, square_v),
            _ => unreachable!(),
        }
    }
}

impl Texture for MandelbrotSet {
    fn color(&self, u: f32, v: f32) -> RGB {
        let escape_time = mandelbrot_escape_time(Complex::new(u * 0.5, v * 0.5));
        match escape_time {
            Some(t) => {
                let index = t * 0.25;
                linear_interpolation(&self.colormap, index).srgb_to_linear()
            }
            None => RGB::BLACK,
        }
    }
}
