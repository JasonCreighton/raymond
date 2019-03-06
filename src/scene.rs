use rayon::prelude::*;

use crate::math::{angle_of_reflection, convolve_2d, gaussian_kernel, LinearRGB, Vec3f};
use crate::surface::Surface;
use crate::texture::Texture;

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
    pub background: LinearRGB,
    pub light_sources: Vec<LightSource>,
    pub objects: Vec<VisObj>,
}

#[derive(Debug)]
pub struct Camera {
    position: Vec3f,
    upper_left: Vec3f,
    delta_x: Vec3f,
    delta_y: Vec3f,
}

impl Camera {
    pub fn new(width: i32, height: i32, position: Vec3f, direction: Vec3f) -> Camera {
        // TODO: Using cross products like this to means that the camera can't point
        // straight up or straight down, because otherwise the cross with Vec3f::UP
        // yields the zero vector, and then normalizing results in NaNs.
        let zoom = 2.0;
        let delta_x = direction
            .cross(&Vec3f::UP)
            .normalize()
            .scale(1.0 / (width as f32));
        // NB: delta_y is scaled relative to width as well, to make sure we get square
        // pixels
        let delta_y = direction
            .cross(&delta_x)
            .normalize()
            .scale(1.0 / (width as f32));
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
    pub fn trace_image(&self, camera: &Camera, width: i32, height: i32) -> Vec<Vec<LinearRGB>> {
        (0..height)
            .into_par_iter()
            .map(|y| {
                (0..width)
                    .map(|x| self.cast(&camera.ray_origin(), &camera.ray_direction(x, y), 10))
                    .collect()
            })
            .collect()
    }

    pub fn trace_image_oversampled(
        &self,
        camera: &Camera,
        width: i32,
        height: i32,
        oversampling_factor: i32,
    ) -> Vec<Vec<LinearRGB>> {
        if oversampling_factor > 1 {
            let sigma = (oversampling_factor as f32) * 0.4;
            let resampling_kernel = gaussian_kernel(sigma);
            let extra_points_needed = (resampling_kernel.len() - 1) as i32;

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
        let ambient_light_intensity = 1.0;
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

                    self.cast(&reflect_origin, &reflect_ray, max_depth - 1)
                        .scale(vobj.reflectivity)
                } else {
                    LinearRGB {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                    }
                };

                vobj_color.scale(light_intensity).add(&reflected_color)
            }
            None => self.background,
        }
    }
}
