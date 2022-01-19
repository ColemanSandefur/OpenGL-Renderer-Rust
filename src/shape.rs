use std::any::Any;

use crate::material::Material;
use crate::renderer::RenderScene;
use crate::vertex::Vertex;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::IndexBuffer;
use glium::VertexBuffer;

pub struct Shape {
    position: Vector3<f32>,
    rotation: Vector3<Rad<f32>>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    material: Box<dyn Material>,
}

impl Shape {
    pub fn with_vertices(
        facade: &impl Facade,
        material: impl Material,
        vertex: &[Vertex],
        index: &[u32],
    ) -> Self {
        Shape {
            position: [0.0, 0.0, 0.0].into(),
            rotation: [Rad(0.0), Rad(0.0), Rad(0.0)].into(),
            vertex_buffer: VertexBuffer::new(facade, vertex).unwrap(),
            index_buffer: IndexBuffer::new(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                index,
            )
            .unwrap(),
            material: Box::new(material),
        }
    }

    pub fn build_matrix(&mut self) {
        let rotation_mat = Matrix4::from_angle_x(self.rotation.x)
            * Matrix4::from_angle_y(self.rotation.y)
            * Matrix4::from_angle_z(self.rotation.z);
        let translation = Matrix4::from_translation(self.position);

        let model = translation * rotation_mat;

        for vert in &mut *self.vertex_buffer.map() {
            vert.model = model.into();
        }
    }

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(
            &self.vertex_buffer,
            &self.index_buffer,
            &*self.material,
        );
    }

    pub fn relative_move(&mut self, position: impl Into<Vector3<f32>>) {
        self.position = self.position + position.into();
        self.build_matrix();
    }

    pub fn relative_rotate(&mut self, rotation: impl Into<Vector3<Rad<f32>>>) {
        let rotation = rotation.into();
        self.rotation[0] += rotation[0];
        self.rotation[1] += rotation[1];
        self.rotation[2] += rotation[2];
        self.build_matrix();
    }

    pub fn square(width: f32, facade: &impl Facade, material: impl Material) -> Self {
        Shape::with_vertices(
            facade,
            material,
            &[
                Vertex {
                    position: [-width / 2.0, width / 2.0, 0.0],
                    ..Default::default()
                },
                Vertex {
                    position: [width / 2.0, width / 2.0, 0.0],
                    ..Default::default()
                },
                Vertex {
                    position: [-width / 2.0, -width / 2.0, 0.0],
                    ..Default::default()
                },
                Vertex {
                    position: [width / 2.0, -width / 2.0, 0.0],
                    ..Default::default()
                },
            ],
            &[0u32, 1, 2, 1, 3, 2],
        )
    }
}
