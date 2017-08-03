
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use std::thread;
use std::f32::consts::PI;
use std::cmp::min;

use cgmath::prelude::*;
use threadpool::ThreadPool;
use num_cpus;

use objects::{Scene, SurfaceType};
use raycast::{Ray,Intersection};
use image::{DynamicImage, GenericImage};
use types::Color;

fn format_time(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}


fn shade_diffuse(scene: &Scene, intersection: &Intersection) -> Color {
    let mut color = Color::from_rgb(0.0, 0.0, 0.0);

    for light in &scene.lights {
        let direction_to_light = -light.direction().normalize();
        let shadow_ray = Ray {
            origin: intersection.hit_point() + intersection.direction() * 1e-13,
            direction: direction_to_light
        };

        let in_light = scene.trace(&shadow_ray).is_none();

        let light_intensity = if in_light { light.intensity() } else { 0.0 };
        let light_power = (intersection.direction().dot(direction_to_light) as f32).max(0.0) * light_intensity;
        let light_reflected = intersection.object().material().albedo / PI;


        color = color + (intersection.object().color(&intersection.texture_coord()).clone() * light.color().clone() * light_power * light_reflected);
    }

    color
}

fn get_color(scene: &Scene, ray: &Ray, intersection: &Intersection, depth: u32) -> Color {
    let mut color = shade_diffuse(scene, intersection);
    if let SurfaceType::Reflective { reflectivity } = intersection.object().material().surface {
        let reflection_ray = Ray::create_reflection(&ray.direction, intersection);
        let reflection_color = cast_ray(scene, &reflection_ray, depth + 1) * reflectivity;
        color = color * (1.0 * reflectivity) + reflection_color
    }

    color
}

pub fn cast_ray(scene: &Scene, ray: &Ray, depth: u32) -> Color {
    if depth >= 32 {
        return Color::from_rgb(0.0, 0.0, 0.0);
    }

    scene.trace(&ray)
        .map(|int| get_color(scene, &ray, &int, depth))
        .unwrap_or(Color::from_rgb(0.0, 0.0, 0.0))
}

pub fn render(scene: Scene) -> DynamicImage {
    let workers = num_cpus::get();
    let pool = ThreadPool::new(workers);

    let sw = scene.width;
    let sh = scene.height;

    let tile_size = 128;
    let cols = (scene.width as f32 / tile_size as f32).ceil() as u32;
    let rows = (scene.height as f32 / tile_size as f32).ceil() as u32;
    let jobs = cols * rows;
    let asc = Arc::new(scene);

    let (tx, rx) = channel();
    for job_idx in 0..jobs {
        let mx = tile_size * (job_idx % cols);
        let my = tile_size * (job_idx / cols);
        let black = Color::from_rgb(0.0, 0.0, 0.0).to_rgba8();
        let mscene = asc.clone();
        let tx = tx.clone();
        pool.execute(move || {
            let tile_width = min((mx + tile_size), sw) - mx;
            let tile_height = min((my + tile_size), sh) - my;
            let mut image = DynamicImage::new_rgb8(tile_width, tile_height);
            for x in 0..tile_width {
                for y in 0..tile_height {
                    let ray = Ray::create_prime(mx + x, my + y, &*mscene);

                    if let Some(inter) = mscene.trace(&ray) {
                        let color = get_color(&*mscene, &ray, &inter, 0);
                        image.put_pixel(x, y, color.clamp().to_rgba8());
                    } else {
                        image.put_pixel(x, y, black);
                    }
                }
            }
            tx.send((image, mx, my)).unwrap();
        });
    }

    rx.iter()
        .take(jobs as usize)
        .fold(
            DynamicImage::new_rgb8(sw, sh),
            |mut image, result| {
                let (part, x, y) = result;
                image.copy_from(&part, x, y);
                image
            }
        )
}