use objects::{Material, TextureCoords, SurfaceType, Structure, WorldPosition};
use types::{Point, Color, Direction};
use raycast::{Ray, Intersection};
use cgmath::prelude::*;
use cgmath::Vector3;

pub struct Plane {
    pub normal: Direction,
}

impl Plane {
    pub fn create(normal: Direction) -> Plane {
        Plane { normal }
    }

    fn intersect(&self, ray: &Ray, position: &WorldPosition) -> Option<f64> {
        let normal = self.normal;
        let denom = normal.dot(ray.direction);
        if denom > 1e-10 {
            let v = position.position - ray.origin;
            let distance = v.dot(normal) / denom;
            if distance >= 0.0 {
                return Some(distance);
            }
        }
        None
    }

    fn texture_coord(&self, hit_point: &Point, position: &WorldPosition) -> TextureCoords {
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
        let hit_vec = *hit_point - position.position;

        TextureCoords {
            x: hit_vec.dot(x_axis) as f32,
            y: hit_vec.dot(y_axis) as f32
        }
    }
}

impl Structure for Plane {
    fn get_intersection(&self, ray: &Ray, position: &WorldPosition) -> Option<Intersection> {
        self.intersect(ray, position).map(|distance| {
            let hit_point = ray.origin + ray.direction * distance;
            Intersection::new(
                distance,
                hit_point,
                self.texture_coord(&hit_point, position),
                -self.normal
            )
        })
    }
}