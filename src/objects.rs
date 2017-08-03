use raycast::{Intersectable,Intersection,Ray};
use types::{Color, Point, Direction};
use cgmath::prelude::*;
use light::Light;



#[derive(Debug)]
pub struct Coloration {
  Color(Color),
  Texture(DynamicImage)
}

impl Coloration {
  
}

#[derive(Debug)]
pub struct Material {
  pub color: Coloration,
  pub albedo: f32
}

#[derive(Debug)]
pub enum Object {
  Sphere(Sphere),
  Plane(Plane)
}

impl Object {
  pub fn color(&self) -> &Color {
    match *self {
      Object::Sphere(ref s) => &s.color,
      Object::Plane(ref p) => &p.color
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
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct Plane {
  pub origin: Point, 
  pub normal: Direction,
  pub color: Color
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
}

#[derive(Debug)]
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
      .filter_map(|object| object.intersect(ray).map(| distance | {
        let hit_point = ray.origin + ray.direction * distance;
        let normal = object.surface_normal(&hit_point);
        Intersection::new(distance, hit_point, normal, object)
      }))
      .min_by(| i1, i2 |  i1.compare_to(&i2).unwrap())
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
