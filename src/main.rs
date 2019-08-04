extern crate cgmath;
extern crate image;
extern crate num_cpus;
extern crate threadpool;
extern crate wavefront_obj;

mod light;
mod objects;
mod raycast;
mod render;
mod scene;
mod types;

use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use cgmath::prelude::*;

use cgmath::Deg;
use cgmath::Quaternion;
use light::*;
use objects::{Material, Mesh, ObjectBuilder, Plane, Sphere};
use render::render;
use scene::{Camera, SceneBuilder};
use types::{Color, Direction, Point};

fn format_time(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

use std::env;

fn main() {
    let idx: f64 = env::args()
        .collect::<Vec<String>>()
        .get(1)
        .unwrap_or(&String::from("0"))
        .parse()
        .unwrap_or(45.0);
    println!("rendering with {:?}° rot.", idx);
    let rotation = Deg(idx * 2.0);

    let teapot_read = wavefront_obj::obj::parse(String::from(include_str!("../teapot.obj")));

    if let Err(err) = teapot_read {
        panic!("{:?}", err);
    }

    let teapot = teapot_read.unwrap();
    // find first object
    let object = teapot
        .objects
        .iter()
        .find(|p| p.vertices.len() > 0)
        .expect("no object found");

    let scene = SceneBuilder::new()
        .add_object(
            ObjectBuilder::create_for(Plane::create(Direction::new(0.0, -1.0, 0.0)))
                .at_position(Point::new(0.0, -4.0, 0.0))
                .with_material(Material::diffuse_color(Color::from_rgb(0.2, 0.3, 0.4), 0.2))
                .into(),
        )
        .add_object(
            ObjectBuilder::create_for(Plane::create(Direction::new(0.0, 0.0, -1.0).normalize()))
                .at_position(Point::new(0.0, 0.0, -20.0))
                .with_material(Material::diffuse_color(Color::from_rgb(0.5, 1.0, 0.5), 0.2))
                .into(),
        )
        .add_object(
            ObjectBuilder::create_for(Mesh::create(object.clone()))
                .with_material(Material::reflective_color(
                    Color::from_rgb(0.6, 0.6, 0.6),
                    0.2,
                    0.02,
                ))
                .scale(1.0)
                .rotation(Quaternion::one() + Quaternion::from_angle_y(rotation))
                .at_position(Point::new(0.0, -2.0, -6.0))
                .into(),
        )
        .add_light(Light::Directional(DirectionalLight {
            direction: Direction::new(0.25, 0.0, -1.0).normalize(),
            color: Color::from_rgb(1.0, 1.0, 1.0),
            intensity: 20.0,
        }))
        .add_light(Light::Directional(DirectionalLight {
            direction: Direction::new(0.0, -1.0, -1.0),
            color: Color::from_rgb(1.0, 1.0, 1.0),
            intensity: 10.0,
        }))
        .finish();

    let camera = Camera {
        width: 1000,
        height: 1000,
        fov: 90.0,
    };

    let before_render = Instant::now();
    let image = render(scene, camera);
    let before_save = Instant::now();
    let ref mut fout = File::create(&Path::new("test.png")).unwrap();
    match image.save(fout, image::PNG) {
        Err(err) => println!("{:?}", err),
        Ok(_) => {}
    };

    println!(
        "render: {:?}, save: {:?}",
        format_time(&before_save.duration_since(before_render)),
        format_time(&before_save.elapsed())
    );
}
