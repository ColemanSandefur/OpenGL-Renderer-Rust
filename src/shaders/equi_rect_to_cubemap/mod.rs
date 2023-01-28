use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::{Cubemap, DepthTexture2d};
use glium::Texture2d;
use glium::{backend::Facade, DrawParameters};
use glium::{IndexBuffer, VertexBuffer};
use nalgebra::Matrix4;
use std::rc::Rc;

use glium::{Program, Surface};

use crate::insert_program;

pub struct EquiRectCubemap {
    program: Rc<Program>,
}

impl EquiRectCubemap {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = Rc::new(insert_program!("./vertex.glsl", "./fragment.glsl", facade));

        Self { program }
    }

    pub fn compute(&self, facade: &impl Facade, texture: &Texture2d, resolution: u32) -> Cubemap {
        let cubemap = Cubemap::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F16F16F16,
            glium::texture::MipmapsOption::NoMipmap,
            resolution,
        )
        .unwrap();

        let layers = [
            glium::texture::CubeLayer::PositiveX,
            glium::texture::CubeLayer::NegativeX,
            glium::texture::CubeLayer::PositiveY,
            glium::texture::CubeLayer::NegativeY,
            glium::texture::CubeLayer::PositiveZ,
            glium::texture::CubeLayer::NegativeZ,
        ];

        let camera_dir = Self::directions();

        let cube_vertices = VertexBuffer::new(facade, &crate::utils::shapes::get_cube()).unwrap();
        let cube_indices = IndexBuffer::new(
            facade,
            glium::index::PrimitiveType::TrianglesList,
            &(0..36u32).into_iter().collect::<Vec<_>>(),
        )
        .unwrap();

        let depth_buffer = DepthTexture2d::empty(facade, resolution, resolution).unwrap();

        for i in 0..6 {
            let image = cubemap.main_level().image(layers[i]);
            let mut surface =
                SimpleFrameBuffer::with_depth_buffer(facade, image, &depth_buffer).unwrap();
            surface.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

            let view: [[f32; 4]; 4] = camera_dir[i].into();
            let perspective: [[f32; 4]; 4] =
                Matrix4::new_perspective(1.0, 90.0f32.to_radians(), 0.01, 10.0).into();

            let uniforms = uniform! {
                view: view,
                projection: perspective,
                equirectangularMap: texture,
            };

            surface
                .draw(
                    &cube_vertices,
                    &cube_indices,
                    &self.program,
                    &uniforms,
                    &DrawParameters {
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
        cubemap
    }

    fn directions() -> [Matrix4<f32>; 6] {
        [
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[1.0, 0.0, 0.0].into(),
                &[0.0, -1.0, 0.0].into(),
            ),
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[-1.0, 0.0, 0.0].into(),
                &[0.0, -1.0, 0.0].into(),
            ),
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[0.0, 1.0, 0.0].into(),
                &[0.0, 0.0, 1.0].into(),
            ),
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[0.0, -1.0, 0.0].into(),
                &[0.0, 0.0, -1.0].into(),
            ),
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[0.0, 0.0, 1.0].into(),
                &[0.0, -1.0, 0.0].into(),
            ),
            Matrix4::look_at_rh(
                &[0.0, 0.0, 0.0].into(),
                &[0.0, 0.0, -1.0].into(),
                &[0.0, -1.0, 0.0].into(),
            ),
        ]
    }
}