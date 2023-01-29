use crate::shaders::pbr::PBRSimple;
use crate::shaders::pbr::PBRTextures;
use crate::utils::positioning::Rotation;
use crate::utils::texture_loader::TextureLoader;
use crate::{renderer::RenderScene, shaders::pbr::PBR};
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use nalgebra::Matrix4;
use nalgebra::Vector3;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use russimp::texture::TextureType;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use crate::{shader::Shader, vertex::Vertex};

pub struct SubModel<S>
where
    S: Shader,
{
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    shader: S,
    euler: Rotation,
    position: Vector3<f32>,
    parent_mat: Matrix4<f32>,
}

impl<S> SubModel<S>
where
    S: Shader,
{
    pub fn publish<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.shader);
    }
    pub fn set_vertex_buffer(&mut self, vb: VertexBuffer<Vertex>) {
        self.vertex_buffer = vb;
    }
    pub fn set_index_buffer(&mut self, ib: IndexBuffer<u32>) {
        self.index_buffer = ib;
    }
    pub fn get_rotation(&self) -> &Rotation {
        &self.euler
    }
    pub fn set_rotation(&mut self, rotation: Rotation) {
        self.euler = rotation;
        self.update_matrix();
    }

    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.update_matrix();
    }

    pub fn get_parent_mat(&self) -> &Matrix4<f32> {
        &self.parent_mat
    }
    fn set_parent_mat(&mut self, mat: Matrix4<f32>) {
        self.parent_mat = mat;
        self.update_matrix();
    }

    pub fn update_matrix(&mut self) {
        self.shader.set_model_mat(
            self.parent_mat * self.euler.get_matrix4().append_translation(&self.position),
        );
    }
}

pub struct Model<S>
where
    S: Shader,
{
    sub_models: Vec<SubModel<S>>,
    shader: S,
    euler: Rotation,
    position: Vector3<f32>,
}

impl<S> Model<S>
where
    S: Shader,
{
    pub fn new(vb: VertexBuffer<Vertex>, ib: IndexBuffer<u32>, shader: S) -> Self {
        let sub_model = SubModel {
            shader: shader.clone_sized(),
            index_buffer: ib,
            vertex_buffer: vb,
            euler: Rotation::from_euler_angles(0.0, 0.0, 0.0),
            position: [0.0; 3].into(),
            parent_mat: Matrix4::new_translation(&[0.0; 3].into()),
        };

        Self {
            sub_models: vec![sub_model],
            shader,
            euler: Rotation::from_euler_angles(0.0, 0.0, 0.0),
            position: [0.0; 3].into(),
        }
    }

    pub fn publish<'a>(&'a self, scene: &mut RenderScene<'a>) {
        for sub_model in &self.sub_models {
            sub_model.publish(scene);
        }
    }

    pub fn get_shader(&self) -> &S {
        &self.shader
    }

    pub fn get_shader_mut(&mut self) -> &mut S {
        &mut self.shader
    }

    pub fn get_rotation(&self) -> &Rotation {
        &self.euler
    }

    pub fn set_rotation(&mut self, rotation: Rotation) {
        self.euler = rotation;
        self.update_matrix();
    }

    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.update_matrix();
    }

    pub fn get_sub_models(&self) -> &Vec<SubModel<S>> {
        &self.sub_models
    }
    pub fn get_sub_models_mut(&mut self) -> &mut Vec<SubModel<S>> {
        &mut self.sub_models
    }

    pub fn update_matrix(&mut self) {
        let mat = self.euler.get_matrix4().append_translation(&self.position);
        self.shader.set_model_mat(mat);

        for sub in &mut self.sub_models {
            sub.set_parent_mat(mat);
        }
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
        let path = path.as_ref();
        let scene = Scene::from_file(
            path.to_str().ok_or(format!(
                "Error loading file {:?}, invalid name",
                path.file_name()
            ))?,
            vec![
                // Quick fix, should change later
                PostProcess::PreTransformVertices,
                PostProcess::GenerateNormals,
                PostProcess::Triangulate,
            ],
        )?;

        let mut sub_models = Vec::new();
        for mesh in scene.meshes {
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
            println!("{:#?}", scene.materials);
            let scene_material = &scene.materials[mesh.material_index as usize];
            let load_file = |name: TextureType| {
                if let Some(texture) = scene_material.textures.get(&name) {
                    println!("reading file {:?}", name);
                    let mut path = PathBuf::from(path);
                    path.set_file_name(&texture[0].path);

                    return TextureLoader::from_fs(facade, &path).ok();
                }

                None
            };

            if let Some(texture) = load_file(TextureType::Diffuse) {
                pbr.get_pbr_params_mut().set_albedo(texture.into());
            }
            if let Some(texture) = load_file(TextureType::Height) {
                pbr.get_pbr_params_mut().set_normal(texture.into());
            }
            if let Some(texture) = load_file(TextureType::AmbientOcclusion) {
                pbr.get_pbr_params_mut().set_ao(texture.into());
            }
            if let Some(texture) = load_file(TextureType::Metalness) {
                pbr.get_pbr_params_mut().set_metallic(texture.into());
            }
            if let Some(texture) = load_file(TextureType::Roughness) {
                pbr.get_pbr_params_mut().set_roughness(texture.into());
            }

            let sub_model = SubModel {
                shader: pbr,
                vertex_buffer,
                index_buffer,
                euler: Rotation::from_euler_angles(0.0, 0.0, 0.0),
                position: [0.0, 0.0, 0.0].into(),
                parent_mat: Matrix4::new_translation(&[0.0; 3].into()),
            };

            sub_models.push(sub_model);
        }

        Ok(Self {
            sub_models,
            shader: PBR::load_from_fs(facade),
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
            self.update_matrix()
        };

        ui.collapsing("Sub Models", |ui| {
            for (i, sub) in self.sub_models.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.collapsing(format!("Sub Model {}", i), |ui| {
                        let mut response = sub.euler.debug_ui(ui).response;

                        ui.horizontal(|ui| {
                            let labels = ["x: ", "y: ", "z: "];

                            let mut widget = |value: &mut f32, label: &str| {
                                ui.add(egui::widgets::DragValue::new(value).prefix(label))
                            };

                            response |= {
                                widget(&mut sub.position[0], labels[0])
                                    | widget(&mut sub.position[1], labels[1])
                                    | widget(&mut sub.position[2], labels[2])
                            };
                        });

                        if response.changed() {
                            sub.update_matrix()
                        };
                        egui::CollapsingHeader::new("PBR Values").show(ui, |ui| {
                            sub.shader.debug_ui(ui);
                        });
                    });
                });
            }
        });

        egui::InnerResponse::new((), response)
    }
}
