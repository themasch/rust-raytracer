use types::{Direction, Color};

#[derive(Debug, Copy, Clone)]
pub enum Light {
  Directional(DirectionalLight)
}

impl Light {
  pub fn direction(&self) -> Direction {
    match *self {
      Light::Directional(ref s) => s.direction
    }
  }

  pub fn intensity(&self) -> f32 {
    match *self {
      Light::Directional(ref s) => s.intensity
    }
  }

  pub fn color(&self) -> &Color {
    match *self {
      Light::Directional(ref s) => &s.color
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct DirectionalLight {
    pub direction: Direction,
    pub color: Color,
    pub intensity: f32
}

