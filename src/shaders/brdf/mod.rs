use crate::insert_program;
use crate::utils::shapes;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::index::NoIndices;
use glium::texture::DepthTexture2d;
use glium::DrawParameters;
use glium::Program;
use glium::Surface;
use glium::Texture2d;
use glium::VertexBuffer;
use std::rc::Rc;

pub struct BRDF {
    program: Rc<Program>,
}

impl BRDF {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = Rc::new(insert_program!("./vertex.glsl", "./fragment.glsl", facade));

        Self { program }
    }
    pub fn compute(&self, facade: &impl Facade) -> Texture2d {
        let brdf = Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F16F16,
            glium::texture::MipmapsOption::NoMipmap,
            512,
            512,
        )
        .unwrap();

        let vb = VertexBuffer::new(facade, &shapes::get_quad()).unwrap();
        let ib = NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let mut fb = SimpleFrameBuffer::new(facade, &brdf).unwrap();

        let uniforms = uniform! {};

        fb.draw(
            &vb,
            &ib,
            &self.program,
            &uniforms,
            &DrawParameters {
                ..Default::default()
            },
        )
        .unwrap();

        brdf
    }
}
