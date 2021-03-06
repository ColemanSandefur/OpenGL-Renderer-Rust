use crate::cubemap_loader::{CubemapLoader, CubemapType};
use crate::renderer::{Renderable, SceneData};
use cgmath::Matrix4;
use cgmath::Rad;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use super::Material;

/// Skybox shader.
///
/// This is responsible for rendering the skybox to the surface. The [`RenderScene`] renders this
/// last to not draw pixels that would otherwise be overwritten by objects in the scene. You just
/// need to provided a cubemap to be used as a skybox and it should just work. **It is recommended
/// that you clone the `SkyboxMat` instead of creating a new one** since the methods `load_from_fs`
/// and `load_from_cubemap` recompile the shader every time. After cloning you can just set the
/// skybox using `set_cubemap`.
///
/// [`RenderScene`]: crate::renderer::RenderScene
#[derive(Clone)]
pub struct SkyboxMat {
    program: Arc<Program>,
    skybox: Arc<CubemapType>,
    pub rotation: [[f32; 4]; 4],
}

impl SkyboxMat {
    pub fn load_from_fs(
        facade: &impl Facade,
        directory: impl Into<PathBuf>,
        extension: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let program = crate::material::insert_program!(
            "../shaders/skybox/vertex.glsl",
            "../shaders/skybox/fragment.glsl",
            facade
        );

        println!("Loading cubemap");
        //let cubemap = CubemapLoader::load_from_fs_hdr("hdr_cubemap/".into(), "hdr", facade);
        let cubemap = CubemapLoader::load_from_fs(directory.into(), extension, facade)?;
        println!("Finished loading cubemap");

        Ok(Self {
            program: Arc::new(program),
            skybox: Arc::new(cubemap),
            rotation: Matrix4::from_angle_y(Rad(0.0)).into(),
        })
    }

    // To be reimplemented
    //pub fn load_from_memory(
    //facade: &impl Facade,
    //images: Vec<Vec<f32>>,
    //width: u32,
    //height: u32,
    //) -> Self {
    //let program = crate::material::load_program(facade, "shaders/skybox/".into());

    //println!("Loading cubemap");
    //let cubemap = CubemapLoader::load_from_memory_hdr(images, width, height, facade);
    //println!("Finished loading cubemap");

    //Self {
    //program: Arc::new(program),
    //skybox: Arc::new(cubemap),
    //}
    //}

    pub fn load_from_cubemap(facade: &impl Facade, cubemap: CubemapType) -> Self {
        let program = crate::material::insert_program!(
            "../shaders/skybox/vertex.glsl",
            "../shaders/skybox/fragment.glsl",
            facade
        );

        Self {
            program: Arc::new(program),
            skybox: Arc::new(cubemap),
            rotation: Matrix4::from_angle_y(Rad(0.0)).into(),
        }
    }

    pub fn get_cubemap(&self) -> &Arc<CubemapType> {
        &self.skybox
    }

    pub fn set_cubemap(&mut self, skybox: Arc<CubemapType>) {
        self.skybox = skybox;
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
            model: self.rotation
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
