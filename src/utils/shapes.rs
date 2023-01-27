use crate::vertex::Vertex;

pub fn get_cube() -> Vec<Vertex> {
    vec![
        // back face
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],

            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        // front face
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        // left face
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        // right face
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        // bottom face
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        // top face
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
    ]
}
