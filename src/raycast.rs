use cgmath::prelude::*;
use objects::TextureCoords;
use scene::Scene;
use types::{Color, Direction, Point};

use std::cmp::Ordering;

pub struct Intersection {
    distance: f64,
    surface_normal: Direction,
    hit_point: Point,
    tex_coord: TextureCoords,
}

impl Intersection {
    pub fn new(
        distance: f64,
        hit_point: Point,
        tex_coord: TextureCoords,
        surface_normal: Direction,
    ) -> Intersection {
        Intersection {
            distance,
            hit_point,
            surface_normal,
            tex_coord,
        }
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn hit_point(&self) -> Point {
        self.hit_point
    }

    pub fn surface_normal(&self) -> Direction {
        self.surface_normal
    }

    pub fn texture_coord(&self) -> TextureCoords {
        self.tex_coord.clone()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RayType {
    Prime,
    Reflection,
    Shadow,
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Direction,
    pub inv_direction: Direction,
    pub ray_type: RayType,
}

impl Ray {
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();
        let aspect_ratio = scene.width as f64 / scene.height as f64;
        let sensor_x =
            (((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0) * aspect_ratio * fov_adjustment;
        let sensor_y = (1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0) * fov_adjustment;
        let direction = Direction {
            x: sensor_x,
            y: sensor_y,
            z: -1.0,
        }
            .normalize();
        Ray {
            origin: Point::new(0.0, 0.0, 0.0),
            inv_direction: Direction {
                x: 1.0 / direction.x,
                y: 1.0 / direction.y,
                z: 1.0 / direction.z
            },
            direction: direction,
            ray_type: RayType::Prime,
        }
    }

    pub fn create_reflection(ray_direction: &Direction, int: &IntersectionResult) -> Ray {
        let direction = ray_direction - (2.0 * ray_direction.dot(int.surface_normal()) * int.surface_normal());
        Ray {
            origin: int.reflection_origin(),
            inv_direction: Direction {
                x: 1.0 / direction.x,
                y: 1.0 / direction.y,
                z: 1.0 / direction.z
            },
            direction: direction,
            ray_type: RayType::Reflection,
        }
    }

    pub fn create_shadow_ray(direction_to_light: Direction, int: &IntersectionResult) -> Ray {
        Ray {
            origin: int.reflection_origin(),
            inv_direction: Direction {
                x: 1.0 / direction_to_light.x,
                y: 1.0 / direction_to_light.y,
                z: 1.0 / direction_to_light.z
            },
            direction: direction_to_light,
            ray_type: RayType::Shadow,
        }
    }
}

#[derive(Debug)]
pub struct IntersectionResult {
    distance: f64,
    hit_point: Point,
    surface_normal: Direction,
    surface: SurfaceProperties,
}

impl PartialEq for IntersectionResult {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }

    fn ne(&self, other: &Self) -> bool {
        self.distance.ne(&other.distance)
    }
}

impl PartialOrd for IntersectionResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }

    fn lt(&self, other: &Self) -> bool {
        self.distance.lt(&other.distance)
    }

    fn le(&self, other: &Self) -> bool {
        self.distance.le(&other.distance)
    }

    fn gt(&self, other: &Self) -> bool {
        self.distance.gt(&other.distance)
    }

    fn ge(&self, other: &Self) -> bool {
        self.distance.ge(&other.distance)
    }
}

impl Eq for IntersectionResult {}

impl Ord for IntersectionResult {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.distance.partial_cmp(&other.distance) {
            Some(ord) => ord,
            None => Ordering::Equal,
        }
    }
}

impl IntersectionResult {
    pub fn create(
        intersection: &Intersection,
        color: Color,
        albedo: f32,
        reflectivity: Option<f32>,
    ) -> IntersectionResult {
        IntersectionResult {
            distance: intersection.distance(),
            surface_normal: intersection.surface_normal(),
            hit_point: intersection.hit_point(),
            surface: SurfaceProperties {
                reflectivity: reflectivity,
                albedo: albedo,
                color: color,
            },
        }
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn hit_point(&self) -> &Point {
        &self.hit_point
    }

    pub fn reflection_origin(&self) -> Point {
        self.hit_point + self.surface_normal * 1e-13
    }

    pub fn surface_normal(&self) -> Direction {
        self.surface_normal
    }

    pub fn albedo(&self) -> f32 {
        self.surface.albedo
    }

    pub fn color(&self) -> Color {
        self.surface.color
    }

    pub fn reflectivity(&self) -> Option<f32> {
        match self.surface.reflectivity {
            Some(r) => {
                if r < 1e-10 {
                    None
                } else {
                    Some(r)
                }
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct SurfaceProperties {
    pub albedo: f32,
    pub color: Color,
    pub reflectivity: Option<f32>,
}
