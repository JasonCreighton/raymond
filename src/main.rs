//use std::vec;
//use std::dbg;

mod ppm;

#[derive(Debug, Copy, Clone)]
struct Vec3f {
	x: f32,
	y: f32,
	z: f32,
}

#[derive(Debug, Copy, Clone)]
struct Sphere {
	center: Vec3f,
	radius: f32,
}

#[derive(Debug, Copy, Clone)]
struct LinearRGB {
	red: f32,
	green: f32,
	blue: f32,
}

#[derive(Debug, Copy, Clone)]
struct LightSource {
	position: Vec3f,
	intensity: f32,
}

#[derive(Debug)]
struct Scene {
	background: LinearRGB,
	light_sources: Vec<LightSource>,
	spheres: Vec<Sphere>,	
}

#[derive(Debug)]
struct Camera {
	position: Vec3f,
	upper_left: Vec3f,
	delta_x: Vec3f,
	delta_y: Vec3f,
}

impl Camera {
	fn new(width: i32, height: i32, position: Vec3f, direction: Vec3f) -> Camera {
		// TODO: Using cross products like this to means that the camera can't point
		// straight up or straight down, because otherwise the cross with Vec3f::UP
		// yields the zero vector, and then normalizing results in NaNs.
		let zoom = 2.0;
		let delta_x = direction.cross(&Vec3f::UP).normalize().scale(1.0 / (width as f32));
		// NB: delta_y is scaled relative to width as well, to make sure we get square
		// pixels
		let delta_y = direction.cross(&delta_x).normalize().scale(1.0 / (width as f32));
		let upper_left = direction
			.normalize()
			.scale(zoom as f32)
			.add(&delta_x.scale(-(width as f32) / 2.0))
			.add(&delta_y.scale(-(height as f32) / 2.0));

		Camera {
			position,
			upper_left,
			delta_x,
			delta_y,
		}
	}
	
	fn ray_origin(&self) -> &Vec3f {
		&self.position
	}
	
	fn ray_direction(&self, x: i32, y: i32) -> Vec3f {
		self.upper_left
			.add(&self.delta_x.scale(x as f32))
			.add(&self.delta_y.scale(y as f32))
	}
}

impl LinearRGB {
	fn gamma_correct_component(linear_value: f32) -> u8 {
		// Clamp to [0.0, 1.0] and apply gamma correction transfer function
		// (I am not a color space expert and I don't think this is quite correct
		// but it is close enough.)
		(linear_value.max(0.0).min(1.0).powf(1.0/2.2) * 255.0) as u8
	}

	fn gamma_correct(&self) -> (u8, u8, u8) {
		(
			LinearRGB::gamma_correct_component(self.red),
			LinearRGB::gamma_correct_component(self.green),
			LinearRGB::gamma_correct_component(self.blue),
		)
	}
}

impl Vec3f {
	const UP: Vec3f = Vec3f { x: 0.0, y: 0.0, z: 1.0 };

	fn add(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
		
	fn sub(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
	
	fn scale(&self, factor: f32) -> Vec3f {
		Vec3f {
			x: self.x * factor,
			y: self.y * factor,
			z: self.z * factor,
		}
	}
	
	fn normalize(&self) -> Vec3f {
		self.scale(1.0 / self.dot(self).sqrt())
	}
		
	fn dot(&self, other: &Vec3f) -> f32 {
		(self.x * other.x) + (self.y * other.y) + (self.z * other.z)
	}
	
	fn cross(&self, other: &Vec3f) -> Vec3f {
		Vec3f {
			x: self.y*other.z - self.z*other.y,
			y: self.z*other.x - self.x*other.z,
			z: self.x*other.y - self.y*other.x,
		}
	}
}

impl Sphere {
	fn intersects(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<(f32, f32)>
	{
		let origin_minus_center = ray_origin.sub(&self.center);
		let a = ray_direction.dot(&ray_direction); // Shouldn't this always be 1.0???
		let b = 2.0 * ray_direction.dot(&origin_minus_center);
		let c = origin_minus_center.dot(&origin_minus_center) - (self.radius * self.radius);
		
		solve_quadratic(a, b, c)
	}
}

impl Scene {
	fn cast(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> LinearRGB {
		let s1 = &self.spheres[0]; // FIXME
		let light = &self.light_sources[0]; // FIXME
		match s1.intersects(&ray_origin, &ray_direction) {
			Some((t1, t2)) => {
				let intersection_pos = ray_origin.add(&ray_direction.scale(t1.min(t2)));
				let surface_normal = intersection_pos.sub(&s1.center);
				let dir_to_light = light.position.sub(&intersection_pos);
				let intensity = dir_to_light.normalize().dot(&surface_normal.normalize()).max(0.0);
				let obj_color = LinearRGB { red: 0.0, green: 0.0, blue: 0.5 };
				
				LinearRGB { red: 0.0, green: 0.0, blue: 0.1 + (intensity/2.0) }
			}
			None => LinearRGB { red: 0.0, green: 0.2, blue: 0.0 },
		}
	}
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
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

fn build_scene() -> Scene {
	let mut scene = Scene {
		background: LinearRGB { red: 0.0, green: 1.0, blue: 1.0 },
		light_sources: Vec::new(),
		spheres: Vec::new(),
	};
	
	scene.light_sources.push(LightSource {
		position: Vec3f { x: -10.0, y: 0.0, z: 10.0 },
		intensity: 1.0,
	});
	
	scene.spheres.push(Sphere {
		center: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
		radius: 1.0,
	});
	
	scene
}


fn main() {
	let width = 640;
	let height = 480;
	let scene = build_scene();
	let camera = Camera::new(width, height, Vec3f { x: -10.0, y: 0.0, z: 0.0 }, Vec3f { x: 1.0, y: 0.0, z: 0.0 });
	
	let mut image = ppm::PPMWriter::new("test.ppm", width, height);
	
	for y in 0..height {
		for x in 0..width {
			let (red, green, blue) = scene.cast(&camera.ray_origin(), &camera.ray_direction(x, y)).gamma_correct();
			image.write(red, green, blue);
		}
	}
}
