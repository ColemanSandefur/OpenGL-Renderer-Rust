use glium::{index::IndicesSource, vertex::VerticesSource, Frame};
use std::any::Any;
pub mod basic;
pub mod pbr;
pub mod phong;
pub mod simple;

pub use basic::*;
pub use pbr::*;
pub use phong::*;
pub use simple::*;

pub trait Material: 'static {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Frame,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
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

