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
struct Plane {
	position: Vec3f,
	u_basis: Vec3f,
	v_basis: Vec3f,
	normal: Vec3f,
}

#[derive(Debug, Copy, Clone)]
struct LinearRGB {
	red: f32,
	green: f32,
	blue: f32,
}

#[derive(Debug, Copy, Clone)]
struct LightSource {
	dir_to_light: Vec3f,
	intensity: f32,
}

struct VisObj {
	surface: Box<dyn Surface>,
	texture: Box<dyn Texture>,
}

struct Scene {
	background: LinearRGB,
	light_sources: Vec<LightSource>,
	objects: Vec<VisObj>,
}

#[derive(Debug)]
struct Camera {
	position: Vec3f,
	upper_left: Vec3f,
	delta_x: Vec3f,
	delta_y: Vec3f,
}

#[derive(Debug, Copy, Clone)]
struct SurfaceProperties {
	normal: Vec3f,
	u: f32,
	v: f32,
}

struct Checkerboard {
	square_size: f32,
	texture1: Box<dyn Texture>,
	texture2: Box<dyn Texture>,
}

trait Texture {
	fn color(&self, u: f32, v: f32) -> LinearRGB;
}

trait Surface {
	fn intersection_with_ray(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<f32>;
	fn at_point(&self, point_on_surface: &Vec3f) -> SurfaceProperties;
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
	
	// TODO: Very similar to Vec3f functionality, and one could imagine use for other
	// methods from Vec3f as well, maybe there is a way to factor out the common methods.
	fn scale(&self, factor: f32) -> LinearRGB {
		LinearRGB {
			red: self.red * factor,
			green: self.green * factor,
			blue: self.blue * factor,
		}
	}
}

impl Texture for LinearRGB {
	fn color(&self, _u: f32, _v: f32) -> LinearRGB {
		*self
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

impl Plane {
	fn new(position: &Vec3f, u_basis: &Vec3f, v_basis: &Vec3f) -> Plane {
		let normal = u_basis.cross(v_basis);
		
		Plane {
			position: *position,
			u_basis: *u_basis,
			v_basis: *v_basis,
			normal: normal,
		}
	}
}

impl Surface for Plane {
	fn intersection_with_ray(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<f32>
	{
		let denom = ray_direction.dot(&self.normal);
		
		if denom.abs() < 0.001 {
			// Basically zero, no intersection
			None
		} else {
			let numer = (self.position.sub(&ray_origin)).dot(&self.normal);
			let d = numer / denom;
			
			if d > 0.0 {
				Some(d)
			} else {
				None
			}
		}		
	}
	
	fn at_point(&self, point_on_surface: &Vec3f) -> SurfaceProperties {
		let vec_within_plane = point_on_surface.sub(&self.position);
		let u = vec_within_plane.dot(&self.u_basis);
		let v = vec_within_plane.dot(&self.v_basis);
		
		SurfaceProperties {
			normal: self.normal,
			u: u,
			v: v,
		}

	}
}

impl Surface for Sphere {
	fn intersection_with_ray(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<f32>
	{
		let origin_minus_center = ray_origin.sub(&self.center);
		let a = ray_direction.dot(&ray_direction); // Shouldn't this always be 1.0???
		let b = 2.0 * ray_direction.dot(&origin_minus_center);
		let c = origin_minus_center.dot(&origin_minus_center) - (self.radius * self.radius);
		
		// TODO: This is a little ugly. We want to max of t1 and t2, but only considering
		// those that are positive, since we don't want to detect objects behind us. Seems
		// like there should be a clearer way to do this.
		match solve_quadratic(a, b, c) {
			Some((t1, t2)) => match (t1 > 0.0, t2 > 0.0) {
				(false, false) => None,
				(false, true)  => Some(t2),
				(true,  false) => Some(t1),
				(true,  true)  => Some(t1.min(t2)),
			},
			None => None,
		}
	}
	
	fn at_point(&self, point_on_surface: &Vec3f) -> SurfaceProperties {
		let d = point_on_surface.sub(&self.center).normalize();
		let surface_normal = point_on_surface.sub(&self.center).normalize();
		//let u = 0.5 + d.z.atan2(d.x) / (2.0 * std::f32::consts::PI);
		//let v = 0.5 - d.y.asin() / std::f32::consts::PI;
		
		let u = 0.5 + d.y.atan2(d.x) / (2.0 * std::f32::consts::PI);
		let v = 0.5 - d.z.asin() / std::f32::consts::PI;
		
		SurfaceProperties {
			normal: surface_normal,
			u: u,
			v: v,
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

impl Scene {
	fn trace_to_nearest_object(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<(&VisObj, f32)>
	{
		self.objects
			.iter()
			// Get a list of intersecting spheres with their distances as a 2-tuple
			.filter_map(|vobj| vobj.surface.intersection_with_ray(&ray_origin, &ray_direction).map(|dist| (vobj, dist)))
			// Select (vobj, distance) 2-tuple with the minimum distance
			.min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
	}
	
	fn light_on_surface(&self, surface_position: &Vec3f, surface_normal: &Vec3f) -> f32 {
		let ambient_light_intensity = 1.0;
		
		// If we try to trace from the exact position on the surface, sometimes we will
		// detect the object that we are on due to floating point rounding issues.
		// Therefore, we add a slight bias in the direction of the surface normal to
		// avoid this.
		let float_bias = 0.001;
		let trace_pos = surface_position.add(&surface_normal.scale(float_bias));
		
		let lambert_light_intensity: f32 = self.light_sources.iter().map(|light_source|
			match self.trace_to_nearest_object(&trace_pos, &light_source.dir_to_light) {
				Some(_) => 0.0, // something is in the way
				None => {
					// There is a path to the light, apply it
					light_source.dir_to_light.normalize().dot(&surface_normal).max(0.0) * light_source.intensity
				}
			})
			.sum();
		
		ambient_light_intensity + lambert_light_intensity
	}

	fn cast(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> LinearRGB {
		match self.trace_to_nearest_object(&ray_origin, &ray_direction) {
			Some((vobj, dist)) => {
				let intersection_pos = ray_origin.add(&ray_direction.scale(dist));
				let surf_prop = vobj.surface.at_point(&intersection_pos);
				let light_intensity = self.light_on_surface(&intersection_pos, &surf_prop.normal);
				let color = vobj.texture.color(surf_prop.u, surf_prop.v);
				
				color.scale(light_intensity)
			}
			None => self.background,
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
		background: LinearRGB { red: 0.0, green: 0.0, blue: 0.0 },
		light_sources: Vec::new(),
		objects: Vec::new(),
	};
	
	scene.light_sources.push(LightSource {
		dir_to_light: Vec3f { x: 0.0, y: 10.0, z: 10.0 },
		intensity: 5.0,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere {
			center: Vec3f { x: 0.0, y: 0.0, z: 1.5 },
			radius: 1.0,
		}),
		texture: Box::new(Checkerboard {
			square_size: 0.1,
			texture1: Box::new(LinearRGB { red: 0.0, green: 0.0, blue: 0.5 }),
			texture2: Box::new(LinearRGB { red: 0.0, green: 0.5, blue: 0.0 }),
		})
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere {
			center: Vec3f { x: 0.0, y: 0.5, z: 0.0 },
			radius: 1.0,
		}),
		texture: Box::new(LinearRGB { red: 0.1, green: 0.0, blue: 0.0 }),
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Plane::new(
			&Vec3f { x: 0.0, y: 0.0, z: 0.0 },
			&Vec3f { x: 1.0, y: 0.0, z: 0.0 },
			&Vec3f { x: 0.0, y: 1.0, z: 0.0 },
		)),
		texture: Box::new(Checkerboard {
			square_size: 0.5,
			texture1: Box::new(LinearRGB { red: 0.5, green: 0.5, blue: 0.5 }),
			texture2: Box::new(LinearRGB { red: 0.5, green: 0.0, blue: 0.0 }),
		}),
	});
	
	scene
}


fn main() {
	let width = 640;
	let height = 480;
	let scene = build_scene();
	let camera = Camera::new(width, height, Vec3f { x: -10.0, y: 0.0, z: 2.0 }, Vec3f { x: 10.0, y: 0.0, z: -1.0 });
	
	let mut image = ppm::PPMWriter::new("test.ppm", width, height);
	
	for y in 0..height {
		for x in 0..width {
			let (red, green, blue) = scene.cast(&camera.ray_origin(), &camera.ray_direction(x, y)).gamma_correct();
			image.write(red, green, blue);
		}
	}
}
