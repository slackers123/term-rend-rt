use glam::{Mat4, Vec3};
use math::{random_vec_in_hemisphere, Camera, Color, Material};

use crate::math::{Ray, Renderable};
use show_image::create_window;

mod math;

// the following are options
const SCREEN_HEIGHT: u32 = 1080;
const SCREEN_WIDTH: u32 = 1920;
const SUN_DIR: Vec3 = Vec3::new(0.1, 1.0, 0.3);
const BOUNCE_AMOUNT: u32 = 70;
const SAMPLES_PER_PIXEL: u32 = 100;
const SKY_COL: Color = Color {
    r: 0.5,
    g: 0.7,
    b: 1.0,
};

// the following are not to be tweaked
const PIXEL_SIZE: f32 = 1.0 / SCREEN_WIDTH as f32;
const PIXEL_OFF_HEIGHT: f32 = PIXEL_SIZE * (SCREEN_HEIGHT as f32 / 2.0);

type Scene = Vec<Box<dyn Renderable>>;

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tri = math::Tri {
        a: Vec3::new(0.0, 1.0, 1.5),
        b: Vec3::new(0.5, 0.0, 1.5),
        c: Vec3::new(-0.5, 0.0, 1.5),
        material: Material {
            color: Color {
                r: 0.5,
                g: 0.0,
                b: 0.5,
            },
            metalness: 0.2,
        },
    };

    let mut sphere = math::Sphere {
        pos: Vec3::new(0.0, 1.0, 10.0),
        rad: 1.0,
        material: Material {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
            },
            metalness: 0.5,
        },
    };

    let mut plane = math::Plane {
        pos: Vec3::new(0.0, 0.0, 0.0),
        norm: Vec3::new(0.0, 1.0, 0.0),
        material: Material {
            color: Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            metalness: 0.0,
        },
    };

    let camera = Camera {
        pos: Vec3::new(0.0, 1.0, 0.0),
        dir: Vec3::new(0.0, 0.0, 1.0),
    };

    let view_matrix = Mat4::look_to_lh(camera.pos, camera.dir, Vec3::Y);

    tri.to_homogeneous(view_matrix);
    sphere.to_homogeneous(view_matrix);
    plane.to_homogeneous(view_matrix);

    use image::{Rgb, RgbImage};

    let scene: Scene = vec![Box::new(sphere), Box::new(plane)];

    let mut img = RgbImage::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let t_start = std::time::Instant::now();
    for y in 0..SCREEN_HEIGHT {
        println!("{}% done", (y as f32 / SCREEN_HEIGHT as f32) * 100.0);
        for x in 0..SCREEN_WIDTH {
            let mut pixel_col = SKY_COL;
            for s in 0..SAMPLES_PER_PIXEL {
                let r = Ray {
                    pos: Vec3::ZERO,
                    dir: Vec3::new(
                        -0.5 + (PIXEL_SIZE * x as f32) + rand::random::<f32>() * PIXEL_SIZE,
                        PIXEL_OFF_HEIGHT - (PIXEL_SIZE * y as f32)
                            + rand::random::<f32>() * PIXEL_SIZE,
                        1.0,
                    ),
                };
                pixel_col = pixel_col + cast_ray_recursive(&scene, r, 0);
            }
            let ratio = 1.0 / SAMPLES_PER_PIXEL as f32;
            pixel_col = pixel_col * ratio;
            img.put_pixel(
                x,
                y,
                Rgb([
                    (255.0 * pixel_col.r.sqrt()) as u8,
                    (255.0 * pixel_col.g.sqrt()) as u8,
                    (255.0 * pixel_col.b.sqrt()) as u8,
                ]),
            );
        }
    }
    println!("it took {:?} to render", t_start.elapsed());

    let window = create_window("image", Default::default())?;
    window.set_image("image-001", img.clone())?;

    for event in window.event_channel()? {
        if let show_image::event::WindowEvent::KeyboardInput(event) = event {
            if event.input.key_code == Some(show_image::event::VirtualKeyCode::Escape)
                && event.input.state.is_pressed()
            {
                break;
            }
        }
    }
    img.save("rendered_image.png")?;

    Ok(())
}

fn cast_ray_recursive(scene: &Scene, ray: Ray, d: u32) -> Color {
    if d == BOUNCE_AMOUNT {
        return Color::BLACK;
    }

    match find_closest(scene, ray) {
        Some((t, n, mat)) => {
            let res_p = ray.pos + ray.dir * t;
            let target = res_p + n + random_vec_in_hemisphere(n);
            return cast_ray_recursive(
                scene,
                Ray {
                    pos: res_p,
                    dir: target - res_p,
                },
                d + 1,
            ) * 0.5;
        }
        None => {
            let unit_dir = ray.dir.normalize();
            let t = 0.5 * (unit_dir.y + 1.0);
            return Color::WHITE * (1.0 - t) + SKY_COL * t;
        }
    }
}

fn find_closest(scene: &Scene, ray: Ray) -> Option<(f32, Vec3, Material)> {
    scene
        .iter()
        .filter_map(|i| i.intersect(ray))
        .filter_map(|i| if i.0 < 0.001 { None } else { Some(i) })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}
