/// 3-D vector or position
#[derive(Debug, Copy, Clone)]
pub struct Vec3f {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

/// "Linear" RGB value. (ie, SRGB without gamma correction)
/// Component values fall in [0.0, 1.0]
#[derive(Debug, Copy, Clone)]
pub struct LinearRGB {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
}

impl Vec3f {
	pub const UP: Vec3f = Vec3f { x: 0.0, y: 0.0, z: 1.0 };

	pub fn add(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
		
	pub fn sub(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
	
	pub fn scale(&self, factor: f32) -> Vec3f {
		Vec3f {
			x: self.x * factor,
			y: self.y * factor,
			z: self.z * factor,
		}
	}
	
	pub fn normalize(&self) -> Vec3f {
		self.scale(1.0 / self.dot(self).sqrt())
	}
		
	pub fn dot(&self, other: &Vec3f) -> f32 {
		(self.x * other.x) + (self.y * other.y) + (self.z * other.z)
	}
	
	pub fn cross(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.y*other.z - self.z*other.y,
			y: self.z*other.x - self.x*other.z,
			z: self.x*other.y - self.y*other.x,
		}
	}
}

impl LinearRGB {
	fn gamma_correct_component(linear_value: f32) -> u8 {
		// Clamp to [0.0, 1.0] and apply gamma correction transfer function
		// (I am not a color space expert and I don't think this is quite correct
		// but it is close enough.)
		(linear_value.max(0.0).min(1.0).powf(1.0/2.2) * 255.0) as u8
	}
	
	/// Produce a 24-bit SRGB value
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

/// Finds the roots of the equation ax^2 + bx + c = 0. Returns None if there is
/// no solution, 
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
	let discriminant = (b*b) - 4.0*a*c;
	// The single solution case tends to be degenerate, we only find the two solution case
	if discriminant > 0.0 {
		let scale = 1.0 / (2.0 * a);
		let x = -b * scale;
		let delta = discriminant.sqrt() * scale;
		
		Some((x + delta, x - delta))
	} else {
		None
	}
}

/// Finds the angle of reflection of an incident ray against a surface with the
/// normal vector.
pub fn angle_of_reflection(incident: &Vec3f, normal: &Vec3f) -> Vec3f {
	incident.sub(&normal.scale(2.0 * incident.dot(normal)))
}

/// Generates a gaussian shaped filter for, eg, a Gaussian blur
pub fn gaussian_kernel(sigma: f32) -> Vec<f32> {
	let half_kernel_length = (sigma * 3.0).ceil() as i32;
	// We always use a symmetric, odd-length kernel
	let kernel_length = (half_kernel_length * 2) + 1;
	
	let kernel_scale_factor = 1.0/(2.0 * std::f32::consts::PI * sigma * sigma).sqrt();
	
	(0..kernel_length)
		.map(|i| (i - half_kernel_length) as f32)
		.map(|x| kernel_scale_factor * (-(x * x)/(2.0 * sigma * sigma)).exp())
		.collect()
}

/// Performs a two dimensional convolution against the provided image and returns
/// a new image. For a W by H image with kernel length K and decimation factor D, the
/// output dimensions will be (W - (K - 1))/D by (H - (K - 1))/D
pub fn convolve_2d(image: &Vec<Vec<LinearRGB>>, kernel: &Vec<f32>, decimation_factor: i32) -> Vec<Vec<LinearRGB>> {
	// We convolve & transpose twice, which results in an untransposed image.
	let horizontally_convolved = convolve_and_transpose(image, &kernel, decimation_factor);
	let vertically_convolved = convolve_and_transpose(&horizontally_convolved, &kernel, decimation_factor);
			
	vertically_convolved
}

/// Convolves the given kernel across the image horizontally, and returns a
/// transposed image, optionally decimating.
///
/// The transposition is intended to allow for the function to easily be
/// applied twice to an image to result in a 2D convolution, see convolve_2d.
fn convolve_and_transpose(image: &Vec<Vec<LinearRGB>>, kernel: &Vec<f32>, decimation_factor: i32) -> Vec<Vec<LinearRGB>> {
	let input_width = image[0].len();
	let input_height = image.len();
	
	// Make sure that the vec of vecs is "square"
	debug_assert!(image.iter().all(|scanline| scanline.len() == input_width));
	
	let kernel_length = kernel.len();
	let output_width = input_height;
	let output_height = (input_width - (kernel_length - 1)) / (decimation_factor as usize);
	let mut output_image : Vec<Vec<LinearRGB>> = (0..output_height).map(|_| Vec::with_capacity(output_width)).collect();
			
	let zero_color = LinearRGB { red: 0.0, green: 0.0, blue: 0.0 };
	
	for out_x in 0..output_width {
		for out_y in 0..output_height {
			let in_x = out_y * (decimation_factor as usize);
			let in_y = out_x;
			
			let convolved_pixel = image[in_y][in_x..(in_x+kernel_length)]
				.iter()
				.zip(kernel)
				.map(|(color, coef)| color.scale(*coef))
				.fold(zero_color, |acc, color| acc.add(&color));
			
			// Essentially what we are doing here is:
			//
			//     output_image[out_y][out_x] = convolved_pixel
			//
			// ...except the vector is empty when we start out, so we have to push()
			// instead.
			debug_assert!(out_x == output_image[out_y].len());
			output_image[out_y].push(convolved_pixel);
		}
	}
	
	// If we did everything right, the output vec of vecs should be the right
	// dimensions
	debug_assert!(output_image.len() == output_height);
	debug_assert!(output_image.iter().all(|scanline| scanline.len() == output_width));
	
	output_image
}

