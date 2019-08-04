use cgmath::prelude::*;
use cgmath::{Quaternion, Vector3};
use image::{DynamicImage, GenericImage};
use raycast::{Intersection, IntersectionResult, Ray};
use types::{Color, Point, Scale};

pub mod mesh;
pub mod plane;
pub mod quad;
pub mod sphere;

pub use self::mesh::*;
pub use self::plane::*;
pub use self::quad::*;
pub use self::sphere::*;

#[derive(Clone)]
pub struct TextureCoords {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub enum SurfaceType {
    Diffuse,
    Reflective { reflectivity: f32 },
}

#[derive(Clone)]
pub enum Coloration {
    Color(Color),
    Texture(DynamicImage),
}

fn wrap(val: f32, bound: u32) -> u32 {
    let signed_bound = bound as i32;
    let float_coord = val * bound as f32;
    let wrapped_coord = (float_coord as i32) % signed_bound;
    if wrapped_coord < 0 {
        (wrapped_coord + signed_bound) as u32
    } else {
        wrapped_coord as u32
    }
}

impl Coloration {
    pub fn color(&self, coords: &TextureCoords) -> Color {
        match *self {
            Coloration::Color(ref c) => c.clone(),
            Coloration::Texture(ref tex) => {
                let tex_x = wrap(coords.x, tex.width());
                let tex_y = wrap(coords.y, tex.height());

                Color::from_rgba(tex.get_pixel(tex_x, tex_y))
            }
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pub color: Coloration,
    pub albedo: f32,
    pub surface: SurfaceType,
}

impl Material {
    pub fn new(color: Coloration, albedo: f32) -> Material {
        Material {
            color,
            albedo,
            surface: SurfaceType::Diffuse,
        }
    }

    pub fn diffuse_color(color: Color, albedo: f32) -> Material {
        Material {
            color: Coloration::Color(color),
            albedo,
            surface: SurfaceType::Diffuse,
        }
    }

    pub fn reflective_color(color: Color, albedo: f32, refl: f32) -> Material {
        Material {
            color: Coloration::Color(color),
            albedo,
            surface: SurfaceType::Reflective { reflectivity: refl },
        }
    }

    pub fn diffuse_texture(image: DynamicImage, albedo: f32) -> Material {
        Material {
            color: Coloration::Texture(image),
            albedo,
            surface: SurfaceType::Diffuse,
        }
    }
}

pub trait Structure {
    fn get_intersection(&self, ray: &Ray, position: &WorldPosition) -> Option<Intersection>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldPosition {
    pub position: Point,
    pub rotation: Quaternion<f64>,
    pub scale: Scale,
}

impl WorldPosition {
    pub fn translate(&self, vec: Point) -> Point {
        self.rotation.rotate_point(vec) * self.scale + self.position.to_vec()
    }
}

pub struct Object {
    material: Material,
    position: WorldPosition,
    structure: Box<Structure + Send + Sync>,
}

impl Object {
    pub fn intersect(&self, ray: &Ray) -> Option<IntersectionResult> {
        self.structure
            .get_intersection(ray, &self.position)
            .map(|intersection| {
                IntersectionResult::create(
                    &intersection,
                    self.color_at(intersection.texture_coord()),
                    self.material.albedo,
                    self.reflectivity_at(intersection.texture_coord()),
                )
            })
    }

    fn reflectivity_at(&self, texture_coordinates: TextureCoords) -> Option<f32> {
        match self.material.surface {
            SurfaceType::Reflective { reflectivity } => Some(reflectivity),
            _ => None,
        }
    }

    fn color_at(&self, texture_coordinates: TextureCoords) -> Color {
        self.material.color.color(&texture_coordinates)
    }
}

impl<E: Structure + Send + Sync> From<ObjectBuilder<E>> for Object
where
    E: 'static,
{
    fn from(builder: ObjectBuilder<E>) -> Self {
        Object {
            material: builder.material,
            structure: builder.structure,
            position: WorldPosition {
                position: builder.position,
                rotation: builder.rotation,
                scale: builder.scale,
            },
        }
    }
}

pub struct ObjectBuilder<E: Structure + Send + Sync> {
    material: Material,
    structure: Box<E>,
    position: Point,
    rotation: Quaternion<f64>,
    scale: Scale,
}

impl<E: Structure + Send + Sync> ObjectBuilder<E> {
    pub fn create_for(object: E) -> ObjectBuilder<E> {
        ObjectBuilder {
            material: Material {
                color: Coloration::Color(Color::from_rgb(0.5, 0.5, 0.5)),
                surface: SurfaceType::Diffuse,
                albedo: 0.1,
            },
            position: Point::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            structure: Box::new(object),
            scale: 1.0,
        }
    }

    pub fn scale(mut self, scale: Scale) -> ObjectBuilder<E> {
        self.scale = scale;
        self
    }

    pub fn rotation(mut self, rotation: Quaternion<f64>) -> ObjectBuilder<E> {
        self.rotation = rotation.normalize();
        self
    }

    pub fn at_position(mut self, position: Point) -> ObjectBuilder<E> {
        self.position = position;
        self
    }

    pub fn with_material(mut self, material: Material) -> ObjectBuilder<E> {
        self.material = material;
        self
    }
}

#[cfg(test)]
mod test {
    use cgmath::{Quaternion, Zero};
    use objects::{Object, ObjectBuilder, Sphere, WorldPosition};
    use types::Point;

    #[test]
    fn test_create_sphere() {
        let obj: Object = ObjectBuilder::create_for(Sphere::create(20.0))
            .at_position(Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            })
            .into();

        assert_eq!(
            obj.position,
            WorldPosition {
                position: Point {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                },
                rotation: Quaternion::zero(),
                scale: 1.0
            }
        );
        assert_eq!(obj.material.albedo, 0.1);
    }
}
