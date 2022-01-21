use crate::renderer::RenderScene;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use std::path::PathBuf;
use tobj::LoadOptions;

use crate::{material::Material, vertex::Vertex};

pub struct ModelSegment<T: Material> {
    material: T,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}
impl<T: Material> ModelSegment<T> {
    pub fn new(
        vertex_buffer: VertexBuffer<Vertex>,
        index_buffer: IndexBuffer<u32>,
        material: T,
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            material,
        }
    }
    pub fn build_matrix(&mut self, model: Matrix4<f32>) {
        for vert in &mut *self.vertex_buffer.map() {
            vert.model = model.into();
        }
    }

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
    }

    pub fn get_material(&self) -> &T {
        &self.material
    }
    pub fn get_material_mut(&mut self) -> &mut T {
        &mut self.material
    }
}
pub struct Model<T: Material> {
    material: T,
    position: Vector3<f32>,
    rotation: Vector3<Rad<f32>>,
    segments: Vec<ModelSegment<T>>,
}

impl<T: Material> Model<T> {
    pub fn load_from_fs(path: PathBuf, facade: &impl Facade, material: T) -> Self {
        let (models, materials) = tobj::load_obj(
            path.as_os_str().to_str().unwrap(),
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();

        if materials.is_ok() {
            println!("{} materials", materials.unwrap().len());
        } else {
            println!("error {}", materials.unwrap_err());
        }

        let mut segments = Vec::new();

        for model in models {
            println!("{}", model.name);
            let mut vertices: Vec<Vertex> = Vec::new();
            let indices: Vec<u32> = model.mesh.indices;

            let num_vertices = model.mesh.positions.len() / 3;

            // Load x, y, z, positions for all vertices
            for triplet in 0..num_vertices {
                let index = triplet * 3;
                let x = model.mesh.positions[index];
                let y = model.mesh.positions[index + 1];
                let z = model.mesh.positions[index + 2];

                vertices.push(Vertex {
                    position: [x, y, z],
                    ..Default::default()
                });
            }

            // Load the normals for all veritces
            for triplet in 0..num_vertices {
                let index = triplet * 3;
                let x = model.mesh.normals[index];
                let y = model.mesh.normals[index + 1];
                let z = model.mesh.normals[index + 2];

                match vertices.get_mut(triplet) {
                    Some(vertex) => {
                        vertex.normal = [x, y, z];
                    }
                    None => {
                        println!("vertex {} is missing", index);
                    }
                };
            }

            let index_buffer =
                IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
                    .unwrap();
            let vertex_buffer = VertexBuffer::new(facade, &vertices).unwrap();

            segments.push(ModelSegment::new(
                vertex_buffer,
                index_buffer,
                material.clone_sized(),
            ));
        }

        Self {
            material,
            position: [0.0; 3].into(),
            rotation: [Rad(0.0); 3].into(),
            segments,
        }
    }

    pub fn build_matrix(&mut self) {
        let rotation_mat = Matrix4::from_angle_x(self.rotation.x)
            * Matrix4::from_angle_y(self.rotation.y)
            * Matrix4::from_angle_z(self.rotation.z);
        let translation = Matrix4::from_translation(self.position);

        let model = translation * rotation_mat;

        for segment in &mut self.segments {
            segment.build_matrix(model.clone());
        }
    }

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        //scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
        for item in &self.segments {
            item.render(scene);
        }
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
    pub fn get_material(&self) -> &T {
        &self.material
    }
    pub fn get_material_mut(&mut self) -> &mut T {
        &mut self.material
    }

    pub fn get_segments(&self) -> &Vec<ModelSegment<T>> {
        &self.segments
    }
    pub fn get_segments_mut(&mut self) -> &mut Vec<ModelSegment<T>> {
        &mut self.segments
    }
}
