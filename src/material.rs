use crate::renderer::Renderable;
use glium::backend::Facade;
use glium::{index::IndicesSource, vertex::VerticesSource, Program};
use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::path::{Path};

pub mod basic;
pub mod equirectangle;
pub mod pbr;
pub mod phong;
pub mod skybox;

pub use basic::*;
pub use equirectangle::*;
pub use pbr::*;
pub use phong::*;
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
    /// Renders the given index and vertex buffers to the given surface. This also gives you access
    /// to the struct that implements this trait. That is how you can render materials with
    /// unique variables
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

    /// Determines if the passed material is the same type and has the same content. Might be used
    /// for batching later on (group objects with identical materials).
    fn equal(&self, material: &dyn Any) -> bool;

    /// Determines if the passed material is the same type
    fn same_material(&self, material: &dyn Any) -> bool
    where
        Self: Sized,
    {
        material.is::<Self>()
    }

    /// Cloning where you don't have the concrete type.
    ///
    /// Usually [`clone_sized`] is more useful as it keeps the concrete type.
    ///
    /// [`clone_sized`]: Self::clone_sized
    fn clone_material(&self) -> Box<dyn Material>;

    /// Cloning where you know the type.
    ///
    /// Can be extremely useful with generics, as it will keep its type.
    ///
    /// # Example
    /// ```
    /// struct Mat<T: Material> {
    ///     material: T
    /// }
    ///
    /// impl<T: Material> Clone for Mat<T> {
    ///     fn clone(&self) -> Self {
    ///         Self {
    ///             // clone_material would not work since it is not a concrete type
    ///             material: self.material.clone_sized()
    ///         }
    ///     }
    /// }
    /// ```
    fn clone_sized(&self) -> Self
    where
        Self: Sized;
}

/// A simple macro which will include fragment and vertex shaders in the binary
///
/// The preferred way to load shaders since portability is guaranteed.
#[macro_export]
macro_rules! insert_program {
    ($vertex:expr, $fragment:expr, $facade:expr) => {
        crate::material::compile_program($facade, &include_str!($vertex), &include_str!($fragment))
    };
}

pub use insert_program;

/// Load program from file system
///
/// A simple helper function to load the vertex and fragment shaders and compile them as a program.
/// The `insert_program` macro should be used for increased portability, but it will have to recompile the program when you change a shader.
pub fn load_program<P>(facade: &impl Facade, path: P) -> Program
where
    P: AsRef<Path>,
{
    let mut path = path.as_ref().to_path_buf();
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

    compile_program(facade, &vertex_shader_src, &fragment_shader_src)
}

pub fn compile_program(facade: &impl Facade, vertex: &str, fragment: &str) -> Program {
    Program::from_source(facade, &vertex, &fragment, None)
        .expect(&format!("Error compiling shader"))
}

