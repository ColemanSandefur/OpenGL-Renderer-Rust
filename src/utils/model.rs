use crate::shaders::pbr::PBRSimple;
use crate::shaders::pbr::PBRTextures;
use crate::utils::positioning::Rotation;
use crate::{renderer::RenderScene, shaders::pbr::PBR};
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use nalgebra::Matrix4;
use nalgebra::Rotation3;
use nalgebra::Vector3;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use std::error::Error;
use std::path::Path;

use crate::{shader::Shader, vertex::Vertex};

pub struct Model<S>
where
    S: Shader,
{
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    shader: S,
    euler: Rotation,
    position: Vector3<f32>,
}

impl<S> Model<S>
where
    S: Shader,
{
    pub fn publish<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.shader);
    }
}

pub trait ModelLoad {
    fn load_from_fs<P>(facade: &impl Facade, path: P) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
        P: AsRef<Path>;
}

impl ModelLoad for Model<PBR> {
    fn load_from_fs<P>(facade: &impl Facade, path: P) -> Result<Model<PBR>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        println!("path: {}", path.as_ref().to_str().unwrap());
        let scene = Scene::from_file(
            path.as_ref()
                .as_os_str()
                .to_str()
                .ok_or("error making path")?,
            vec![
                // Quick fix, should change later
                PostProcess::PreTransformVertices,
                PostProcess::GenerateNormals,
                PostProcess::Triangulate,
            ],
        )?;

        let mesh = &scene.meshes[0];
        let vertices = (0..mesh.vertices.len())
            .into_par_iter()
            .map(|index| {
                let vertex = mesh.vertices[index as usize];
                let position: [f32; 3] = [vertex.x, vertex.y, vertex.z];
                let normal_vec = mesh.normals[index as usize];
                let normal = [normal_vec.x, normal_vec.y, normal_vec.z];
                let tex_coords = match mesh.texture_coords[0].as_ref() {
                    Some(texture_coords) => {
                        let vec3 = texture_coords[index as usize];
                        [vec3.x, vec3.y]
                    }
                    None => [0.0; 2],
                };

                return Vertex {
                    position,
                    normal,
                    tex_coords,
                    ..Default::default()
                };
            })
            .collect::<Vec<_>>();

        let indices = mesh
            .faces
            .par_iter()
            .flat_map(|face| face.0.clone())
            .collect::<Vec<_>>();

        let index_buffer =
            IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)?;
        let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
        let mut pbr = PBR::load_from_fs(facade);
        let pbr_tex = PBRTextures::from_simple(
            facade,
            PBRSimple {
                albedo: [1.0, 0.0, 0.0],
                ..Default::default()
            },
        );
        pbr.set_pbr_params(pbr_tex);

        Ok(Self {
            index_buffer,
            vertex_buffer,
            shader: pbr,
            euler: Rotation::from_euler_angles(0.0, 0.0, 0.0),
            position: [0.0, 0.0, 0.0].into(),
        })
    }
}

impl Model<PBR> {
    pub fn debug_ui(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        let mut response = self.euler.debug_ui(ui).response;

        ui.horizontal(|ui| {
            let labels = ["x: ", "y: ", "z: "];

            let mut widget = |value: &mut f32, label: &str| {
                ui.add(egui::widgets::DragValue::new(value).prefix(label))
            };

            response |= {
                widget(&mut self.position[0], labels[0])
                    | widget(&mut self.position[1], labels[1])
                    | widget(&mut self.position[2], labels[2])
            };
        });

        if response.changed() {
            self.shader
                .set_model_mat(self.euler.get_matrix4().append_translation(&self.position));
        };

        egui::InnerResponse::new((), response)
    }
}
