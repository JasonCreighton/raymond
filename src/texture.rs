pub trait Texture : Sync {
	fn color(&self, u: f32, v: f32) -> LinearRGB;
}

#[derive(Debug, Copy, Clone)]
pub struct LinearRGB {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
}

pub struct Checkerboard {	
	texture1: Box<dyn Texture>,
	texture2: Box<dyn Texture>,
	square_size: f32,
}

impl LinearRGB {
	fn gamma_correct_component(linear_value: f32) -> u8 {
		// Clamp to [0.0, 1.0] and apply gamma correction transfer function
		// (I am not a color space expert and I don't think this is quite correct
		// but it is close enough.)
		(linear_value.max(0.0).min(1.0).powf(1.0/2.2) * 255.0) as u8
	}

	pub fn gamma_correct(&self) -> (u8, u8, u8) {
		(
			LinearRGB::gamma_correct_component(self.red),
			LinearRGB::gamma_correct_component(self.green),
			LinearRGB::gamma_correct_component(self.blue),
		)
	}
	
	// TODO: Very similar to Vec3f functionality, and one could imagine use for other
	// methods from Vec3f as well, maybe there is a way to factor out the common methods.
	pub fn scale(&self, factor: f32) -> LinearRGB {
		LinearRGB {
			red: self.red * factor,
			green: self.green * factor,
			blue: self.blue * factor,
		}
	}
	pub fn add(&self, other: &LinearRGB) -> LinearRGB {
		LinearRGB {
			red: self.red + other.red,
			green: self.green + other.green,
			blue: self.blue + other.blue,
		}
	}
}

impl Texture for LinearRGB {
	fn color(&self, _u: f32, _v: f32) -> LinearRGB {
		*self
	}
}

impl Checkerboard {
	pub fn new(texture1: Box<dyn Texture>, texture2: Box<dyn Texture>, square_size: f32) -> Checkerboard {
		Checkerboard {
			texture1: texture1,
			texture2: texture2,
			square_size: square_size,
		}
	}
}

impl Texture for Checkerboard {
	fn color(&self, u: f32, v: f32) -> LinearRGB {
		let scaled_u = u / self.square_size;
		let scaled_v = v / self.square_size;
		let square_number = (scaled_u.floor() + scaled_v.floor()) as i32;
		let square_u = scaled_u - u.floor();
		let square_v = scaled_v - v.floor();
		
		match (square_number + 1000000) % 2 {
			0 => self.texture1.color(square_u, square_v),
			1 => self.texture2.color(square_u, square_v),
			_ => unreachable!(),
		}
	}
}