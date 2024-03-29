use num_complex::Complex;

use crate::util::{run_parallel_jobs, Array2D};

/// 3-D vector or position
#[derive(Debug, Copy, Clone)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// "Linear" Rgb value. (ie, SRGB without gamma correction)
/// Component values fall in [0.0, 1.0]
#[derive(Debug, Copy, Clone)]
pub struct Rgb {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Vec3f {
    pub const UP: Vec3f = Vec3f {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

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
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl Rgb {
    pub const BLACK: Rgb = Rgb {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    /// Produce a 24-bit Rgb value (It is assumed that the caller has already converted
    /// to SRGB with linear_to_srgb())
    pub fn rgb24(&self) -> (u8, u8, u8) {
        (
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8,
        )
    }

    fn map(&self, f: impl Fn(f32) -> f32) -> Rgb {
        Rgb {
            red: f(self.red),
            green: f(self.green),
            blue: f(self.blue),
        }
    }

    pub fn linear_to_srgb(&self) -> Rgb {
        self.map(|x| x.max(0.0).min(1.0).powf(1.0 / 2.2))
    }

    pub fn srgb_to_linear(&self) -> Rgb {
        self.map(|x| x.max(0.0).min(1.0).powf(2.2))
    }

    // TODO: Very similar to Vec3f functionality, and one could imagine use for other
    // methods from Vec3f as well, maybe there is a way to factor out the common methods.
    pub fn scale(&self, factor: f32) -> Rgb {
        Rgb {
            red: self.red * factor,
            green: self.green * factor,
            blue: self.blue * factor,
        }
    }
    pub fn add(&self, other: &Rgb) -> Rgb {
        Rgb {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

/// Finds the roots of the equation ax^2 + bx + c = 0. Returns None if there is
/// no solution,
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    let discriminant = (b * b) - 4.0 * a * c;
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

    let kernel_scale_factor = 1.0 / (2.0 * std::f32::consts::PI * sigma * sigma).sqrt();

    (0..kernel_length)
        .map(|i| (i - half_kernel_length) as f32)
        .map(|x| kernel_scale_factor * (-(x * x) / (2.0 * sigma * sigma)).exp())
        .collect()
}

/// Performs a two dimensional convolution against the provided image and returns
/// a new image. For a W by H image with kernel length K and decimation factor D, the
/// output dimensions will be (W - (K - 1))/D by (H - (K - 1))/D
pub fn convolve_2d(image: &Array2D<Rgb>, kernel: &[f32], decimation_factor: usize) -> Array2D<Rgb> {
    // We convolve & transpose twice, which results in an untransposed image
    let flipped = convolve_and_transpose(image, kernel, decimation_factor);
    convolve_and_transpose(&flipped, kernel, decimation_factor)
}

/// Convolves the given kernel across the image horizontally, and returns a
/// transposed image, optionally decimating.
///
/// The transposition is intended to allow for the function to easily be
/// applied twice to an image to result in a 2D convolution, see convolve_2d.
fn convolve_and_transpose(
    image: &Array2D<Rgb>,
    kernel: &[f32],
    decimation_factor: usize,
) -> Array2D<Rgb> {
    let input_width = image.columns;
    let input_height = image.rows;
    let kernel_length = kernel.len();
    let output_width = input_height;
    let output_height = (input_width - (kernel_length - 1)) / decimation_factor;
    let mut output_image = Array2D::new(output_height, output_width, &Rgb::BLACK);

    // Create jobs that blur the rows of the input image into the columns of
    // the output image.
    let jobs: Vec<_> = output_image
        .iter_columns_mut()
        .zip(image.iter_rows())
        .map(|(mut out_column, in_row)| {
            move || {
                for (out_pixel, out_y) in out_column.iter_mut().zip(0..output_height) {
                    let in_x = out_y * decimation_factor;
                    *out_pixel = in_row[in_x..(in_x + kernel_length)]
                        .iter()
                        .zip(kernel)
                        .map(|(color, coef)| color.scale(*coef))
                        .fold(Rgb::BLACK, |acc, color| acc.add(&color));
                }
            }
        })
        .collect();

    // Run jobs
    run_parallel_jobs(jobs);

    output_image
}

/// Returns the number of iterations it took for a given point on the complex plane to
/// diverge from close to zero, or None if it does not happen after a large number of
/// iterations.
pub fn mandelbrot_escape_time(c: Complex<f32>) -> Option<f32> {
    const MAX_ITERATIONS: i32 = 100;
    // To avoid banding in our smooth shading equation, it is necessary to extend the escape
    // radius beyond the usual 2.0.
    const ESCAPE_RADIUS: f32 = 50.0;

    let mut z = Complex::new(0.0, 0.0);
    let mut i = 0;

    loop {
        z = z * z + c;
        i += 1;

        if z.norm_sqr() > (ESCAPE_RADIUS * ESCAPE_RADIUS) {
            break;
        }

        if i == MAX_ITERATIONS {
            // It didn't escape quickly, we say the point is in the set
            return None;
        }
    }

    // We did escape, now we need to figure out the "fractional iteration"
    // See https://iquilezles.org/www/articles/mset_smooth/mset_smooth.htm
    let escape_time =
        (i as f32) - ((0.5 * z.norm_sqr().ln()) / ESCAPE_RADIUS.ln()).ln() / (2.0_f32).ln();
    Some(escape_time)
}

/// Linearlly interpolates into a grid of colors, wrapping a circular manner if index
/// exceeds the length of the grid.
pub fn linear_interpolation(grid: &[Rgb], index: f32) -> Rgb {
    let base_index = index as usize;
    let fractional_index = index - (base_index as f32);
    let a = grid[base_index % grid.len()];
    let b = grid[(base_index + 1) % grid.len()];

    a.scale(1.0 - fractional_index)
        .add(&b.scale(fractional_index))
}
