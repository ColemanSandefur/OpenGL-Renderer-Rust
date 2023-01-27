use glium::Blend;
use glium::DrawParameters;
use glium::Texture2d;
use glium::{backend::Facade, Program};
use nalgebra::Matrix4;
use std::any::Any;
use std::{rc::Rc, sync::Arc};

use crate::utils::texture_loader::TextureLoader;
use crate::{insert_program, shader::Shader};

#[derive(Clone)]
pub struct PBRSimple {
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
}

impl Default for PBRSimple {
    fn default() -> Self {
        Self {
            albedo: [1.0; 3],
            metallic: 0.0,
            roughness: 0.05,
            ao: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct PBRTextures {
    albedo: Rc<Texture2d>,
    metallic: Rc<Texture2d>,
    roughness: Rc<Texture2d>,
    ao: Rc<Texture2d>,
    normal: Rc<Texture2d>,
}

impl PBRTextures {
    pub fn from_simple(facade: &impl Facade, simple: PBRSimple) -> Self {
        let create_texture =
            |data: [f32; 3]| Rc::new(TextureLoader::from_memory_f32(facade, &data, 1, 1).unwrap());

        println!("from simple: {:?}", simple.albedo);

        Self {
            albedo: create_texture(simple.albedo),
            metallic: create_texture([simple.metallic; 3]),
            roughness: create_texture([simple.roughness; 3]),
            ao: create_texture([simple.ao; 3]),
            normal: create_texture([0.5, 0.5, 1.0]),
        }
    }

    pub fn set_albedo(&mut self, texture: Rc<Texture2d>) {
        self.albedo = texture;
    }
    pub fn set_metallic(&mut self, texture: Rc<Texture2d>) {
        self.metallic = texture;
    }
    pub fn set_roughness(&mut self, texture: Rc<Texture2d>) {
        self.roughness = texture;
    }
    pub fn set_ao(&mut self, texture: Rc<Texture2d>) {
        self.ao = texture;
    }
    pub fn set_normal(&mut self, texture: Rc<Texture2d>) {
        self.normal = texture;
    }
}

#[derive(Clone)]
pub struct PBR {
    program: Rc<Program>,
    pbr_params: PBRTextures,
    model: Matrix4<f32>,
}

impl PBR {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = Rc::new(insert_program!("./vertex.glsl", "./fragment.glsl", facade));

        Self {
            program,
            pbr_params: PBRTextures::from_simple(facade, Default::default()),
            model: Matrix4::new_translation(&[0.0; 3].into()),
        }
    }

    pub fn set_pbr_params(&mut self, params: PBRTextures) {
        self.pbr_params = params;
    }
}

impl Shader for PBR {
    fn render<'a>(
        &self,
        vertex_buffer: glium::vertex::VerticesSource<'a>,
        index_buffer: glium::index::IndicesSource<'a>,
        surface: &mut crate::renderer::Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        scene_data: &crate::renderer::SceneData,
    ) {
        let model_matrix: [[f32; 4]; 4] = self.model.into();
        let uniforms = uniform! {
            projection: camera,
            view: position,
            model: model_matrix,
            albedo_map: &*self.pbr_params.albedo,
            metallic_map: &*self.pbr_params.metallic,
            roughness_map: &*self.pbr_params.roughness,
            ao_map: &*self.pbr_params.ao,
            normal_map: &*self.pbr_params.normal,
            lightPositions: [10.0f32, 10.0, 3.0],
            lightColors: [1500.0f32;3]
        };

        surface
            .draw(
                vertex_buffer,
                index_buffer,
                &self.program,
                &uniforms,
                &DrawParameters {
                    depth: glium::Depth {
                        test: glium::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    blend: Blend {
                        color: glium::BlendingFunction::Addition {
                            source: glium::LinearBlendingFactor::SourceAlpha,
                            destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
                        },
                        alpha: glium::BlendingFunction::Addition {
                            source: glium::LinearBlendingFactor::One,
                            destination: glium::LinearBlendingFactor::Zero,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn get_model_mat(&self) -> &Matrix4<f32> {
        &self.model
    }

    fn set_model_mat(&mut self, model: Matrix4<f32>) {
        self.model = model;
    }

    fn equal_shader(&self, shader: &dyn std::any::Any) -> bool {
        false
    }

    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_shader(&self) -> Box<dyn Shader> {
        Box::new(self.clone())
    }
    fn clone_sized(&self) -> Self {
        self.clone()
    }
}
