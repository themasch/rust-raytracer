use cgmath::prelude::*;
use objects::{Structure, TextureCoords, WorldPosition};
use raycast::{Intersection, Ray};
use types::{Direction, Point, Scale};

use std::f32::consts::PI;

pub struct Sphere {
    pub radius: f64,
}

impl Sphere {
    pub fn create(radius: f64) -> Sphere {
        Sphere { radius }
    }

    fn intersect(&self, ray: &Ray, position: &WorldPosition, scale: &Scale) -> Option<f64> {
        let l = position.position - ray.origin;
        let adj2 = l.dot(ray.direction);

        let d2 = l.dot(l) - adj2.powi(2);
        let radius2 = (self.radius * scale).powi(2);

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

    fn surface_normal(&self, hit_point: &Point, position: &WorldPosition) -> Direction {
        (*hit_point - position.position).normalize()
    }

    fn texture_coord(
        &self,
        hit_point: &Point,
        position: &WorldPosition,
        scale: &Scale,
    ) -> TextureCoords {
        let hit_vec = *hit_point - position.position;
        TextureCoords {
            x: (1.0 + (hit_vec.z.atan2(hit_vec.x) as f32) / PI) * 0.5,
            y: (hit_vec.y / (self.radius * scale)).acos() as f32 / PI,
        }
    }
}

impl Structure for Sphere {
    fn get_intersection(
        &self,
        ray: &Ray,
        position: &WorldPosition,
        scale: &Scale,
    ) -> Option<Intersection> {
        self.intersect(ray, position, scale).map(|distance| {
            let hit_point = ray.origin + ray.direction * distance;
            Intersection::new(
                distance,
                hit_point,
                self.texture_coord(&hit_point, position, scale),
                self.surface_normal(&hit_point, position),
            )
        })
    }
}
