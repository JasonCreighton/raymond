use crate::math::LinearRGB;

/// A Texture maps a (u, v) coordinate on a Surface into a color
pub trait Texture : Sync {
	fn color(&self, u: f32, v: f32) -> LinearRGB;
}

/// Infinite checkerboard pattern, alternating between two "sub Textures"
pub struct Checkerboard {	
	texture1: Box<dyn Texture>,
	texture2: Box<dyn Texture>,
	square_size: f32,
}

/// A color can be used as a Texture
impl Texture for LinearRGB {
	fn color(&self, _u: f32, _v: f32) -> LinearRGB {
		*self
	}
}

impl Checkerboard {
	pub fn new(texture1: Box<dyn Texture>, texture2: Box<dyn Texture>, square_size: f32) -> Checkerboard {
		Checkerboard { texture1, texture2, square_size }
	}
}

impl Texture for Checkerboard {
	fn color(&self, u: f32, v: f32) -> LinearRGB {
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