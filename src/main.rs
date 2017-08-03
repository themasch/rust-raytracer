extern crate image;
extern crate cgmath;

mod types;
mod raycast;
mod objects;
mod light;
mod render;

use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use cgmath::prelude::*;

use objects::{Object, Sphere, Plane, SceneBuilder};
use types::{Color,Point,Direction};
use light::*;
use render::render;



fn format_time(duration: &Duration) -> f64 {
  duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

fn main() {
  let scene = SceneBuilder::new(1920, 1080)
    .add_object(Object::Sphere(Sphere {
      center: Point::new(0.0, 1.0, -5.0),
      radius: 1.0,
      color: Color::from_rgb(0.4, 1.0, 0.4)
    }))
    .add_object(Object::Sphere(Sphere {
      center: Point::new(-1.0, 0.0, -9.0),
      radius: 2.0,
      color: Color::from_rgb(1.0, 0.1, 0.1)
    }))
    .add_object(Object::Plane(Plane {
      origin: Point::new(0.0, 0.0, -20.0), 
      normal: Direction::new(0.0, 0.0, -1.0).normalize(),
      color: Color::from_rgb(0.0, 0.0, 1.0)
    }))
    .add_object(Object::Plane(Plane {
      origin: Point::new(0.0, -2.0, -5.0), 
      normal: Direction::new(0.0, -1.0, 0.0).normalize(),
      color: Color::from_rgb(0.1, 0.3, 0.6)
    }))
    .add_light(Light::Directional(DirectionalLight {
      direction: Direction::new(0.0, -0.5, -1.0),
      color: Color::from_rgb(1.0, 1.0, 1.0), 
      intensity: 20.0
    }))
    .add_light(Light::Directional(DirectionalLight {
      direction: Direction::new(0.0, -1.0, -1.0),
      color: Color::from_rgb(1.0, 1.0, 1.0), 
      intensity: 10.0
    }))
    .finish();


  let before_render = Instant::now();
  let image = render(scene);
  let before_save = Instant::now();
  let ref mut fout = File::create(&Path::new("test.png")).unwrap();
  let res = image.save(fout, image::PNG).unwrap();

  println!("render: {:?}, save: {:?}", 
    format_time(&before_save.duration_since(before_render)),
    format_time(&before_save.elapsed())
  );
}


#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq;

#[test]
fn test_can_render_scene() {
  let scene = SceneBuilder::new(1920, 1080)
    .add_object(Object::Sphere(Sphere {
      center: Point::new(0.0, 0.0, -5.0),
      radius: 1.0,
      color: Color::from_rgb(0.4, 1.0, 0.4)
    }))
    .finish();

  let img: DynamicImage = render(&scene);
  assert_eq!(scene.width, img.width());
  assert_eq!(scene.height, img.height());
}


#[test]
fn test_creates_prime_ray() {
  let scene = SceneBuilder::new(640, 480)
    .add_object(Object::Sphere(Sphere {
      center: Point::new(0.0, 0.0, -5.0),
      radius: 1.0,
      color: Color::from_rgb(0.4, 1.0, 0.4)
    }))
    .finish();

  let ray = Ray::create_prime(20, 20, &scene);

  assert_eq!(0.0, ray.origin.x);
  assert_eq!(0.0, ray.origin.y);
  assert_eq!(0.0, ray.origin.z);
  assert_approx_eq!(1.0, ray.direction.magnitude());
}

