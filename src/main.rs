mod math;
mod ppm;
mod scene;
mod surface;
mod texture;
mod util;

use rand::random;
use std::time::Instant;
use structopt::StructOpt;

use math::*;
use scene::*;
use surface::*;
use texture::*;

#[derive(StructOpt)]
#[structopt(name = "raymond")]
struct CommandLineArguments {
    /// Output file in PPM format (overwritten if already exists)
    #[structopt(short = "o", long = "output", default_value = "raymond_out.ppm")]
    output_file: String,

    /// Width of output image (in pixels)
    #[structopt(short = "w", long = "width", default_value = "1024")]
    width: usize,

    /// Height of output image (in pixels)
    #[structopt(short = "h", long = "height", default_value = "768")]
    height: usize,

    /// Oversampling factor (ie, antialiasing)
    #[structopt(short = "s", long = "samples", default_value = "2")]
    oversampling_factor: usize,
}

#[allow(dead_code)]
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

fn build_scene(camera: &Camera) -> Scene {
    let mut scene = Scene {
        background: RGB {
            red: 0.3,
            green: 0.5,
            blue: 0.9,
        },
        ambient_light_intensity: 0.25,
        light_sources: Vec::new(),
        objects: Vec::new(),
    };

    scene.light_sources.push(LightSource {
        dir_to_light: Vec3f {
            x: 0.0,
            y: -10.0,
            z: 10.0,
        },
        intensity: 0.75,
    });

    // Classic red and white infinite checkerboard
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
        texture: Box::new(Checkerboard::new(
            Box::new(RGB {
                red: 2.5 / 3.0,
                green: 2.5 / 3.0,
                blue: 2.5 / 3.0,
            }),
            Box::new(RGB {
                red: 2.5,
                green: 0.0,
                blue: 0.0,
            }),
        )),
        reflectivity: 0.0,
    });

    let mut colormap = Vec::new();
    colormap.push(RGB {
        red: 0.0,
        green: 0.0,
        blue: 0.5,
    });
    colormap.push(RGB {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
    });
    colormap.push(RGB {
        red: 0.0,
        green: 1.0,
        blue: 1.0,
    });
    colormap.push(RGB {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
    });
    colormap.push(RGB {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
    });
    colormap.push(RGB {
        red: 0.5,
        green: 0.0,
        blue: 0.0,
    });

    // Rectangle showing the Mandelbrot set
    scene.objects.push(VisObj {
        surface: Box::new(Quad::new(
            Plane::new(
                &Vec3f {
                    x: -1.0,
                    y: 4.0,
                    z: 1.0,
                },
                &Vec3f {
                    x: 1.0,
                    y: -1.0,
                    z: 0.0,
                }
                .normalize(),
                &Vec3f {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            ),
            3.0,
            2.5,
        )),
        texture: Box::new(CoordinateTransform {
            texture: Box::new(MandelbrotSet { colormap }),
            u_offset: -2.0,
            v_offset: -1.25,
            u_scale: 1.0,
            v_scale: 1.0,
        }),
        reflectivity: 0.0,
    });

    // Rectangle recursively showing the same scene
    scene.objects.push(VisObj {
        surface: Box::new(Quad::new(
            Plane::new(
                &Vec3f {
                    x: -1.0,
                    y: -4.0,
                    z: 1.0,
                },
                &Vec3f {
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                }
                .normalize(),
                &Vec3f {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            ),
            3.0,
            2.5,
        )),
        texture: Box::new(CoordinateTransform {
            texture: Box::new(Portal {
                camera: camera.clone(),
            }),
            u_offset: -1.5,
            v_offset: -1.25,
            u_scale: -1.0 / 1.5,
            v_scale: -1.0,
        }),
        reflectivity: 0.0,
    });

    // Nice reflective sphere
    scene.objects.push(VisObj {
        surface: Box::new(Sphere::new(
            &Vec3f {
                x: 0.0,
                y: 0.0,
                z: 2.25,
            },
            1.5,
        )),
        texture: Box::new(RGB::BLACK),
        reflectivity: 0.9,
    });

    scene
}

fn main() {
    let args = CommandLineArguments::from_args();

    let camera = Camera::new(
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
        45.0,
    );
    let scene = build_scene(&camera);

    let trace_start = Instant::now();
    let image =
        scene.trace_image_oversampled(&camera, args.width, args.height, args.oversampling_factor);
    println!("Traced image in {} ms.", trace_start.elapsed().as_millis());

    let write_start = Instant::now();
    let mut ppm_out =
        ppm::PPMWriter::new(&args.output_file, image.columns as i32, image.rows as i32).unwrap();

    for scanline in image.iter_rows() {
        for pixel in scanline {
            let (red, green, blue) = pixel.linear_to_srgb().to_rgb24();
            ppm_out.write(red, green, blue).unwrap();
        }
    }
    println!("Wrote output in {} ms.", write_start.elapsed().as_millis());
}
