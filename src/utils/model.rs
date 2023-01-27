use crate::shaders::pbr::PBRSimple;
use crate::shaders::pbr::PBRTextures;
use crate::utils::texture_loader::TextureLoader;
use crate::{renderer::RenderScene, shaders::pbr::PBR};
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use nalgebra::Matrix4;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;

use crate::{shader::Shader, vertex::Vertex};

pub struct Model<S>
where
    S: Shader,
{
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    shader: S,
    euler: [f32; 3],
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
            euler: [0.0, 0.0, 0.0],
        })
    }
}

impl Model<PBR> {
    pub fn debug_ui(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            let mut angles = [
                self.euler[0].to_degrees(),
                self.euler[1].to_degrees(),
                self.euler[2].to_degrees(),
            ];

            let mut changed = false;
            changed = ui
                .add(egui::widgets::DragValue::new(&mut angles[0]).prefix("roll: "))
                .changed()
                || changed;
            changed = ui
                .add(egui::widgets::DragValue::new(&mut angles[1]).prefix("pitch: "))
                .changed()
                || changed;
            changed = ui
                .add(egui::widgets::DragValue::new(&mut angles[2]).prefix("yaw: "))
                .changed()
                || changed;

            self.euler[0] = (angles[0] % 360.0).to_radians();
            self.euler[1] = (angles[1] % 360.0).to_radians();
            self.euler[2] = (angles[2] % 360.0).to_radians();

            if changed {
                self.shader.set_model_mat(Matrix4::from_euler_angles(
                    self.euler[0],
                    self.euler[1],
                    self.euler[2],
                ));
            }
        })
    }
}
