use egui::Ui;
use glium::backend::Context;
use glium::texture::Cubemap;
use glium::Blend;
use glium::DrawParameters;
use glium::Texture2d;
use glium::{backend::Facade, Program};
use nalgebra::Matrix4;
use std::any::Any;
use std::rc::Rc;

use crate::utils::pbr_skybox::PBRSkybox;
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
    facade: Rc<Context>,
}

impl PBRTextures {
    pub fn from_simple(facade: &impl Facade, simple: PBRSimple) -> Self {
        let create_texture =
            |data: [f32; 3]| Rc::new(TextureLoader::from_memory_f32(facade, &data, 1, 1).unwrap());

        Self {
            albedo: create_texture(simple.albedo),
            metallic: create_texture([simple.metallic; 3]),
            roughness: create_texture([simple.roughness; 3]),
            ao: create_texture([simple.ao; 3]),
            normal: create_texture([0.5, 0.5, 1.0]),
            facade: facade.get_context().clone(),
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

    pub fn debug_ui(&mut self, ui: &mut Ui) {
        //Albedo
        ui.label("Albedo");
        let albedo: Vec<Vec<_>> = self.albedo.read();

        let mut pixel = [
            albedo[0][0].0 as f32 / 255.0,
            albedo[0][0].1 as f32 / 255.0,
            albedo[0][0].2 as f32 / 255.0,
        ];

        if egui::widgets::color_picker::color_edit_button_rgb(ui, &mut pixel).changed() {
            self.set_albedo(
                TextureLoader::from_memory_f32(&self.facade, &pixel, 1, 1)
                    .unwrap()
                    .into(),
            );
        }

        // Metallic
        let metallic: Vec<Vec<_>> = self.metallic.read();
        let mut metallic = metallic[0][0].0;
        if self.debug_slider(ui, "metallic", &mut metallic) {
            self.set_metallic(
                TextureLoader::from_memory_f32(&self.facade, &[metallic as f32 / 255.0; 3], 1, 1)
                    .unwrap()
                    .into(),
            );
        }

        // Roughness
        let roughness: Vec<Vec<_>> = self.roughness.read();
        let mut roughness = roughness[0][0].0;
        if self.debug_slider(ui, "roughness", &mut roughness) {
            self.set_roughness(
                TextureLoader::from_memory_f32(&self.facade, &[roughness as f32 / 255.0; 3], 1, 1)
                    .unwrap()
                    .into(),
            );
        }

        // Ambient Occlusion
        let ao: Vec<Vec<_>> = self.ao.read();
        let mut ao = ao[0][0].0;
        if self.debug_slider(ui, "ao", &mut ao) {
            self.set_ao(
                TextureLoader::from_memory_f32(&self.facade, &[ao as f32 / 255.0; 3], 1, 1)
                    .unwrap()
                    .into(),
            );
        }
    }

    fn debug_slider(&self, ui: &mut Ui, label: &str, value: &mut u8) -> bool {
        ui.label(label);

        let changed = ui.add(egui::widgets::Slider::new(value, 0..=255)).changed();

        ui.separator();

        changed
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

    pub fn get_pbr_params_mut(&mut self) -> &mut PBRTextures {
        &mut self.pbr_params
    }

    pub fn debug_ui(&mut self, ui: &mut Ui) {
        self.pbr_params.debug_ui(ui);
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

        let pbr_skybox = scene_data.get_scene_object::<PBRSkybox>().unwrap();

        let irradiance_map = pbr_skybox.get_irradiance().as_ref();

        let prefilter_map = pbr_skybox.get_prefilter().as_ref();

        let brdf_lut = pbr_skybox.get_brdf().as_ref();

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
            lightColors: [1500.0f32;3],
            camPos: Into::<[f32; 3]>::into(scene_data.camera.position),
            irradiance_map: irradiance_map,
            prefilter_map: prefilter_map,
            brdfLUT: brdf_lut,
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

    fn get_model_mat(&self) -> Matrix4<f32> {
        self.model
    }

    fn set_model_mat(&mut self, model: Matrix4<f32>) {
        self.model = model;
    }

    fn equal_shader(&self, _shader: &dyn std::any::Any) -> bool {
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
