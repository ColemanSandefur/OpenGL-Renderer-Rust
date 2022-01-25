pub mod prefilter;
pub use prefilter::*;
use std::sync::Arc;

use glium::{backend::Facade, VertexBuffer};

use crate::vertex::Vertex;

#[derive(Clone)]
pub struct Specular {}

impl Specular {
    pub fn load(facade: &impl Facade) -> Self {
        Self {}
    }
}
