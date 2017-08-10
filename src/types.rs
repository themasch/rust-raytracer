use cgmath::{Vector3, Point3};
use image::Rgba;
use std::ops::{Mul, Add};

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32
}

impl Color {
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color { red: r, green: g, blue: b }
    }

    pub fn from_rgba(source: Rgba<u8>) -> Color {
        Color {
            red: source.data[0] as f32 / 255.0,
            green: source.data[1] as f32 / 255.0,
            blue: source.data[2] as f32 / 255.0,
        }
    }

    pub fn to_rgba8(&self) -> Rgba<u8> {
        Rgba {
            data: [
                (self.red * 255.0).round() as u8,
                (self.green * 255.0).round() as u8,
                (self.blue * 255.0).round() as u8,
                0
            ]
        }
    }

    pub fn clamp(&self) -> Color {
        Color {
            red: self.red.min(1.0).max(0.0),
            blue: self.blue.min(1.0).max(0.0),
            green: self.green.min(1.0).max(0.0),
        }
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color {
            red: self.red * other.red,
            blue: self.blue * other.blue,
            green: self.green * other.green
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, other: f32) -> Color {
        Color {
            red: self.red * other,
            blue: self.blue * other,
            green: self.green * other
        }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            red: self.red + other.red,
            blue: self.blue + other.blue,
            green: self.green + other.green
        }
    }
}

pub type Point = Point3<f64>;

pub type Direction = Vector3<f64>;
