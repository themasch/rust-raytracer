use raycast::{Intersectable,Intersection,Ray};
use types::{Color, Point, Direction};
use cgmath::prelude::*;
use cgmath::Vector3;
use light::Light;
use image::{DynamicImage, GenericImage};

use std::f32::consts::PI;

pub struct TextureCoords {
    pub x: f32,
    pub y: f32
}

pub enum Coloration {
  Color(Color),
  Texture(DynamicImage)
}

fn wrap(val: f32, bound: u32) -> u32 {
    let signed_bound = bound as i32;
    let float_coord = val * bound as f32;
    let wrapped_coord = (float_coord as i32) % signed_bound;
    if wrapped_coord < 0 {
        (wrapped_coord + signed_bound) as u32
    } else {
        wrapped_coord as u32
    }
}

impl Coloration {
    pub fn color(&self, coords: &TextureCoords) -> Color {
        match *self {
            Coloration::Color(ref c) => c.clone(),
            Coloration::Texture(ref tex) => {
                let tex_x = wrap(coords.x, tex.width());
                let tex_y = wrap(coords.y, tex.height());

                Color::from_rgba(tex.get_pixel(tex_x, tex_y))
            }
        }
    }
}

pub struct Material {
    pub color: Coloration,
    pub albedo: f32
}

impl Material {
    pub fn new(color: Coloration, albedo: f32) -> Material {
        Material { color, albedo }
    }

    pub fn diffuse_color(color: Color, albedo: f32) -> Material {
        Material {
            color: Coloration::Color(color),
            albedo
        }
    }

    pub fn diffuse_texture(image: DynamicImage, albedo: f32) -> Material {
        Material {
            color: Coloration::Texture(image),
            albedo
        }
    }
}

pub enum Object {
  Sphere(Sphere),
  Plane(Plane)
}

impl Object {
    pub fn color(&self, coords: &TextureCoords) -> Color {
        match *self {
            Object::Sphere(ref s) => s.material.color.color(coords),
            Object::Plane(ref p) => p.material.color.color(coords)
        }
    }
}

impl Intersectable for Object {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        match *self {
            Object::Sphere(ref s) => s.intersect(ray),
            Object::Plane(ref p) => p.intersect(ray)
        }
    }

    fn surface_normal(&self, hit_point: &Point) -> Direction {
        match *self {
            Object::Sphere(ref s) => s.surface_normal(hit_point),
            Object::Plane(ref p) => p.surface_normal(hit_point)
        }
    }

    fn texture_coord(&self, hit_point: &Point) -> TextureCoords {
        match *self {
            Object::Sphere(ref s) => s.texture_coord(hit_point),
            Object::Plane(ref p) => p.texture_coord(hit_point)
        }
    }
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub material: Material
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let l = self.center - ray.origin;
        let adj2 = l.dot(ray.direction);

        let d2 = l.dot(l) - adj2.powi(2);
        let radius2 = self.radius.powi(2);

        if d2 > radius2 {
            return None;
        }

        let thc = (radius2 - d2).sqrt();
        let t0 = adj2 - thc;
        let t1 = adj2 + thc;
        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }

        let distance = if t0 < t1 { t0 } else { t1 };
        Some(distance)
    }

    fn surface_normal(&self, hit_point: &Point) -> Direction {
        (*hit_point - self.center).normalize()
    }

    fn texture_coord(&self, hit_point: &Point) -> TextureCoords {
        let hit_vec = *hit_point - self.center;
        TextureCoords {
            x: (1.0 + (hit_vec.z.atan2(hit_vec.x) as f32) / PI) * 0.5,
            y: (hit_vec.y / self.radius).acos() as f32 / PI
        }
    }
}

pub struct Plane {
  pub origin: Point, 
  pub normal: Direction,
    pub material: Material
}

impl Intersectable for Plane {
  fn intersect(&self, ray: &Ray) -> Option<f64> {
    let normal = self.normal;
    let denom = normal.dot(ray.direction);
    if denom > 1e-6 {
      let v = self.origin - ray.origin;
      let distance = v.dot(normal) / denom;
      if distance >= 0.0 {
        return Some(distance);
      }
    }
    None
  }

  fn surface_normal(&self, _: &Point) -> Direction {
    -self.normal
  }

    fn texture_coord(&self, hit_point: &Point) -> TextureCoords {
        let mut x_axis = self.normal.cross(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0
        });

        if x_axis.magnitude() == 0.0 {
            x_axis = self.normal.cross(Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0
            });
        }

        let y_axis = self.normal.cross(x_axis.clone());
        let hit_vec = *hit_point - self.origin;

        TextureCoords {
            x: hit_vec.dot(x_axis) as f32,
            y: hit_vec.dot(y_axis) as f32
        }
    }
}

pub struct Scene {
  pub width: u32,
  pub height: u32,
  pub fov: f64,
  pub objects: Vec<Object>,
  pub lights: Vec<Light>
}

impl Scene {
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(ray).map(|distance| {
                let hit_point = ray.origin + ray.direction * distance;
                let normal = object.surface_normal(&hit_point);
                let texture_coord = object.texture_coord(&hit_point);
                Intersection::new(distance, hit_point, texture_coord, normal, object)
            }))
            .min_by(|i1, i2| i1.compare_to(&i2).unwrap())
    }
}

pub struct SceneBuilder {
  width: u32, 
  height:u32,
  fov: f64, 
  objects: Vec<Object>,
  lights: Vec<Light>
}

impl SceneBuilder {
  pub fn new(width: u32, height: u32) -> SceneBuilder {
    SceneBuilder {
      width: width, 
      height: height,
      fov: 90.0,
      objects: Vec::new(),
      lights: Vec::new()
    }
  }

  pub fn with_fov(mut self, fov: f64) -> SceneBuilder {
    self.fov = fov;
    self   
  }

  pub fn add_object(mut self, obj: Object) -> SceneBuilder {
    self.objects.push(obj);
    self
  }

  pub fn add_light(mut self, light: Light) -> SceneBuilder {
    self.lights.push(light);
    self
  }

  pub fn finish(self) -> Scene {
    Scene {
      width: self.width,
      height: self.height,
      fov: self.fov,
      objects: self.objects,
      lights: self.lights
    }
  }
}
