use cgmath::prelude::*;
use objects::{Sphere, Structure, TextureCoords, WorldPosition};
use raycast::{Intersection, Ray, RayType};
use std::cmp::{max, min};
use types::{Direction, Point, Scale};
use wavefront_obj::obj;

#[derive(Debug, Clone)]
struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub fn intersects(&self, ray: &Ray, position: &WorldPosition) -> bool {
        let pmin = position.translate(self.min);
        let pmax = position.translate(self.max);

        let tx1 = (pmin.x - ray.origin.x) * ray.inv_direction.x;
        let tx2 = (pmax.x - ray.origin.x) * ray.inv_direction.x;

        let mut tmin = tx1.min(tx2);
        let mut tmax = tx1.max(tx2);

        let ty1 = (pmin.y - ray.origin.y) * ray.inv_direction.y;
        let ty2 = (pmax.y - ray.origin.y) * ray.inv_direction.y;

        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));

        let tz1 = (pmin.z - ray.origin.z) * ray.inv_direction.z;
        let tz2 = (pmax.z - ray.origin.z) * ray.inv_direction.z;

        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));

        tmax >= tmin && tmax >= 0.0
    }
}

const EPSILON: f64 = 1e-13;

pub struct Triangle {
    p1: Point,
    p2: Point,
    p3: Point,
    normals: Option<(Direction, Direction, Direction)>,
}

impl Triangle {
    pub fn from_obj_vertices(v1: &obj::Vertex, v2: &obj::Vertex, v3: &obj::Vertex) -> Triangle {
        Triangle {
            p1: Point {
                x: v1.x,
                y: v1.y,
                z: v1.z,
            },
            p2: Point {
                x: v2.x,
                y: v2.y,
                z: v2.z,
            },
            p3: Point {
                x: v3.x,
                y: v3.y,
                z: v3.z,
            },
            normals: None,
        }
    }

    fn center(&self) -> Point {
        Point {
            x: (self.p1.x + self.p2.x + self.p3.x) / 3.0,
            y: (self.p1.y + self.p2.y + self.p3.y) / 3.0,
            z: (self.p1.z + self.p2.z + self.p3.z) / 3.0,
        }
    }

    fn with_normals(mut self, n1: &obj::Normal, n2: &obj::Normal, n3: &obj::Normal) -> Triangle {
        self.normals = Some((
            Direction {
                x: n1.x,
                y: n1.y,
                z: n1.z,
            },
            Direction {
                x: n2.x,
                y: n2.y,
                z: n2.z,
            },
            Direction {
                x: n3.x,
                y: n3.y,
                z: n3.z,
            },
        ));
        self
    }

    pub fn surface_normal(&self, u: f64, v: f64, position: &WorldPosition) -> Direction {
        if let Some((n1, n2, n3)) = self.normals {
            let n1 = position.rotation.rotate_vector(n1);
            let n2 = position.rotation.rotate_vector(n2);
            let n3 = position.rotation.rotate_vector(n3);
            let w = (1.0 - u - v);
            (n1 * w + n2 * u + n3 * v).normalize()
        } else {
            let vec_a = self.p2 - self.p1;
            let vec_b = self.p3 - self.p1;

            vec_a.cross(vec_b).normalize()
        }
    }

    /// implements möller-trumbore
    /// http://webserver2.tecgraf.puc-rio.br/~mgattass/cg/trbRR/Fast%20MinimumStorage%20RayTriangle%20Intersection.pdf
    pub fn intersects(
        &self,
        ray: &Ray,
        position: &WorldPosition,
    ) -> Option<(Direction, TextureCoords, f64)> {
        let point_0 = position.translate(self.p1);
        let point_1 = position.translate(self.p2);
        let point_2 = position.translate(self.p3);
        let edge_1 = point_1 - point_0;
        let edge_2 = point_2 - point_0;

        let pvec = ray.direction.cross(edge_2);

        let det = edge_1.dot(pvec);
        if det < EPSILON && det > -EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        let tvec = ray.origin - point_0;
        let u = tvec.dot(pvec) * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(edge_1);
        let v = ray.direction.dot(qvec) * inv_det;

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge_2.dot(qvec) * inv_det;

        let normal = self.surface_normal(u, v, position);

        Some((normal, TextureCoords { x: 0.0, y: 0.0 }, t))
    }
}

pub struct Mesh {
    mesh: obj::Object,
    root: MeshTreeNode,
}

enum MeshTreeNode {
    Node(BoundingBox, Box<MeshTreeNode>, Box<MeshTreeNode>),
    Leaf(BoundingBox, Vec<Triangle>),
}

#[inline]
fn min4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a.min(b).min(c).min(d)
}

#[inline]
fn max4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a.max(b).max(c).max(d)
}

#[derive(Debug)]
enum SplitRule {
    X(f64),
    Y(f64),
    Z(f64),
}

enum SplitResult<T> {
    Left(T),
    Right(T),
}

impl SplitRule {
    fn sort_to(&self, t: Triangle) -> SplitResult<Triangle> {
        let center = t.center();
        match self {
            SplitRule::X(ref bp) => {
                if center.x < *bp {
                    SplitResult::Left(t)
                } else {
                    SplitResult::Right(t)
                }
            }
            SplitRule::Z(ref bp) => {
                if center.z < *bp {
                    SplitResult::Left(t)
                } else {
                    SplitResult::Right(t)
                }
            }
            SplitRule::Y(ref bp) => {
                if center.y < *bp {
                    SplitResult::Left(t)
                } else {
                    SplitResult::Right(t)
                }
            }
        }
    }
}

