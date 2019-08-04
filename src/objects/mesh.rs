use cgmath::prelude::*;
use objects::{Sphere, Structure, TextureCoords, WorldPosition};
use raycast::{Intersection, Ray, RayType};
use types::{Direction, Point, Scale};
use wavefront_obj::obj;

struct BoundingBox {
    min: Point,
    max: Point
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
        position: &WorldPosition
    ) -> Option<(Direction, TextureCoords, f64)> {
        let point_0 = position.translate(self.p1);
        let point_1 = position.translate(self.p2);
        let point_2 = position.translate(self.p3);
        let edge_1 = point_1 - point_0;
        let edge_2 = point_2 - point_0;

        let pvec = ray.direction.cross(edge_2);

        let det = edge_1.dot(pvec);
        if det < EPSILON {
            return None;
        }

        let tvec = ray.origin - point_0;
        let u = tvec.dot(pvec);

        if u < 0.0 || u > det {
            return None;
        }

        let qvec = tvec.cross(edge_1);
        let v = ray.direction.dot(qvec);

        if v < 0.0 || u + v > det {
            return None;
        }

        let t = edge_2.dot(qvec);
        let inv_det = 1.0 / det;
        let t = t * inv_det;
        let u = u * inv_det;
        let v = v * inv_det;

        let normal = self.surface_normal(u, v, position);

        Some((normal, TextureCoords { x: 0.0, y: 0.0 }, t))
    }
}

pub struct Mesh {
    mesh: obj::Object,
    bb: BoundingBox,
    triangles: Vec<Triangle>,
    root: MeshTreeNode
}

enum MeshTreeNode {
    Node(Box<MeshTreeNode>, Box<MeshTreeNode>),
    Leaf(Vec<Triangle>)
}

impl Structure for Mesh {
    fn get_intersection(
        &self,
        ray: &Ray,
        position: &WorldPosition
    ) -> Option<Intersection> {
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
        position: &WorldPosition
    ) -> Option<(Direction, TextureCoords, f64)> {
        if !self.check_bb(ray, position) {
            return None;
        }

        self.triangles
            .iter()
            .filter_map(|triangle| triangle.intersects(ray, position))
            .min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
    }

    fn create_bounding_box(obj: &obj::Object) -> BoundingBox {
        let first_vert = obj.vertices.get(0).unwrap();
        let min = (first_vert.x, first_vert.y, first_vert.z);
        let max = (first_vert.x, first_vert.y, first_vert.z);

        let (min, max) = obj.vertices.iter().fold((min, max), |(min, max), &v| {
            (
                (min.0.min(v.x), min.1.min(v.y), min.2.min(v.z)),
                (max.0.max(v.x), max.1.max(v.y), max.2.max(v.z)),
            )
        });

        let a = Point {
            x: min.0,
            y: min.1,
            z: min.2,
        };
        let b = Point {
            x: max.0,
            y: max.1,
            z: max.2,
        };

        BoundingBox { min: a, max: b }
    }

    fn check_bb(&self, ray: &Ray, position: &WorldPosition) -> bool {
        self.bb.intersects(ray, position)
    }

    pub fn create(obj: obj::Object) -> Mesh {
        Mesh {
            bb: Mesh::create_bounding_box(&obj),
            triangles: Mesh::build_triangles(&obj),
            root: MeshTreeNode::Leaf(Mesh::build_triangles(&obj)),
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

#[cfg(test)]
mod test {
    use cgmath::{One, Quaternion};
    use objects::mesh::{Mesh, Triangle};
    use objects::ObjectBuilder;
    use objects::{Sphere, Structure, WorldPosition};
    use raycast::IntersectionResult;
    use raycast::Ray;
    use raycast::SurfaceProperties;
    use scene::Scene;
    use types::Color;
    use types::{Direction, Point, Scale};
    use wavefront_obj;

    #[test]
    /*fn test_triangle_intersection() {
        let tri = Triangle {
            p1: Point { x: 2.0, y: 0.0, z: 2.0 },
            p2: Point { x: 2.0, y: 0.0, z: -2.0 },
            p3: Point { x: -2.0, y: 0.0, z: -2.0 },
            normals: None,
        };

        let ray = Ray { origin: Point { x: 0.0, y: -1.0, z: 0.0 }, direction: Direction { x: 0.0, y: 1.0, z: 0.0 } };

        let intersections = tri.intersects(&ray);

        assert_eq!(true, intersections.is_some());
        let inter = intersections.unwrap();
        let pos = inter.0;
        let distance = inter.2;
        let hit_point = ray.origin + ray.direction * distance;
        assert_eq!(distance, 1.0, "distance wrong");
        assert_eq!(hit_point.x, 0.0, "x coord wrong");
        assert_eq!(hit_point.y, 0.0, "y coord wrong");
        assert_eq!(hit_point.z, 0.0, "z coord wrong");
    }*/
    #[test]
    fn test_simple_mesh_intersection() {
        let simple = String::from(
            r#"
            v  2.0 0.0  2.0  # 1
            v  2.0 0.0 -2.0  # 2
            v -2.0 0.0 -2.0  # 3
            f 1 2 3
        "#,
        );
        let model_read = wavefront_obj::obj::parse(simple);
        if let Err(err) = model_read {
            panic!("{:?}", err);
        }

        let read_uw = model_read.unwrap();
        let model = read_uw.objects.get(0);

        let zero = Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let mesh = Mesh {
            mesh: model.unwrap().clone(),
            bb: (zero.clone(), Sphere::create(2.0)),
        };

        let ray = Ray {
            origin: Point {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            direction: Direction {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        };

        let intersections = mesh.get_intersection(
            &ray,
            &WorldPosition {
                position: zero.clone(),
                rotation: Quaternion::one(),
            },
            &1.0f64,
        );

        assert_eq!(true, intersections.is_some());
    }

    #[test]
    fn test_shadow_ray_collides() {
        let simple = String::from(
            r#"
            v  2.0 0.0  2.0  # 1
            v  2.0 0.0 -2.0  # 2
            v -2.0 0.0 -2.0  # 3
            f 1 2 3
        "#,
        );
        let model_read = wavefront_obj::obj::parse(simple);
        if let Err(err) = model_read {
            panic!("{:?}", err);
        }

        let read_uw = model_read.unwrap();
        let model = read_uw.objects.get(0);

        let zero = Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let mesh = Mesh {
            mesh: model.unwrap().clone(),
            bb: (zero.clone(), Sphere::create(2.0)),
        };

        let scene = Scene {
            width: 100,
            height: 100,
            fov: 90.0,
            objects: vec![ObjectBuilder::create_for(mesh).into()],
            lights: Vec::new(),
        };

        let to_light = Direction {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let ray = Ray {
            origin: Point {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            direction: to_light.clone(),
        };
        let result = scene.trace(&ray);
        assert!(result.is_some());
        let inter = IntersectionResult {
            distance: 0.0,
            hit_point: Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            surface_normal: Vector {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            surface: SurfaceProperties {
                albedo: 0.0,
                color: Color::from_rgb(0.0, 0.0, 0.0),
                reflectivity: None,
            },
        };

        let ray = Ray::create_shadow_ray(to_light.clone(), &inter);
        let result = scene.trace(&ray);
        assert!(result.is_some());
    }
}
