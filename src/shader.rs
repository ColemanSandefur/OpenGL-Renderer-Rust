use crate::renderer::Renderable;
use crate::renderer::SceneData;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::Program;
use nalgebra::Matrix4;
use std::any::Any;

pub trait Shader: 'static {
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

    fn get_model_mat(&self) -> Matrix4<f32>;

    fn set_model_mat(&mut self, model: Matrix4<f32>);

    fn to_any(self) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Determines if the passed material is the same type and has the same content. Might be used
    /// for batching later on (group objects with identical materials).
    fn equal_shader(&self, shader: &dyn Any) -> bool;

    /// Determines if the passed material is the same type
    fn same_shader(&self, shader: &dyn Any) -> bool
    where
        Self: Sized,
    {
        shader.is::<Self>()
    }

    /// Cloning where you don't have the concrete type.
    ///
    /// Usually [`clone_sized`] is more useful as it keeps the concrete type.
    ///
    /// [`clone_sized`]: Self::clone_sized
    fn clone_shader(&self) -> Box<dyn Shader>;

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
        crate::shader::compile_program($facade, &include_str!($vertex), &include_str!($fragment))
    };
}

pub use insert_program;

pub fn compile_program(facade: &impl Facade, vertex: &str, fragment: &str) -> Program {
    Program::from_source(facade, &vertex, &fragment, None)
        .expect(&format!("Error compiling shader"))
}
