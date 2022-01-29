use cgmath::Vector3;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::texture::Texture2d;
use glium::vertex::VerticesSource;
use glium::Surface;
use glium::{backend::Context, BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use crate::cubemap_loader::CubemapType;
use crate::renderer::{Renderable, SceneData};

use super::Material;

// Creates a texture based off of a single color
// Helper function for PBRTextures when converting from PBRParams
pub fn create_texture(facade: &impl Facade, color: [f32; 3]) -> Texture2d {
    let texture = Texture2d::empty(facade, 1, 1).unwrap();
    texture
        .as_surface()
        .clear_color(color[0], color[1], color[2], 1.0);

    texture
}

// Basic definition of physically based rendering parameters
// Now used for easy creation of PBRTextures
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
            ao: 1.0,
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
    pub fn set_albedo(&mut self, data: impl Into<Vector3<f32>>) {
        self.albedo = data.into();
    }
    pub fn set_metallic(&mut self, metallic: f32) {
        self.metallic = metallic;
    }
    pub fn set_roughness(&mut self, roughness: f32) {
        self.roughness = roughness;
    }
    pub fn set_ao(&mut self, ao: f32) {
        self.ao = ao;
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
            albedo: [1.0; 3].into(),
            metallic: 1.0,
            roughness: 0.05,
            ao: 1.0,
        }
    }
}

// Holds texture maps for physically based rendering
// Has now replaced PBRParams in the PBR shader, you can easily convert PBRParams into PBRTextures
// using from_params function
#[derive(Clone)]
pub struct PBRTextures {
    pub albedo: Arc<Texture2d>,
    pub metallic: Arc<Texture2d>,
    pub roughness: Arc<Texture2d>,
    pub ao: Arc<Texture2d>,
}

impl PBRTextures {
    pub fn from_params(params: PBRParams, facade: &impl Facade) -> Self {
        Self {
            albedo: Arc::new(create_texture(facade, params.albedo.into())),
            metallic: Arc::new(create_texture(facade, [params.metallic; 3])),
            roughness: Arc::new(create_texture(facade, [params.roughness; 3])),
            ao: Arc::new(create_texture(facade, [params.ao; 3])),
        }
    }

    pub fn set_albedo_map(&mut self, map: Texture2d) {
        self.albedo = Arc::new(map);
    }

    pub fn set_metallic_map(&mut self, map: Texture2d) {
        self.metallic = Arc::new(map);
    }

    pub fn set_roughness_map(&mut self, map: Texture2d) {
        self.roughness = Arc::new(map);
    }

    pub fn set_ao_map(&mut self, map: Texture2d) {
        self.ao = Arc::new(map);
    }
}

#[derive(Clone)]
pub struct PBR {
    light_pos: Vector3<f32>,
    light_color: Vector3<f32>,
    program: Arc<Program>,
    pbr_params: PBRTextures,
    context: Rc<Context>,
}

impl PBR {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = crate::material::load_program(facade, "shaders/pbr/".into());
        let pbr_params = PBRParams::default();
        let params = PBRTextures::from_params(pbr_params.clone(), facade);

        Self {
            light_pos: [0.0; 3].into(),
            light_color: [300.0; 3].into(),
            program: Arc::new(program),
            pbr_params: params,
            context: facade.get_context().clone(),
        }
    }

    pub fn set_pbr_params(&mut self, pbr_textures: PBRTextures) {
        self.pbr_params = pbr_textures;
    }

    pub fn get_pbr_params(&self) -> &PBRTextures {
        &self.pbr_params
    }

    pub fn get_pbr_params_mut(&mut self) -> &mut PBRTextures {
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
                    albedo_map: &*self.pbr_params.albedo,
                    metallic_map: &*self.pbr_params.metallic,
                    roughness_map: &*self.pbr_params.roughness,
                    ao_map: &*self.pbr_params.ao,
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

        false
        //simple.light_pos == self.light_pos
        //&& simple.light_color == self.light_color
        //&& self.pbr_params == simple.pbr_params
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
