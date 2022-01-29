use cgmath::Vector3;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::sync::Arc;

use crate::cubemap_loader::CubemapType;
use crate::renderer::{Renderable, SceneData};

use super::Material;

#[derive(Clone, Debug)]
pub struct PBRParams {
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
}

impl PBRParams {
    pub fn sample() -> Self {
        Self {
            albedo: [1.0, 0.0, 0.0].into(),
            metallic: 0.0,
            roughness: 0.25,
            ao: 0.0,
        }
    }

    pub fn metal() -> Self {
        Self {
            albedo: [0.7, 0.7, 0.7].into(),
            metallic: 0.75,
            roughness: 0.24,
            ao: 1.0,
        }
    }
}

impl PartialEq for PBRParams {
    fn eq(&self, other: &Self) -> bool {
        self.albedo == other.albedo
            && self.metallic == other.metallic
            && self.roughness == other.roughness
            && self.ao == other.ao
    }
}

impl Default for PBRParams {
    fn default() -> Self {
        Self {
            albedo: [0.0; 3].into(),
            metallic: 0.0,
            roughness: 0.0,
            ao: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct PBR {
    light_pos: Vector3<f32>,
    light_color: Vector3<f32>,
    program: Arc<Program>,
    pbr_params: PBRParams,
}

impl PBR {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = crate::material::load_program(facade, "shaders/pbr/".into());

        Self {
            light_pos: [0.0; 3].into(),
            light_color: [300.0; 3].into(),
            program: Arc::new(program),
            pbr_params: PBRParams {
                ..Default::default()
            },
        }
    }

    pub fn set_pbr_params(&mut self, params: PBRParams) {
        self.pbr_params = params;
    }
    pub fn get_pbr_params(&self) -> &PBRParams {
        &self.pbr_params
    }
    pub fn get_pbr_params_mut(&mut self) -> &mut PBRParams {
        &mut self.pbr_params
    }

    pub fn set_light_pos(&mut self, pos: impl Into<Vector3<f32>>) {
        self.light_pos = pos.into();
    }
    pub fn set_light_color(&mut self, color: impl Into<Vector3<f32>>) {
        self.light_color = color.into();
    }
}

impl Material for PBR {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        scene_data: &SceneData,
    ) {
        let light_pos: [f32; 3] = self.light_pos.clone().into();
        let light_color: [f32; 3] = self.light_color.clone().into();
        let camera_pos: [f32; 3] = [position[3][0], position[3][1], position[3][2]];

        let albedo: [f32; 3] = self.pbr_params.albedo.into();
        let metallic = self.pbr_params.metallic;
        let roughness = self.pbr_params.roughness;
        let ao = self.pbr_params.ao;
        let skybox_obj = scene_data.get_skybox().unwrap();
        let skybox = skybox_obj.get_skybox().get_cubemap();

        match &skybox_obj.get_prefilter().as_ref().unwrap() {
            &CubemapType::Cubemap(prefilter) => {
                prefilter
                    .sampled()
                    .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear);
                let uniforms = uniform! {
                    light_pos: light_pos,
                    light_color: light_color,
                    projection: camera,
                    view: position,
                    camera_pos: camera_pos,
                    albedo: albedo,
                    metallic: metallic,
                    roughness: roughness,
                    ao: ao,
                    irradiance_map: skybox_obj.get_ibl().as_ref().unwrap(),
                    prefilter_map: prefilter,
                    brdf_lut: skybox_obj.get_brdf().as_ref().unwrap(),
                    skybox: &**skybox,
                };

                surface
                    .draw(
                        vertex_buffer,
                        index_buffer,
                        &*self.program,
                        &uniforms,
                        &DrawParameters {
                            backface_culling: BackfaceCullingMode::CullCounterClockwise,
                            depth: glium::Depth {
                                test: glium::DepthTest::IfLess,
                                write: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            _ => return,
        };
    }

    fn equal(&self, material: &dyn Any) -> bool {
        let simple = match material.downcast_ref::<Self>() {
            Some(simple) => simple,
            None => return false,
        };

        simple.light_pos == self.light_pos
            && simple.light_color == self.light_color
            && self.pbr_params == simple.pbr_params
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
    fn clone_material(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
    fn clone_sized(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }
}
