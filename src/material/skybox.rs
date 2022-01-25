use crate::cubemap_loader::{CubemapLoader, CubemapType};
use crate::renderer::{Renderable, SceneData};
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use super::Material;

#[derive(Clone)]
pub struct SkyboxMat {
    program: Arc<Program>,
    skybox: Arc<CubemapType>,
}

impl SkyboxMat {
    pub fn load_from_fs(
        facade: &impl Facade,
        directory: impl Into<PathBuf>,
        extension: &str,
    ) -> Self {
        let mut vertex_shader_file = File::open("shaders/skybox/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file = File::open("shaders/skybox/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        println!("Loading cubemap");
        //let cubemap = CubemapLoader::load_from_fs_hdr("hdr_cubemap/".into(), "hdr", facade);
        let cubemap = CubemapLoader::load_from_fs_hdr(directory.into(), extension, facade);
        println!("Finished loading cubemap");

        Self {
            program: Arc::new(program),
            skybox: Arc::new(cubemap),
        }
    }
    pub fn load_from_memory(
        facade: &impl Facade,
        images: Vec<Vec<f32>>,
        width: u32,
        height: u32,
    ) -> Self {
        let mut vertex_shader_file = File::open("shaders/skybox/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file = File::open("shaders/skybox/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        println!("Loading cubemap");
        let cubemap = CubemapLoader::load_from_memory_hdr(images, width, height, facade);
        println!("Finished loading cubemap");

        Self {
            program: Arc::new(program),
            skybox: Arc::new(cubemap),
        }
    }

    pub fn load_from_cubemap(facade: &impl Facade, cubemap: CubemapType) -> Self {
        let mut vertex_shader_file = File::open("shaders/skybox/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file = File::open("shaders/skybox/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        Self {
            program: Arc::new(program),
            skybox: Arc::new(cubemap),
        }
    }

    pub fn get_cubemap(&self) -> &Arc<CubemapType> {
        &self.skybox
    }
}
impl Material for SkyboxMat {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        _scene_data: &SceneData,
    ) {
        let camera_pos: [f32; 3] = [position[3][0], position[3][1], position[3][2]];

        let cubemap = &self.skybox;

        let uniforms = uniform! {
            projection: camera,
            view: position,
            camera_pos: camera_pos,
            skybox: &**cubemap,
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
                        test: glium::DepthTest::IfLessOrEqual,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn equal(&self, material: &dyn Any) -> bool {
        let _simple = match material.downcast_ref::<Self>() {
            Some(simple) => simple,
            None => return false,
        };

        true
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
