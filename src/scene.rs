use cgmath::InnerSpace;
use light::Light;
use objects::Object;
use raycast::{IntersectionResult, Ray};
use types::Direction;

pub struct Camera {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
}

impl Camera {
    pub fn to_sensor_direction(&self, x: f64, y: f64) -> Direction {
        let fov_adjustment = (self.fov.to_radians() / 2.0).tan();
        let aspect_ratio = self.width as f64 / self.height as f64;
        let sensor_x =
            (((x + 0.5) / self.width as f64) * 2.0 - 1.0) * aspect_ratio * fov_adjustment;
        let sensor_y = (1.0 - ((y + 0.5) / self.height as f64) * 2.0) * fov_adjustment;

        Direction {
            x: sensor_x,
            y: sensor_y,
            z: -1.0,
        }
        .normalize()
    }
}

pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn trace(&self, ray: &Ray) -> Option<IntersectionResult> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(ray))
            .filter(|intersection| intersection.distance() > 1e-13)
            .min()
    }
}

pub struct SceneBuilder {
    objects: Vec<Object>,
    lights: Vec<Light>,
}

impl SceneBuilder {
    pub fn new() -> SceneBuilder {
        SceneBuilder {
            objects: Vec::new(),
            lights: Vec::new(),
        }
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
            objects: self.objects,
            lights: self.lights,
        }
    }
}
