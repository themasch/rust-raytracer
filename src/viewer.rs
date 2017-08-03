#[derive(Debug)]
pub struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2]
}

implement_vertex!(Vertex, position, tex_coords);

