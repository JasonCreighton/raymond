use crate::math::{Vec3f, solve_quadratic};

pub trait Surface {
	fn intersection_with_ray(&self, ray_origin: &Vec3f, ray_direction: &Vec3f) -> Option<f32>;
	fn at_point(&self, point_on_surface: &Vec3f) -> SurfaceProperties;
}

#[derive(Debug, Copy, Clone)]
pub struct SurfaceProperties {
	pub normal: Vec3f,
	pub u: f32,
	pub v: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
	center: Vec3f,
	radius: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
	position: Vec3f,
	u_basis: Vec3f,
	v_basis: Vec3f,
	normal: Vec3f,
}

impl Sphere {
	pub fn new(center: &Vec3f, radius: f32) -> Sphere {
		Sphere {
			center: *center,
			radius: radius,
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
		let u = 0.5 + d.y.atan2(d.x) / (2.0 * std::f32::consts::PI);
		let v = 0.5 - d.z.asin() / std::f32::consts::PI;
		
		SurfaceProperties {
			normal: surface_normal,
			u: u,
			v: v,
		}
	}
}

impl Plane {
	pub fn new(position: &Vec3f, u_basis: &Vec3f, v_basis: &Vec3f) -> Plane {
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