impl MeshTreeNode {
    pub fn create(triangles: Vec<Triangle>) -> MeshTreeNode {
        let bb = MeshTreeNode::create_bounding_box(&triangles);

        if triangles.len() <= 250 {
            return MeshTreeNode::Leaf(bb, triangles);
        }

        let (left, right) = MeshTreeNode::split_triangles(&bb, triangles);

        MeshTreeNode::Node(bb, Box::new(left), Box::new(right))
    }

    fn split_triangles(bb: &BoundingBox, triangles: Vec<Triangle>) -> (MeshTreeNode, MeshTreeNode) {
        let delta_x = (bb.min.x - bb.max.x).abs();
        let delta_y = (bb.min.y - bb.max.y).abs();
        let delta_z = (bb.min.z - bb.max.z).abs();

        let split_rule = if delta_x > delta_y && delta_x > delta_z {
            // split in x
            SplitRule::X(bb.min.x + delta_x / 2.0)
        } else if delta_y > delta_x && delta_y > delta_z {
            // split in y
            SplitRule::Y(bb.min.y + delta_y / 2.0)
        } else {
            // split in z
            SplitRule::Z(bb.min.z + delta_z / 2.0)
        };

        let mut left = Vec::new();
        let mut right = Vec::new();
        for tri in triangles {
            match split_rule.sort_to(tri) {
                SplitResult::Left(tri) => left.push(tri),
                SplitResult::Right(tri) => right.push(tri),
            }
        }
        (MeshTreeNode::create(left), MeshTreeNode::create(right))
    }

    fn create_bounding_box(triangles: &Vec<Triangle>) -> BoundingBox {
        let first_vert = triangles.get(0).unwrap().p1;
        let pmin = first_vert.clone();
        let pmax = first_vert.clone();

        let (pmin, pmax) = triangles.iter().fold((pmin, pmax), |(pmin, pmax), t| {
            (
                Point {
                    x: min4(pmin.x, t.p1.x, t.p2.x, t.p3.x),
                    y: min4(pmin.y, t.p1.y, t.p2.y, t.p3.y),
                    z: min4(pmin.z, t.p1.z, t.p2.z, t.p3.z),
                },
                Point {
                    x: max4(pmax.x, t.p1.x, t.p2.x, t.p3.x),
                    y: max4(pmax.y, t.p1.y, t.p2.y, t.p3.y),
                    z: max4(pmax.z, t.p1.z, t.p2.z, t.p3.z),
                },
            )
        });

        BoundingBox {
            min: pmin,
            max: pmax,
        }
    }

    fn intersect(
        &self,
        ray: &Ray,
        position: &WorldPosition,
    ) -> Option<(Direction, TextureCoords, f64)> {
        match self {
            MeshTreeNode::Leaf(bbox, triangles) => {
                if !bbox.intersects(ray, position) {
                    return None;
                }

                triangles
                    .iter()
                    .filter_map(|triangle| triangle.intersects(ray, position))
                    .min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
            }
            MeshTreeNode::Node(bbox, a, b) => {
                if !bbox.intersects(ray, position) {
                   return None;
                }

                let left_match = a.intersect(ray, position);
                let right_match = b.intersect(ray, position);

                match (left_match, right_match) {
                    (Some(x), None) => return Some(x),
                    (None, Some(x)) => return Some(x),
                    (None, None) => None,
                    (Some(x), Some(y)) => {
                        if x.2 < y.2 {
                            Some(x)
                        } else {
                            Some(y)
                        }
                    }
                }
            }
        }
    }
}

impl Structure for Mesh {
    fn get_intersection(&self, ray: &Ray, position: &WorldPosition) -> Option<Intersection> {
        self.intersect(ray, position).map(|result| {
            let (normal, texc, distance) = result;
            let hit_point = ray.origin + ray.direction * distance;
            Intersection::new(distance, hit_point, texc, normal)
        })
    }
}

impl Mesh {
    fn intersect(
        &self,
        ray: &Ray,
        position: &WorldPosition,
    ) -> Option<(Direction, TextureCoords, f64)> {
        self.root.intersect(ray, position)
    }

    pub fn create(obj: obj::Object) -> Mesh {
        Mesh {
            root: MeshTreeNode::create(Mesh::build_triangles(&obj)),
            mesh: obj,
        }
    }

    fn build_triangles(obj: &obj::Object) -> Vec<Triangle> {
        obj.geometry
            .iter()
            .map(|geom| {
                geom.shapes
                    .iter()
                    .filter_map(|shape| match shape.primitive {
                        obj::Primitive::Triangle(vidx1, vidx2, vidx3) => {
                            let v1 = obj.vertices[vidx1.0];
                            let v2 = obj.vertices[vidx2.0];
                            let v3 = obj.vertices[vidx3.0];

                            if vidx1.2.is_some() && vidx2.2.is_some() && vidx3.2.is_some() {
                                let n1 = obj.normals[vidx1.2.unwrap()];
                                let n2 = obj.normals[vidx2.2.unwrap()];
                                let n3 = obj.normals[vidx3.2.unwrap()];
                                Some(
                                    Triangle::from_obj_vertices(&v1, &v2, &v3)
                                        .with_normals(&n1, &n2, &n3),
                                )
                            } else {
                                Some(Triangle::from_obj_vertices(&v1, &v2, &v3))
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<Triangle>>()
            })
            .flatten()
            .collect()
    }
}
