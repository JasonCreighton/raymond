use rayon::prelude::*;

use crate::math::{angle_of_reflection, convolve_2d, gaussian_kernel, Vec3f, RGB};
use crate::surface::Surface;
use crate::texture::Texture;
use crate::util::Array2D;

// If we try to trace from the exact position on a surface, sometimes we will
// detect the object that we are on due to floating point rounding issues.
// Therefore, we add a slight bias in the direction of the surface normal to
// avoid this.
const FLOAT_BIAS: f32 = 0.001;

#[derive(Debug, Copy, Clone)]
pub struct LightSource {
    pub dir_to_light: Vec3f,
    pub intensity: f32,
}

pub struct VisObj {
    pub surface: Box<dyn Surface>,
    pub texture: Box<dyn Texture>,
    pub reflectivity: f32,
}

pub struct Scene {
    pub background: RGB,
    pub ambient_light_intensity: f32,
    pub light_sources: Vec<LightSource>,
    pub objects: Vec<VisObj>,
}

#[derive(Debug, Clone)]
pub struct Camera {
    position: Vec3f,
    direction: Vec3f,
    delta_x: Vec3f,
    delta_y: Vec3f,
}

impl Camera {
    pub fn new(position: Vec3f, direction: Vec3f, fov_degrees: f32) -> Camera {
        // TODO: Using cross products like this to means that the camera can't point
        // straight up or straight down, because otherwise the cross with Vec3f::UP
        // yields the zero vector, and then normalizing results in NaNs.

        let fov_radians = fov_degrees * ((2.0 * std::f32::consts::PI) / 360.0);
        let fov_scale = (fov_radians / 2.0).tan();

        let delta_x = direction.cross(&Vec3f::UP).normalize().scale(fov_scale);

        let delta_y = direction.cross(&delta_x).normalize().scale(fov_scale);

        Camera {
            position,
            direction: direction.normalize(),
            delta_x,
            delta_y,
        }
    }

    pub fn ray_origin(&self) -> &Vec3f {
        &self.position
    }

    pub fn ray_direction(&self, x: f32, y: f32) -> Vec3f {
        self.direction
            .add(&self.delta_x.scale(x))
            .add(&self.delta_y.scale(y))
    }
}

impl Scene {
    pub fn trace_image(&self, camera: &Camera, width: usize, height: usize) -> Array2D<RGB> {
        let mut image = Array2D::new(height, width, &RGB::BLACK);

        let largest_dimension = width.max(height) as f32;
        let x_offset = (width as f32) / 2.0;
        let y_offset = (height as f32) / 2.0;
        let camera_scale = 2.0 / largest_dimension;

        // Can't figure out how to get Rayon to use my iterator directly, so I
        // convert to a vec of references first.
        let mut tmp: Vec<&mut [RGB]> = image.iter_rows_mut().collect();
        tmp.par_iter_mut().zip(0..height).for_each(|(row, y)| {
            row.iter_mut().zip(0..width).for_each(|(pixel, x)| {
                let camera_x = ((x as f32) - x_offset) * camera_scale;
                let camera_y = ((y as f32) - y_offset) * camera_scale;
                *pixel = self.cast(
                    &camera.ray_origin(),
                    &camera.ray_direction(camera_x, camera_y),
                    10,
                );
            })
        });

        image
    }

    pub fn trace_image_oversampled(
        &self,
        camera: &Camera,
        width: usize,
        height: usize,
        oversampling_factor: usize,
    ) -> Array2D<RGB> {
        if oversampling_factor > 1 {
            let sigma = (oversampling_factor as f32) * 0.4;
            let resampling_kernel = gaussian_kernel(sigma);
            let extra_points_needed = resampling_kernel.len() - 1;

            let oversampled_width = (width * oversampling_factor) + extra_points_needed;
            let oversampled_height = (height * oversampling_factor) + extra_points_needed;

            let oversampled_image = self.trace_image(camera, oversampled_width, oversampled_height);

            convolve_2d(&oversampled_image, &resampling_kernel, oversampling_factor)
        } else {
            self.trace_image(camera, width, height)
        }
    }
    fn trace_to_nearest_object(
        &self,
        ray_origin: &Vec3f,
        ray_direction: &Vec3f,
    ) -> Option<(&VisObj, f32)> {
        self.objects
            .iter()
            // Get a list of intersecting spheres with their distances as a 2-tuple
            .filter_map(|vobj| {
                vobj.surface
                    .intersection_with_ray(&ray_origin, &ray_direction)
                    .map(|dist| (vobj, dist))
            })
            // Select (vobj, distance) 2-tuple with the minimum distance
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
    }

    fn light_on_surface(&self, surface_position: &Vec3f, surface_normal: &Vec3f) -> f32 {
        let trace_pos = surface_position.add(&surface_normal.scale(FLOAT_BIAS));

        let lambert_light_intensity: f32 = self
            .light_sources
            .iter()
            .map(|light_source| {
                match self.trace_to_nearest_object(&trace_pos, &light_source.dir_to_light) {
                    Some(_) => 0.0, // something is in the way
                    None => {
                        // There is a path to the light, apply it
                        light_source
                            .dir_to_light
                            .normalize()
                            .dot(&surface_normal)
                            .max(0.0)
                            * light_source.intensity
                    }
                }
            })
            .sum();

        self.ambient_light_intensity + lambert_light_intensity
    }

    pub fn cast(&self, ray_origin: &Vec3f, ray_direction: &Vec3f, max_depth: i32) -> RGB {
        if max_depth == 0 {
            return self.background;
        }

        match self.trace_to_nearest_object(&ray_origin, &ray_direction) {
            Some((vobj, dist)) => {
                let intersection_pos = ray_origin.add(&ray_direction.scale(dist));
                let surf_prop = vobj.surface.at_point(&intersection_pos);
                let light_intensity = self.light_on_surface(&intersection_pos, &surf_prop.normal);
                let vobj_color = vobj
                    .texture
                    .color(&self, max_depth, surf_prop.u, surf_prop.v);

                let reflected_color = if vobj.reflectivity != 0.0 {
                    let reflect_ray = angle_of_reflection(&ray_direction, &surf_prop.normal);
                    let reflect_origin = intersection_pos.add(&surf_prop.normal.scale(FLOAT_BIAS));

                    self.cast(&reflect_origin, &reflect_ray, max_depth - 1)
                        .scale(vobj.reflectivity)
                } else {
                    RGB::BLACK
                };

                vobj_color.scale(light_intensity).add(&reflected_color)
            }
            None => self.background,
        }
    }
}
