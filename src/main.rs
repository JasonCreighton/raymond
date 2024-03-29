mod math;
mod ppm;
mod scene;
mod surface;
mod texture;
mod util;

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use math::*;
use scene::*;
use surface::*;
use texture::*;

struct CommandLineArguments {
    output_file: String,
    width: usize,
    height: usize,
    oversampling_factor: usize,
}

type FlagNames = (&'static str, &'static str);
impl CommandLineArguments {
    const FLAG_OUTPUT: FlagNames = ("-o", "--output");
    const FLAG_WIDTH: FlagNames = ("-w", "--width");
    const FLAG_HEIGHT: FlagNames = ("-h", "--height");
    const FLAG_SAMPLES: FlagNames = ("-s", "--samples");

    fn default() -> CommandLineArguments {
        CommandLineArguments {
            output_file: String::from("raymond_out.ppm"),
            width: 1024,
            height: 768,
            oversampling_factor: 2,
        }
    }

    fn show_usage() {
        fn flag_usage(f: FlagNames, desc: &str) {
            eprintln!("    {}, {:20} {}", f.0, f.1, desc);
        }

        eprintln!("Usage: raymond [options]");
        eprintln!();
        flag_usage(
            Self::FLAG_OUTPUT,
            "Output file in PPM format (overwritten if already exists)",
        );
        flag_usage(Self::FLAG_WIDTH, "Width of output image (in pixels)");
        flag_usage(Self::FLAG_HEIGHT, "Height of output image (in pixels)");
        flag_usage(Self::FLAG_SAMPLES, "Oversampling factor (ie, antialiasing)");
    }

    fn from_args() -> Result<CommandLineArguments, String> {
        fn is_flag(s: &str, flag: FlagNames) -> bool {
            s == flag.0 || s == flag.1
        }

        let mut raw_args: Vec<String> = env::args().collect();
        let mut args = Self::default();

        raw_args.reverse();
        raw_args.pop(); // skip program name

        while let Some(flag) = raw_args.pop() {
            let arg = match raw_args.pop() {
                Some(arg) => arg,
                None => return Err(String::from("Value expected after command line argument")),
            };

            if is_flag(&flag, Self::FLAG_OUTPUT) {
                args.output_file = arg;
            } else if is_flag(&flag, Self::FLAG_WIDTH) {
                args.width = arg.parse().map_err(|_| "Could not parse width")?;
            } else if is_flag(&flag, Self::FLAG_HEIGHT) {
                args.height = arg.parse().map_err(|_| "Could not parse height")?;
            } else if is_flag(&flag, Self::FLAG_SAMPLES) {
                args.oversampling_factor = arg
                    .parse()
                    .map_err(|_| "Could not parse oversampling factor")?;
            } else {
                return Err(String::from("Unexpected command line argument"));
            }
        }

        Ok(args)
    }
}

#[allow(dead_code)]
fn random_sphere() -> VisObj {
    VisObj {
        surface: Box::new(Sphere::new(
            &Vec3f {
                x: util::rand_f32() * 10.0,
                y: util::rand_f32() * 10.0 - 5.0,
                z: util::rand_f32() * 5.0,
            },
            1.0,
        )),
        texture: Box::new(Rgb {
            red: util::rand_f32(),
            green: util::rand_f32(),
            blue: util::rand_f32(),
        }),
        reflectivity: 0.9,
    }
}

fn build_scene(camera: &Camera) -> Scene {
    let mut scene = Scene {
        background: Rgb {
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
            Box::new(Rgb {
                red: 2.5 / 3.0,
                green: 2.5 / 3.0,
                blue: 2.5 / 3.0,
            }),
            Box::new(Rgb {
                red: 2.5,
                green: 0.0,
                blue: 0.0,
            }),
        )),
        reflectivity: 0.0,
    });

    let colormap = vec![
        Rgb {
            red: 0.0,
            green: 0.0,
            blue: 0.5,
        },
        Rgb {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
        },
        Rgb {
            red: 0.0,
            green: 1.0,
            blue: 1.0,
        },
        Rgb {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        },
        Rgb {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
        },
        Rgb {
            red: 0.5,
            green: 0.0,
            blue: 0.0,
        },
    ];

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
        texture: Box::new(Rgb::BLACK),
        reflectivity: 0.9,
    });

    scene
}

fn main() -> ExitCode {
    let args = match CommandLineArguments::from_args() {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("Error processing command line arguments: {}", msg);
            eprintln!();
            CommandLineArguments::show_usage();
            return ExitCode::FAILURE;
        }
    };

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
            let (red, green, blue) = pixel.linear_to_srgb().rgb24();
            ppm_out.write(red, green, blue).unwrap();
        }
    }
    println!("Wrote output in {} ms.", write_start.elapsed().as_millis());

    ExitCode::SUCCESS
}
