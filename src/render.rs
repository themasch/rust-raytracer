use std::cmp::min;
use std::f32::consts::PI;
use std::sync::mpsc::channel;
use std::sync::Arc;

use cgmath::prelude::*;
use num_cpus;
use threadpool::ThreadPool;

use image::Rgba;
use image::{DynamicImage, GenericImage};
use raycast::{IntersectionResult, Ray};
use scene::{Camera, Scene};
use std::time::{Duration, Instant};
use types::Color;

fn shade_diffuse(scene: &Scene, intersection: &IntersectionResult) -> Color {
    let mut color = Color::from_rgb(0.0, 0.0, 0.0);
    for light in &scene.lights {
        let direction_to_light = (-light.direction()).normalize();
        let shadow_ray = Ray::create_shadow_ray(direction_to_light, intersection);
        let shadow_trace: Option<IntersectionResult> = scene.trace(&shadow_ray);
        if shadow_trace.is_none() {
            let light_intensity = light.intensity();
            let light_power = (intersection.surface_normal().dot(direction_to_light) as f32).abs();
            let light_reflected = intersection.albedo() / PI;
            color = color
                + (intersection.color()
                    * light.color().clone()
                    * light_power
                    * light_intensity
                    * light_reflected);
        }
    }

    color
}

fn get_color(scene: &Scene, ray: &Ray, intersection: &IntersectionResult, depth: u32) -> Color {
    let mut color = shade_diffuse(scene, intersection);
    if let Some(relf) = intersection.reflectivity() {
        let reflection_ray = Ray::create_reflection(&ray.direction, intersection);
        let reflection_color = cast_ray(scene, &reflection_ray, depth + 1) * relf;
        color = color * (1.0 - relf) + reflection_color
    }

    color
}

pub fn cast_ray(scene: &Scene, ray: &Ray, depth: u32) -> Color {
    if depth >= 32 {
        return Color::from_rgb(0.0, 0.0, 0.0);
    }

    scene
        .trace(&ray)
        .map(|int| get_color(scene, &ray, &int, depth))
        .unwrap_or(Color::from_rgb(0.0, 0.0, 0.0))
}

pub fn sample(x: f64, y: f64, scene: &Scene, camera: &Camera) -> Option<Rgba<u8>> {
    let ray = Ray::create_prime(x, y, &scene, &camera);
    let trace = scene.trace(&ray);
    trace.map(|inter| {
        let color = get_color(&scene, &ray, &inter, 0);
        color.clamp().to_rgba8()
    })
}

pub fn average_color(samples: Vec<Rgba<u8>>) -> Rgba<u8> {
    let sample_count = samples.len();
    let data: [usize; 4] = samples.iter().fold([0, 0, 0, 0], |mut data, sample| {
        data[0] = data[0] + sample.data[0] as usize;
        data[1] = data[1] + sample.data[1] as usize;
        data[2] = data[2] + sample.data[2] as usize;
        data[3] = data[3] + sample.data[3] as usize;
        data
    });

    let data: [u8; 4] = [
        (data[0] / sample_count) as u8,
        (data[1] / sample_count) as u8,
        (data[2] / sample_count) as u8,
        (data[3] / sample_count) as u8,
    ];

    Rgba(data)
}

pub fn super_sample(x: f64, y: f64, scene: &Scene, camera: &Camera) -> Option<Rgba<u8>> {
    let black = Color::from_rgb(0.0, 0.0, 0.0).to_rgba8();
    let samples = vec![
        sample((x - 0.25), (y - 0.25), scene, camera).unwrap_or(black),
        sample((x + 0.25), (y - 0.25), scene, camera).unwrap_or(black),
        sample((x - 0.25), (y + 0.25), scene, camera).unwrap_or(black),
        sample((x + 0.25), (y + 0.25), scene, camera).unwrap_or(black),
        sample((x), (y), scene, camera).unwrap_or(black),
    ];

    Some(average_color(samples))
}

pub fn render(scene: Scene, camera: Camera) -> DynamicImage {
    let workers = num_cpus::get();
    let pool = ThreadPool::new(workers);

    let sw = camera.width;
    let sh = camera.height;

    let tile_size = 128;
    let cols = (camera.width as f32 / tile_size as f32).ceil() as u32;
    let rows = (camera.height as f32 / tile_size as f32).ceil() as u32;
    let jobs = cols * rows;
    let asc = Arc::new(scene);
    let camera = Arc::new(camera);

    let (tx, rx) = channel();
    for job_idx in 0..jobs {
        let mx = tile_size * (job_idx % cols);
        let my = tile_size * (job_idx / cols);
        let black = Color::from_rgb(0.0, 0.0, 0.0).to_rgba8();
        let mscene = asc.clone();
        let tx = tx.clone();
        let camera = camera.clone();
        pool.execute(move || {
            let start = Instant::now();
            let tile_width = min(mx + tile_size, sw) - mx;
            let tile_height = min(my + tile_size, sh) - my;
            let mut image = DynamicImage::new_rgb8(tile_width, tile_height);

            for x in 0..tile_width {
                for y in 0..tile_height {
                    let color = super_sample((mx + x) as f64, (my + y) as f64, &mscene, &camera)
                        .unwrap_or(black);
                    image.put_pixel(x, y, color);
                }
            }
            tx.send((image, mx, my)).unwrap();
        });
    }

    let mut counter = 0;
    rx.iter()
        .inspect(|_| {
            counter = counter + 1;
            println!("{:?} of {:?} done", counter, jobs);
        })
        .take(jobs as usize)
        .fold(DynamicImage::new_rgb8(sw, sh), |mut image, result| {
            let (part, x, y) = result;
            image.copy_from(&part, x, y);
            image
        })
}
