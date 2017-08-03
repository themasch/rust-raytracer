use types::{Direction,Point};
use cgmath::prelude::*;
use objects::{Object, Scene, TextureCoords};

use std::cmp::Ordering;

pub struct Intersection<'a> {  
  distance: f64,
  direction: Direction,
  hit_point: Point,
  tex_coord: TextureCoords,
  object: &'a Object
}

impl<'a> Intersection<'a> {
  pub fn new<'b>(distance: f64, hit_point: Point, tex_coord: TextureCoords, direction: Direction, object: &'b Object) -> Intersection<'b> {
    Intersection { 
      distance,
      hit_point,
      direction,
      tex_coord,
      object
    }
  }

  pub fn compare_to(&self, other: &Intersection) -> Option<Ordering> {
    self.distance.partial_cmp(&other.distance)
  }

  pub fn object(&self) -> &Object {
    self.object
  } 

  pub fn hit_point(&self) -> Point {
    self.hit_point
  }

  pub fn direction(&self) -> Direction {
    self.direction
  }

  pub fn texture_coord(&self) -> &TextureCoords { &self.tex_coord }
}

pub trait Intersectable {
  fn intersect(&self, ray: &Ray) -> Option<f64>;

  fn surface_normal(&self, hit_point: &Point) -> Direction;
  fn texture_coord(&self, hit_point: &Point) -> TextureCoords;
}

pub struct Ray {
  pub origin: Point,
  pub direction: Direction
}

impl Ray {
  pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
    let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();
    let aspect_ratio = scene.width as f64 / scene.height as f64;
    let sensor_x = (((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0) * aspect_ratio * fov_adjustment;
    let sensor_y = (1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0) * fov_adjustment;

    Ray { 
      origin: Point::zero(),
      direction: Direction {
        x: sensor_x,
        y: sensor_y,
        z: -1.0
      }.normalize()
    }
  }
}

