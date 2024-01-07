use num_complex::Complex;

use crate::math::{linear_interpolation, mandelbrot_escape_time, Rgb};
use crate::scene::{Camera, Scene};

/// A Texture maps a (u, v) coordinate on a Surface into a color
pub trait Texture: Sync {
    fn color(&self, scene: &Scene, max_depth: i32, u: f32, v: f32) -> Rgb;
}

/// Infinite checkerboard pattern, alternating between two "sub Textures"
pub struct Checkerboard {
    texture1: Box<dyn Texture>,
    texture2: Box<dyn Texture>,
}

/// Offsets and scales the (u, v) coordinates of another Texture
pub struct CoordinateTransform {
    pub texture: Box<dyn Texture>,
    pub u_offset: f32,
    pub v_offset: f32,
    pub u_scale: f32,
    pub v_scale: f32,
}

/// Texture representing the Mandelbrot set
pub struct MandelbrotSet {
    pub colormap: Vec<Rgb>,
}

/// Texture used to recursively cast a ray into the same scene
pub struct Portal {
    pub camera: Camera,
}

/// A color can be used as a Texture
impl Texture for Rgb {
    fn color(&self, _scene: &Scene, _current_depth: i32, _u: f32, _v: f32) -> Rgb {
        *self
    }
}

impl Checkerboard {
    pub fn new(texture1: Box<dyn Texture>, texture2: Box<dyn Texture>) -> Checkerboard {
        Checkerboard { texture1, texture2 }
    }
}

impl Texture for Checkerboard {
    fn color(&self, scene: &Scene, max_depth: i32, u: f32, v: f32) -> Rgb {
        let square_number = (u.floor() + v.floor()) as i32;
        let square_u = u - u.floor();
        let square_v = v - v.floor();

        match (square_number + 1_000_000) % 2 {
            0 => self.texture1.color(scene, max_depth, square_u, square_v),
            1 => self.texture2.color(scene, max_depth, square_u, square_v),
            _ => unreachable!(),
        }
    }
}

impl Texture for CoordinateTransform {
    fn color(&self, scene: &Scene, max_depth: i32, u: f32, v: f32) -> Rgb {
        let u2 = self.u_scale * (self.u_offset + u);
        let v2 = self.v_scale * (self.v_offset + v);

        self.texture.color(scene, max_depth, u2, v2)
    }
}

impl Texture for MandelbrotSet {
    fn color(&self, _scene: &Scene, _max_depth: i32, u: f32, v: f32) -> Rgb {
        let escape_time = mandelbrot_escape_time(Complex::new(u, v));
        match escape_time {
            Some(t) => {
                let index = t * 0.25;
                linear_interpolation(&self.colormap, index).srgb_to_linear()
            }
            None => Rgb::BLACK,
        }
    }
}

impl Texture for Portal {
    fn color(&self, scene: &Scene, max_depth: i32, u: f32, v: f32) -> Rgb {
        scene.cast(
            self.camera.ray_origin(),
            &self.camera.ray_direction(u, v),
            max_depth - 1,
        )
    }
}
