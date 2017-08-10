use objects::WorldPosition;
use cgmath::prelude::*;
use types::Point;
use raycast::{Ray, Intersection};

pub struct Quad {
    size: Point
}

impl Quad {
    pub fn intersects(&self, ray: &Ray, position: &WorldPosition) -> bool {
        let pmin = position.position;
        let pmax = position.position + self.size.to_vec();

        let tmin = (pmin.x - ray.origin.x) / ray.direction.x;
        let tmax = (pmax.x - ray.origin.x) / ray.direction.x;

        if tmin > tmax {
            let (tmin, tmax) = (tmax, tmin);
        }

        let tymin = (pmin.y - ray.origin.y) / ray.direction.y;
        let tymax = (pmax.y - ray.origin.y) / ray.direction.y;

        if tymin > tymax {
            let (tymin, tymax) = (tymax, tymin);
        }

        if tmin > tymax || tymin > tmax {
            return false;
        }

        if tymin > tmin {
            let tmin = tymin;
        }

        if tymax > tmax {
            let tmax = tymax;
        }

        let tzmin = (pmin.z - ray.origin.z) / ray.direction.z;
        let tzmax = (pmax.z - ray.origin.z) / ray.direction.z;

        if tzmin > tzmax {
            let (tzmin, tzmax) = (tzmax, tzmin);
        }

        if tmin > tzmax || tzmin > tmax {
            return false;
        }

        true
    }
}
