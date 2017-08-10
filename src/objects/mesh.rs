use objects::{Material, TextureCoords, SurfaceType, Sphere, Structure, WorldPosition, Quad};
use types::{Point, Color, Direction};
use raycast::{Ray, Intersection};
use cgmath::prelude::*;
use cgmath::Vector3;
use wavefront_obj::obj;

struct Triangle {
    p1: Point,
    p2: Point,
    p3: Point
}


impl Triangle {
    pub fn from_obj_vertices(origin: &Point, v1: &obj::Vertex, v2: &obj::Vertex, v3: &obj::Vertex) -> Triangle {
        Triangle {
            p1: Point { x: origin.x - v1.x, y: origin.y - v1.y, z: origin.z - v1.z },
            p2: Point { x: origin.x - v2.x, y: origin.y - v2.y, z: origin.z - v2.z },
            p3: Point { x: origin.x - v3.x, y: origin.y - v3.y, z: origin.z - v3.z },
        }
    }

    pub fn surface_normal(&self) -> Direction {
        let vec_a = self.p2 - self.p1;
        let vec_b = self.p3 - self.p1;

        vec_a.cross(vec_b)
    }

    /// implements https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    pub fn intersects(&self, ray: &Ray) -> Option<(Direction, TextureCoords, f64)> {
        let vec_a = self.p2 - self.p1;
        let vec_b = self.p3 - self.p1;
        let normal = self.surface_normal();

        let pvec = ray.direction.cross(vec_b);
        let det = (vec_a).dot(pvec);

        // parallel
        if det.abs() < 1e-8 {
            //println!("PARALLEL: {:?} | {:?} | {:?}", vec_a, vec_b, det);
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - self.p1;
        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            //println!("U       : {:?} | {:?}", u, tvec);
            return None;
        }

        let qvec = tvec.cross(vec_a);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            //println!("V       : {:?} | {:?}", u, v);
            return None;
        }

        Some((-normal.normalize(), TextureCoords { x: 0.0, y: 0.0 }, vec_b.dot(qvec) * inv_det))
    }
}

pub struct Mesh {
    pub mesh: obj::Object,
    pub bb: (Point, Point)
}

impl Structure for Mesh {
    fn get_intersection(&self, ray: &Ray, position: &WorldPosition) -> Option<Intersection> {
        self.intersect(ray, position).map(|result| {
            let (normal, texc, distance) = result;
            let hit_point = ray.origin + ray.direction * distance;
            Intersection::new(
                distance,
                hit_point,
                texc,
                normal
            )
        })
    }
}

impl Mesh {
    fn intersect(&self, ray: &Ray, position: &WorldPosition) -> Option<(Direction, TextureCoords, f64)> {
        if !self.check_bb(ray, position) {
            //            return None;
        }

        self.mesh.geometry.iter().filter_map(|geom| {
            geom.shapes.iter().filter_map(|shape| {
                match shape.primitive {
                    obj::Primitive::Triangle(vidx1, vidx2, vidx3) => {
                        let v1 = self.mesh.vertices[vidx1.0];
                        let v2 = self.mesh.vertices[vidx2.0];
                        let v3 = self.mesh.vertices[vidx3.0];

                        let triangle = Triangle::from_obj_vertices(&position.position, &v1, &v2, &v3);
                        triangle.intersects(ray)
                    }
                    _ => None
                }
            }).min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
        }).min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
    }

    fn create_bounding_box(obj: &obj::Object) -> (Point, Point) {
        let first_vert = obj.vertices.get(0).unwrap();
        let min = (first_vert.x, first_vert.y, first_vert.z);
        let max = (first_vert.x, first_vert.y, first_vert.z);

        let (min, max) = obj.vertices.iter().fold((min, max), |(min, max), &v| {
            (
                (
                    min.0.min(v.x),
                    min.1.min(v.y),
                    min.2.min(v.z)
                ),
                (
                    max.0.max(v.x),
                    max.1.max(v.y),
                    max.2.max(v.z)
                )
            )
        });

        println!("BB:{:?}", (
            Point { x: min.0, y: min.1, z: min.2 },
            Point { x: max.0, y: max.1, z: max.2 }
        ));

        (
            Point { x: min.0, y: min.1, z: min.2 },
            Point { x: max.0, y: max.1, z: max.2 }
        )
    }

    fn check_bb(&self, ray: &Ray, position: &WorldPosition) -> bool {
        let pmin = position.position + self.bb.0;
        let pmax = position.position + self.bb.1;

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

        /*
                if ray.origin.eq(&Point::zero()) {
                    println!("{:?} . {:?} . {:?}", ray.direction, pmin, pmax);
                }
        */
        true
    }

    pub fn create(obj: obj::Object) -> Mesh {
        Mesh {
            bb: Mesh::create_bounding_box(&obj),
            mesh: obj
        }
    }
}