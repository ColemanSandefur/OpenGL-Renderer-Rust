//! Basic Vertex implementation
//!
//! The [`Renderer`](crate::renderer::Renderer) should accept any Vertex implementation but this is
//! the default implementation

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0; 2],
        }
    }
}

implement_vertex!(Vertex, position, normal, tex_coords);
