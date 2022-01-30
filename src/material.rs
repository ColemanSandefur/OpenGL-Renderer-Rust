use crate::renderer::Renderable;
use glium::backend::Facade;
use glium::{index::IndicesSource, vertex::VerticesSource, Program};
use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
pub mod basic;
pub mod equirectangle;
pub mod pbr;
pub mod phong;
pub mod simple;
pub mod skybox;

pub use basic::*;
pub use equirectangle::*;
pub use pbr::*;
pub use phong::*;
pub use simple::*;
pub use skybox::*;

use crate::renderer::SceneData;

/// A shader material
///
/// This is how shaders are ran in the [`RenderScene`](crate::renderer::RenderScene). 
///
/// # Example
///
/// ```
/// use crate::renderer::Renderable;
/// use glium::backend::Facade;
/// use glium::index::IndicesSource;
/// use glium::vertex::VerticesSource;
/// use glium::{BackfaceCullingMode, DrawParameters, Program};
/// use std::any::Any;
/// use std::sync::Arc;
/// 
/// use crate::renderer::SceneData;
/// use crate::material::Material;
///
/// #[derive(Clone)]
/// pub struct BasicShader {
///     program: Arc<Program>,
///     color: [f32; 3],
/// }
///
/// impl Material for BasicShader {
///    fn render<'a>(
///        &self,
///        vertex_buffer: VerticesSource<'a>,
///        index_buffer: IndicesSource<'a>,
///        surface: &mut Renderable,
///        camera: [[f32; 4]; 4],
///        position: [[f32; 4]; 4],
///        _scene_data: &SceneData,
///    ) {
///        let uniforms = uniform! {
///            color: color
///        };
///
///        surface
///            .draw(
///                vertex_buffer,
///                index_buffer,
///                &*self.program,
///                &uniforms,
///                &DrawParameters {
///                    backface_culling: BackfaceCullingMode::CullCounterClockwise,
///                    depth: glium::Depth {
///                        test: glium::DepthTest::IfLess,
///                        write: true,
///                        ..Default::default()
///                    },
///                    ..Default::default()
///                },
///            )
///            .unwrap();
///    }
///
///    fn equal(&self, material: &dyn Any) -> bool {
///        let simple = match material.downcast_ref::<Self>() {
///            Some(simple) => simple,
///            None => return false,
///        };
///
///        true
///    }
///
///    fn to_any(self) -> Box<dyn Any> {
///        Box::new(self)
///    }
///    fn as_any(&self) -> &dyn Any {
///        self
///    }
///    fn as_any_mut(&mut self) -> &mut dyn Any {
///        self
///    }
///    fn clone_material(&self) -> Box<dyn Material> {
///        Box::new(self.clone())
///    }
///    fn clone_sized(&self) -> Self
///    where
///        Self: Sized,
///    {
///        self.clone()
///    }
/// }
/// ```
pub trait Material: 'static {
    /// Render the material
    ///
    /// Renders the given index and vertex buffers to the given surface
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        scene_data: &SceneData,
    );

    fn to_any(self) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn equal(&self, material: &dyn Any) -> bool;
    fn same_material(&self, material: &dyn Any) -> bool
    where
        Self: Sized,
    {
        material.is::<Self>()
    }

    fn clone_material(&self) -> Box<dyn Material>;
    fn clone_sized(&self) -> Self
    where
        Self: Sized;
}

/// Load program from file system
///
/// A simple helper function to load the vertex and fragment shaders and compile them as a program.
pub fn load_program(facade: &impl Facade, mut path: PathBuf) -> Program {
    if path.is_dir() {
        path.push("vertex.glsl");
    }
    let mut vertex_shader_file = File::open(path.with_file_name("vertex.glsl")).unwrap();
    let mut vertex_shader_src = String::new();
    vertex_shader_file
        .read_to_string(&mut vertex_shader_src)
        .unwrap();
    let mut fragment_shader_file = File::open(path.with_file_name("fragment.glsl")).unwrap();
    let mut fragment_shader_src = String::new();
    fragment_shader_file
        .read_to_string(&mut fragment_shader_src)
        .unwrap();

    Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).expect(&format!("Error compiling shader {}", path.as_os_str().to_str().unwrap()))
}
