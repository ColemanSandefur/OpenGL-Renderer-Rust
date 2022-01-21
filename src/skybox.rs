use crate::renderer::RenderScene;
use glium::backend::Facade;
use glium::index::NoIndices;
use glium::VertexBuffer;

use crate::material::SkyboxMat;
use crate::vertex::Vertex;

pub struct Skybox {
    skybox: SkyboxMat,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
}

impl PartialEq for Skybox {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: [x, y, z],
        ..Default::default()
    }
}

impl Skybox {
    pub fn new(facade: &impl Facade, skybox: SkyboxMat) -> Self {
        let vertex_buffer = VertexBuffer::new(
            facade,
            &[
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, -1.0, 1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, 1.0),
            ],
        )
        .unwrap();
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        Self {
            index_buffer,
            vertex_buffer,
            skybox,
        }
    }
    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.skybox);
    }
}