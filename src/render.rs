use std::cmp::min;
use std::f32::consts::PI;
use std::sync::mpsc::channel;
use std::sync::Arc;

use cgmath::prelude::*;
use num_cpus;
use threadpool::ThreadPool;

use image::{DynamicImage, GenericImage};
use raycast::{IntersectionResult, Ray};
use scene::Scene;
use types::Color;

fn shade_diffuse(scene: &Scene, intersection: &IntersectionResult) -> Color {
    let mut color = Color::from_rgb(0.0, 0.0, 0.0);
    //println!("sn: {:?}", intersection.surface_normal());
    for light in &scene.lights {
        let direction_to_light = (-light.direction()).normalize();
        let shadow_ray = Ray::create_shadow_ray(direction_to_light, intersection);
        let shadow_trace = scene.trace(&shadow_ray);
        if shadow_trace.is_none() {
            let light_intensity = light.intensity();
            let light_power = (intersection.surface_normal().dot(direction_to_light) as f32).abs();
            let light_reflected = intersection.albedo() / PI;
            //  println!("P: {:?} \t R: {:?} ", light_power, light_reflected);
            color = color
                + (intersection.color()
                    * light.color().clone()
                    * light_power
                    * light_intensity
                    * light_reflected);
        } else {
            //println!("SHADOW distance: {:?}", shadow_trace.unwrap().distance());
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
            let tile_width = min(mx + tile_size, sw) - mx;
            let tile_height = min(my + tile_size, sh) - my;
            let mut image = DynamicImage::new_rgb8(tile_width, tile_height);

            for x in 0..tile_width {
                for y in 0..tile_height {
                    if !(mx + x > 380 && mx + x < 420 && my + y > 80 && my + y < 100) {
                        //  continue;
                    }
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
