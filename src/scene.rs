use raycast::{Ray, IntersectionResult};
use objects::Object;
use light::Light;

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub objects: Vec<Object>,
    pub lights: Vec<Light>
}

impl Scene {
    pub fn trace(&self, ray: &Ray) -> Option<IntersectionResult> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(ray))
            .filter(|intersection| intersection.distance() > 0.0)
            .min()
    }
}

pub struct SceneBuilder {
    width: u32,
    height: u32,
    fov: f64,
    objects: Vec<Object>,
    lights: Vec<Light>
}

impl SceneBuilder {
    pub fn new(width: u32, height: u32) -> SceneBuilder {
        SceneBuilder {
            width: width,
            height: height,
            fov: 90.0,
            objects: Vec::new(),
            lights: Vec::new()
        }
    }

    pub fn with_fov(mut self, fov: f64) -> SceneBuilder {
        self.fov = fov;
        self
    }

    pub fn add_object(mut self, obj: Object) -> SceneBuilder {
        self.objects.push(obj);
        self
    }

    pub fn add_light(mut self, light: Light) -> SceneBuilder {
        self.lights.push(light);
        self
    }

    pub fn finish(self) -> Scene {
        Scene {
            width: self.width,
            height: self.height,
            fov: self.fov,
            objects: self.objects,
            lights: self.lights
        }
    }
}
