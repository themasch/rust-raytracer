use cgmath::prelude::*;
use objects::{Sphere, Structure, TextureCoords, WorldPosition};
use raycast::{Intersection, Ray, RayType};
use types::{Direction, Point, Scale};
use wavefront_obj::obj;

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
            (n1 * u + n2 * v + n3 * (1.0 - u - v)).normalize()
        } else {
            let vec_a = self.p2 - self.p1;
            let vec_b = self.p3 - self.p1;

            vec_a.cross(vec_b).normalize()
        }
    }

    pub fn intersects(
        &self,
        ray: &Ray,
        position: &WorldPosition,
        scale: &Scale,
    ) -> Option<(Direction, TextureCoords, f64)> {
        let point1 = position.rotation.rotate_point(self.p1) * *scale + position.position.to_vec();
        let point2 = position.rotation.rotate_point(self.p2) * *scale + position.position.to_vec();
        let point3 = position.rotation.rotate_point(self.p3) * *scale + position.position.to_vec();
        let vec_a = point2 - point1;
        let vec_b = point3 - point1;
        let normal = vec_a.cross(vec_b);
        let area2 = normal.magnitude();

        let n_dot_dir = normal.dot(ray.direction);
        if n_dot_dir.abs() < EPSILON {
            // ray and plane are parallel
            return None;
        }

        let d = normal.dot(point1.to_vec());
        let dt = normal.dot(ray.origin.to_vec());
        let t = (dt + d) / n_dot_dir;

        if t < 0.0 {
            // plane is behind the start if the ray
            return None;
        }

        let p = ray.origin + t * ray.direction;
        if normal.dot(vec_a.cross(p - point1)) < 0.0 {
            return None;
        }

        let c = (point3 - point2).cross(p - point2);
        let u = c.magnitude() / area2;
        if normal.dot(c) < 0.0 {
            return None;
        }

        let c = (point1 - point3).cross(p - point3);
        let v = c.magnitude() / area2;
        if normal.dot(c) < 0.0 {
            return None;
        }

        let normal = self.surface_normal(u, v, position);

        if ray.ray_type == RayType::Shadow {
            //println!("shadow ray intersection at {:?}, {:?}, {:?}", (u, v), normal, ray.direction);
            //println!("hit point {:?}",  p);
        }

        Some((normal, TextureCoords { x: 0.0, y: 0.0 }, t))
    }
}

pub struct Mesh {
    pub mesh: obj::Object,
    pub bb: (Point, Sphere),
    pub triangles: Vec<Triangle>,
}

impl Structure for Mesh {
    fn get_intersection(
        &self,
        ray: &Ray,
        position: &WorldPosition,
        scale: &Scale,
    ) -> Option<Intersection> {
        self.intersect(ray, position, scale).map(|result| {
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
        scale: &Scale,
    ) -> Option<(Direction, TextureCoords, f64)> {
        if !self.check_bb(ray, position, scale) {
            return None;
        }

        self.triangles
            .iter()
            .filter_map(|triangle| triangle.intersects(ray, position, scale))
            .min_by(|f1, f2| f1.2.partial_cmp(&f2.2).unwrap())
    }

    fn create_bounding_box(obj: &obj::Object) -> (Point, Sphere) {
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

        let center = Point::midpoint(a, b);
        let distance = center.distance(a);

        return (center, Sphere::create(distance));
    }

    fn check_bb(&self, ray: &Ray, position: &WorldPosition, scale: &Scale) -> bool {
        let center = self.bb.0 + position.position.to_vec();
        self.bb
            .1
            .get_intersection(
                ray,
                &WorldPosition {
                    position: center,
                    rotation: position.rotation,
                },
                scale,
            )
            .is_some()
    }

    pub fn create(obj: obj::Object) -> Mesh {
        Mesh {
            bb: Mesh::create_bounding_box(&obj),
            triangles: Mesh::build_triangles(&obj),
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
