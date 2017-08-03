
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use std::thread;
use std::f32::consts::PI;

use cgmath::prelude::*;

use objects::Scene;
use raycast::{Ray,Intersection};
use image::{DynamicImage, GenericImage};
use types::Color;

fn format_time(duration: &Duration) -> f64 {
  duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

fn get_color(scene: &Scene, intersection: &Intersection) -> Color {
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
    let light_reflected = 0.10 / PI;
    color = color + (intersection.object().color().clone() * light.color().clone() * light_power * light_reflected);
  }
  
  color
}

pub fn render(scene: Scene) -> DynamicImage {
  let sw = scene.width;
  let sh = scene.height;
  let tw = sw / 2;
  let th = sh / 2;

  let mut children = Vec::new();
  let asc = Arc::new(scene);

  for t in 0..4 {
    let mx = tw * (t % 2);
    let my = th * (t / 2);
    let black = Color::from_rgb(0.0, 0.0, 0.0);
    let start = Instant::now();
    let (tx, rx) = channel();
    let mscene = asc.clone();
    let child = thread::spawn(move || {
      let mut image = DynamicImage::new_rgb8(tw, th);
      for x in 0..tw {
        for y in 0..th {
          let ray = Ray::create_prime(mx + x, my + y, &*mscene);

          if let Some(inter) = mscene.trace(&ray) {
            let color = get_color(&*mscene, &inter);
            image.put_pixel(x, y, color.clamp().to_rgba8());
          } else {
            image.put_pixel(x, y, black.to_rgba8());
          }          
        }
      }
      tx.send(image).unwrap();    
    });

    children.push((rx, child, mx, my, start));
  }

  let mut image = DynamicImage::new_rgb8(sw, sh);
  for (rx, child, mx, my, start_time) in children {
    let region = rx.recv().unwrap();
    let before_copy = Instant::now();
    image.copy_from(&region, mx, my);
    let tdur = start_time.elapsed();
    let duration = before_copy.elapsed();
    println!(
      "copy image: {:?}, thread time: {:?}", 
      format_time(&duration), format_time(&tdur)
    );
    let _ = child.join();
  }

  image
}