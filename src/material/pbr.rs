use cgmath::{Vector3, Matrix4};
use glium::Blend;
use glium::backend::Facade;
use glium::backend::Context;
use glium::index::IndicesSource;
use glium::texture::Texture2d;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use crate::gui::DebugGUI;
use crate::cubemap_loader::CubemapType;
use crate::gui::DebugGUIFormat;
use crate::renderer::{Renderable, SceneData};
use crate::texture::TextureLoader;

use super::Material;

/// Basic definition of physically based rendering parameters.
///
/// Now used for easy creation of [`PBRTextures`] which will create a texture for each value
#[derive(Clone, Debug)]
pub struct PBRParams {
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
}

impl PBRParams {
    /// Simple red material
    pub fn sample() -> Self {
        Self {
            albedo: [1.0, 0.0, 0.0].into(),
            metallic: 0.0,
            roughness: 0.25,
            ao: 1.0,
        }
    }

    /// Simple metal material
    pub fn metal() -> Self {
        Self {
            albedo: [0.7, 0.7, 0.7].into(),
            metallic: 1.0,
            roughness: 0.05,
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

    /// Set ambient occlusion
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

// Holds texture maps for physically based rendering.
// Has now replaced PBRParams in the PBR shader, you can easily convert PBRParams into PBRTextures
// using from_params function
#[derive(Clone)]
pub struct PBRTextures {
    pub albedo: Arc<Texture2d>,
    pub metallic: Arc<Texture2d>,
    pub roughness: Arc<Texture2d>,
    pub ao: Arc<Texture2d>,
    pub facade: Rc<Context>,
}

impl PBRTextures {
    pub fn from_params(params: PBRParams, facade: &impl Facade) -> Self {
        let create_texture = TextureLoader::from_memory_rgbf32;
        let albedo_slice: &[f32; 3] = params.albedo.as_ref();
        Self {
            albedo: Arc::new(create_texture(facade, albedo_slice, 1, 1).unwrap()),
            metallic: Arc::new(create_texture(facade, &[params.metallic; 3], 1, 1).unwrap()),
            roughness: Arc::new(create_texture(facade, &[params.roughness; 3], 1, 1).unwrap()),
            ao: Arc::new(create_texture(facade, &[params.ao; 3], 1, 1).unwrap()),
            facade: facade.get_context().clone(),
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

impl DebugGUI for PBRTextures {
    fn debug(&mut self, ui: &mut egui::Ui) {
        // Create a slider for the value, and create a new texture if the value was changed
        let print_texture = |texture: &Texture2d, name: &str, ui: &mut egui::Ui, facade: &Rc<Context>| -> Option<Texture2d>{
            // Only display if texture is a 1x1 texture, else the texture shouldn't be able to be
            // modified by sliders
            if texture.get_width() == 1 && texture.get_height().unwrap_or(1) == 1 {
                let rgb: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
                let mut pixel = rgb[0][0];
                ui.label(name);
                if ui.add(egui::Slider::new(&mut pixel.0, 0..=255)).changed() {
                    return TextureLoader::from_memory_rgb8(facade, &[pixel.0; 3], 1, 1).ok();
                }
            }
            
            None
        };

        // Only display if texture is a 1x1 texture, else the texture shouldn't be able to be
        // modified by sliders
        if self.albedo.get_width() == 1 && self.albedo.get_height().unwrap_or(1) == 1 {
            let rgb: Vec<Vec<(u8, u8, u8, u8)>> = self.albedo.read();
            let pixel = rgb[0][0];
            let mut pixel = [pixel.0, pixel.1, pixel.2];
            ui.label("Albedo");

            if DebugGUIFormat::rgb_byte(ui, &mut pixel).changed() {
                if let Ok(texture) = TextureLoader::from_memory_rgb8(&self.facade, &pixel, 1, 1) {
                    self.set_albedo_map(texture);
                }
            }
        }
        
        if let Some(texture) = print_texture(&self.metallic, "Metallic", ui, &self.facade) {
            self.set_metallic_map(texture);
        }
        if let Some(texture) = print_texture(&self.roughness, "Roughness", ui, &self.facade) {
            self.set_roughness_map(texture);
        }
        if let Some(texture) = print_texture(&self.ao, "Ao", ui, &self.facade) {
            self.set_ao_map(texture);
        }
    }
}

/// Physically Based Rendering shader.
///
/// This struct is responsible for handling physically based rendering. It holds the necessary
/// parameters for PBR to function like how metallic the object is, and model transformations.
/// Every object will have their own `PBR` to handle the object's properties. Cloning should be
/// fairly cheap, so even if you have multiple objects that have differing materials, **it is
/// recommended to clone this struct instead of creating a new one** since it will have to reload the
/// program from the file system. To render you use the [`Material`] trait.
#[derive(Clone)]
pub struct PBR {
    light_pos: Vector3<f32>,
    light_color: Vector3<f32>,
    program: Arc<Program>,
    pbr_params: PBRTextures,
    context: Rc<Context>,
    model: Matrix4<f32>,
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
            model: Matrix4::from_translation([0.0; 3].into()),
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

    pub fn set_model_matrix(&mut self, model: Matrix4<f32>) {
        self.model = model;
    }

    pub fn get_model_matrix(&self) -> &Matrix4<f32> {
        &self.model
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
        let model_matrix: [[f32; 4]; 4] = self.model.into();

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
                    model: model_matrix,
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
            _ => return,
        };
    }

    fn equal(&self, _material: &dyn Any) -> bool {
        //let _simple = match material.downcast_ref::<Self>() {
            //Some(simple) => simple,
            //None => return false,
        //};

        // There is currently no implementation for checking if PbrTextures are equal so to
        // avoid potential bugs, it will always return false
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

impl DebugGUI for PBR {
    fn debug(&mut self, ui: &mut egui::Ui) {
        self.pbr_params.debug(ui);
    }
}
