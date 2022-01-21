use cgmath::{Matrix4, SquareMatrix};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub model: [[f32; 4]; 4],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 1.0],
            model: Matrix4::from_diagonal([1.0, 1.0, 1.0, 1.0].into()).into(),
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0; 2],
        }
    }
}

implement_vertex!(Vertex, position, model, normal);
