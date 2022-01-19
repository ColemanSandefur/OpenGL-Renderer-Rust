use crate::renderer::RenderScene;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use crate::{material::Material, vertex::Vertex};

pub struct Model<T: Material> {
    material: T,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    position: Vector3<f32>,
    rotation: Vector3<Rad<f32>>,
}

impl<T: Material> Model<T> {
    pub fn load_from_fs(path: PathBuf, facade: &impl Facade, material: T) -> Option<Self> {
        let file = File::open(path).unwrap();

        let buf = BufReader::new(file);

        let mut vertex_vec: Vec<Vertex> = Vec::new();
        let mut index_vec: Vec<u32> = Vec::new();

        for line in buf.lines() {
            if line.is_err() {
                return None;
            }

            let line = line.unwrap();

            let mut split = line.split_whitespace();

            if let Some(command) = split.next() {
                match command {
                    // Defines a vertex
                    "v" => {
                        let vertex = Vertex {
                            position: [
                                split.next().unwrap().parse().unwrap(),
                                split.next().unwrap().parse().unwrap(),
                                split.next().unwrap().parse().unwrap(),
                            ],
                            ..Default::default()
                        };

                        vertex_vec.push(vertex);
                    }
                    "vt" => {}
                    "vn" => {}
                    // Defines a face
                    "f" => {
                        for _ in 0..3 {
                            let segment = split.next().unwrap();

                            let mut seg_split = segment.split("/");

                            index_vec
                                .push(seg_split.next().unwrap().parse::<u32>().unwrap() - 1u32);
                            //index_vec
                            //.push(seg_split.next().unwrap().parse::<u32>().unwrap() - 1u32);
                            //index_vec
                            //.push(seg_split.next().unwrap().parse::<u32>().unwrap() - 1u32);
                        }
                    }
                    _ => (),
                }
            }
        }

        Some(Self {
            position: [0.0; 3].into(),
            rotation: [Rad(0.0); 3].into(),
            index_buffer: IndexBuffer::new(
                facade,
                glium::index::PrimitiveType::TrianglesList,
                &index_vec,
            )
            .unwrap(),
            vertex_buffer: VertexBuffer::new(facade, &vertex_vec).unwrap(),
            material,
        })
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
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
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
}
