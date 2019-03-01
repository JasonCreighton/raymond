mod ppm;
mod math;
mod surface;
mod texture;

use math::Vec3f;
use math::angle_of_reflection;
use surface::*;
use texture::*;

// If we try to trace from the exact position on a surface, sometimes we will
// detect the object that we are on due to floating point rounding issues.
// Therefore, we add a slight bias in the direction of the surface normal to
// avoid this.
const FLOAT_BIAS: f32 = 0.0001;

#[derive(Debug, Copy, Clone)]
struct LightSource {
	dir_to_light: Vec3f,
	intensity: f32,
}

struct VisObj {
	surface: Box<dyn Surface>,
	texture: Box<dyn Texture>,
	reflectivity: f32,
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
		let trace_pos = surface_position.add(&surface_normal.scale(FLOAT_BIAS));
		
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

	fn cast(&self, ray_origin: &Vec3f, ray_direction: &Vec3f, max_depth: i32) -> LinearRGB {
		if max_depth == 0 {
			return self.background;
		}
	
		match self.trace_to_nearest_object(&ray_origin, &ray_direction) {
			Some((vobj, dist)) => {
				let intersection_pos = ray_origin.add(&ray_direction.scale(dist));
				let surf_prop = vobj.surface.at_point(&intersection_pos);
				let light_intensity = self.light_on_surface(&intersection_pos, &surf_prop.normal);
				let vobj_color = vobj.texture.color(surf_prop.u, surf_prop.v);
				
				let reflected_color = if vobj.reflectivity != 0.0 {
					let reflect_ray = angle_of_reflection(&ray_direction, &surf_prop.normal);
					let reflect_origin = intersection_pos.add(&surf_prop.normal.scale(FLOAT_BIAS));
					
					self.cast(&reflect_origin, &reflect_ray, max_depth - 1).scale(vobj.reflectivity)
				} else {
					LinearRGB { red: 0.0, green: 0.0, blue: 0.0 }
				};
				
				vobj_color.scale(light_intensity).add(&reflected_color)				
			}
			None => self.background,
		}
	}
}



fn build_scene() -> Scene {
	let mut scene = Scene {
		background: LinearRGB { red: 0.3, green: 0.5, blue: 0.9 },
		light_sources: Vec::new(),
		objects: Vec::new(),
	};
	
	scene.light_sources.push(LightSource {
		dir_to_light: Vec3f { x: 0.0, y: 10.0, z: 10.0 },
		intensity: 5.0,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere::new(&Vec3f { x: 0.0, y: -1.5, z: 1.5 }, 1.0)),
		texture: Box::new(Checkerboard::new(
			Box::new(LinearRGB { red: 0.0, green: 0.0, blue: 0.5 }),
			Box::new(LinearRGB { red: 0.0, green: 0.5, blue: 0.0 }),
			0.1,
		)),
		reflectivity: 0.0,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere::new(&Vec3f { x: 0.0, y: 1.5, z: 1.5 }, 1.0)),
		texture: Box::new(LinearRGB { red: 0.05, green: 0.0, blue: 0.0 }),
		reflectivity: 0.9,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere::new(&Vec3f { x: 2.5, y: 0.0, z: 1.5 }, 1.0)),
		texture: Box::new(LinearRGB { red: 0.01, green: 0.01, blue: 0.01 }),
		reflectivity: 0.9,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Sphere::new(&Vec3f { x: -2.0, y: 0.0, z: 3.5 }, 1.0)),
		texture: Box::new(LinearRGB { red: 0.3, green: 0.3, blue: 0.1 }),
		reflectivity: 0.0,
	});
	
	scene.objects.push(VisObj {
		surface: Box::new(Plane::new(
			&Vec3f { x: 0.0, y: 0.0, z: 0.0 },
			&Vec3f { x: 1.0, y: 0.0, z: 0.0 },
			&Vec3f { x: 0.0, y: 1.0, z: 0.0 },
		)),
		texture: Box::new(Checkerboard::new(
			Box::new(LinearRGB { red: 0.5, green: 0.5, blue: 0.5 }),
			Box::new(LinearRGB { red: 0.5, green: 0.0, blue: 0.0 }),
			0.5,
		)),
		reflectivity: 0.025,
	});
	
	scene
}

fn main() {
	let width = 1024;
	let height = 768;
	let scene = build_scene();
	let camera = Camera::new(width, height, Vec3f { x: -10.0, y: 0.0, z: 2.0 }, Vec3f { x: 10.0, y: 0.0, z: -1.0 });
	
	let mut image = ppm::PPMWriter::new("test.ppm", width, height);
	
	for y in 0..height {
		for x in 0..width {
			let (red, green, blue) = scene.cast(&camera.ray_origin(), &camera.ray_direction(x, y), 10).gamma_correct();
			image.write(red, green, blue);
		}
	}
}
