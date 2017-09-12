use objects::{Material, TextureCoords, SurfaceType, Sphere, Structure, WorldPosition, Quad};
use types::{Point, Color, Direction, Scale};
use raycast::{Ray, Intersection};
use cgmath::prelude::*;
use cgmath::Vector3;
use wavefront_obj::obj;

struct Triangle {
    p1: Point,
    p2: Point,
    p3: Point,
    normals: Option<(Direction, Direction, Direction)>
}


impl Triangle {
    pub fn from_obj_vertices(position: &WorldPosition, scale: &Scale, v1: &obj::Vertex, v2: &obj::Vertex, v3: &obj::Vertex) -> Triangle {
        let rot = position.rotation;
        let origin = position.position;

        Triangle {
            p1: rot.rotate_point(Point { x: v1.x, y: v1.y, z: v1.z }) * *scale + origin.to_vec(),
            p2: rot.rotate_point(Point { x: v2.x, y: v2.y, z: v2.z }) * *scale + origin.to_vec(),
            p3: rot.rotate_point(Point { x: v3.x, y: v3.y, z: v3.z }) * *scale + origin.to_vec(),
            normals: None
        }
    }

    pub fn surface_normal(&self, u: f64, v: f64) -> Direction {
        if let Some((n1, n2, n3)) = self.normals {
            (n1 * u + n2 * v + n3 * (1.0 - u - v)).normalize()
        } else {
            let vec_a = self.p2 - self.p1;
            let vec_b = self.p3 - self.p1;

            vec_a.cross(vec_b).normalize()
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<(Direction, TextureCoords, f64)> {
        let vec_a = self.p2 - self.p1;
        let vec_b = self.p3 - self.p1;
        let normal = vec_a.cross(vec_b);
        let area2 = normal.magnitude();

        let n_dot_dir = normal.dot(ray.direction);
        if n_dot_dir.abs() < 1e-10 {
            return None;
        }

        let d = normal.dot(self.p1.to_vec());
        let t = (normal.dot(ray.origin.to_vec()) + d) / n_dot_dir;

        if t < 0.0 { return None; }

        let p = ray.origin + t * ray.direction;

        if normal.dot(vec_a.cross(p - self.p1)) < 0.0 {
            return None;
        }

        let c = (self.p3 - self.p2).cross(p - self.p2);
        let u = c.magnitude() / area2;
        if normal.dot(c) < 0.0 { return None; }

        let c = (self.p1 - self.p3).cross(p - self.p3);
        let v = c.magnitude() / area2;
        if normal.dot(c) < 0.0 { return None; }

        let normal = self.surface_normal(u, v);

        Some((normal, TextureCoords { x: 0.0, y: 0.0 }, t))
    }

    fn with_normals(mut self, position: &WorldPosition, n1: &obj::Normal, n2: &obj::Normal, n3: &obj::Normal) -> Triangle {
        self.normals = Some((
            position.rotation.rotate_vector(Direction { x: n1.x, y: n1.y, z: n1.z }),
            position.rotation.rotate_vector(Direction { x: n2.x, y: n2.y, z: n2.z }),
            position.rotation.rotate_vector(Direction { x: n3.x, y: n3.y, z: n3.z }),
        ));
        self
    }
}

pub struct Mesh {
    pub mesh: obj::Object,
    pub bb: (Point, Sphere)
}

impl Structure for Mesh {
    fn get_intersection(&self, ray: &Ray, position: &WorldPosition, scale: &Scale) -> Option<Intersection> {
        self.intersect(ray, position, scale).map(|result| {
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
    fn intersect(&self, ray: &Ray, position: &WorldPosition, scale: &Scale) -> Option<(Direction, TextureCoords, f64)> {
        if !self.check_bb(ray, position, scale) {
            return None;
        }

        self.mesh.geometry.iter().filter_map(|geom| {
            geom.shapes.iter().filter_map(|shape| {
                match shape.primitive {
                    obj::Primitive::Triangle(vidx1, vidx2, vidx3) => {
                        let v1 = self.mesh.vertices[vidx1.0];
                        let v2 = self.mesh.vertices[vidx2.0];
                        let v3 = self.mesh.vertices[vidx3.0];

                        let triangle = if vidx1.2.is_some() && vidx2.2.is_some() && vidx3.2.is_some() {
                            let n1 = self.mesh.normals[vidx1.2.unwrap()];
                            let n2 = self.mesh.normals[vidx2.2.unwrap()];
                            let n3 = self.mesh.normals[vidx3.2.unwrap()];
                            Triangle::from_obj_vertices(&position, scale, &v1, &v2, &v3).with_normals(&position, &n1, &n2, &n3)
                        } else {
                            Triangle::from_obj_vertices(&position, scale, &v1, &v2, &v3)
                        };

                        triangle.intersects(ray)
                    }
                    _ => None
                }
            }).min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
        }).min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
    }

    fn create_bounding_box(obj: &obj::Object) -> (Point, Sphere) {
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

        let a = Point { x: min.0, y: min.1, z: min.2 };
        let b = Point { x: max.0, y: max.1, z: max.2 };

        let center = Point::midpoint(a, b);

        let distance = center.distance(a);

        return (center, Sphere::create(distance))
    }

    fn check_bb(&self, ray: &Ray, position: &WorldPosition, scale: &Scale) -> bool {
        let center = self.bb.0 + position.position.to_vec();

        self.bb.1.get_intersection(ray, &WorldPosition { position: center, rotation: position.rotation }, scale).is_some()
    }

    pub fn create(obj: obj::Object) -> Mesh {
        Mesh {
            bb: Mesh::create_bounding_box(&obj),
            mesh: obj
        }
    }
}