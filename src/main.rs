mod math;
mod ppm;
mod scene;
mod surface;
mod texture;
mod util;

use rand::random;
use std::time::Instant;

use math::*;
use scene::*;
use surface::*;
use texture::*;

fn random_sphere() -> VisObj {
    VisObj {
        surface: Box::new(Sphere::new(
            &Vec3f {
                x: random::<f32>() * 10.0,
                y: random::<f32>() * 10.0 - 5.0,
                z: random::<f32>() * 5.0,
            },
            1.0,
        )),
        texture: Box::new(RGB {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            //red: random::<f32>(),
            //green: random::<f32>(),
            //blue: random::<f32>(),
        }),
        reflectivity: 0.9,
    }
}

fn build_scene() -> Scene {
    let mut scene = Scene {
        background: RGB {
            red: 0.3,
            green: 0.5,
            blue: 0.9,
        },
        light_sources: Vec::new(),
        objects: Vec::new(),
    };

    scene.light_sources.push(LightSource {
        dir_to_light: Vec3f {
            x: 0.0,
            y: 10.0,
            z: 10.0,
        },
        intensity: 5.0,
    });

    let mut colormap = Vec::new();
    colormap.push(RGB {
        red: 0.0,
        green: 0.0,
        blue: 0.25,
    });
    colormap.push(RGB {
        red: 0.0,
        green: 0.0,
        blue: 0.5,
    });
    colormap.push(RGB {
        red: 0.0,
        green: 0.5,
        blue: 0.5,
    });
    colormap.push(RGB {
        red: 0.5,
        green: 0.5,
        blue: 0.0,
    });
    colormap.push(RGB {
        red: 0.5,
        green: 0.0,
        blue: 0.0,
    });
    colormap.push(RGB {
        red: 0.25,
        green: 0.0,
        blue: 0.0,
    });

    // Infinite plane of the Mandelbrot Set
    scene.objects.push(VisObj {
        surface: Box::new(Plane::new(
            &Vec3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            &Vec3f {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            &Vec3f {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        )),
        texture: Box::new(MandelbrotSet { colormap }),
        reflectivity: 0.0,
    });

    for _ in 0..10 {
        scene.objects.push(random_sphere());
    }

    scene
}

fn main() {
    let oversampling_factor = 2;
    let width = 1024;
    let height = 768;

    let scene = build_scene();
    // TODO: It's awkward to have to tell both the Camera and trace_image_oversampled()
    // about the oversampling factor
    let camera = Camera::new(
        width * oversampling_factor,
        height * oversampling_factor,
        Vec3f {
            x: -11.0,
            y: 0.0,
            z: 2.0,
        },
        Vec3f {
            x: 10.0,
            y: 0.0,
            z: -1.0,
        },
    );

    let trace_start = Instant::now();
    let image = scene.trace_image_oversampled(&camera, width, height, oversampling_factor);
    println!("Traced image in {} ms.", trace_start.elapsed().as_millis());

    let write_start = Instant::now();
    let mut ppm_out =
        ppm::PPMWriter::new("test.ppm", image.columns as i32, image.rows as i32).unwrap();

    for scanline in image.iter_rows() {
        for pixel in scanline {
            let (red, green, blue) = pixel.linear_to_srgb().to_rgb24();
            ppm_out.write(red, green, blue).unwrap();
        }
    }
    println!("Wrote output in {} ms.", write_start.elapsed().as_millis());
}
