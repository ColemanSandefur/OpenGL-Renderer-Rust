use crate::insert_program;
use crate::utils::shapes;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::Cubemap;
use glium::texture::DepthTexture2d;
use glium::DrawParameters;
use glium::IndexBuffer;
use glium::Program;
use glium::Surface;
use glium::VertexBuffer;
use nalgebra::Matrix4;
use std::rc::Rc;

pub struct Prefilter {
    program: Rc<Program>,
}

impl Prefilter {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = Rc::new(insert_program!("./vertex.glsl", "./fragment.glsl", facade));

        Self { program }
    }
    pub fn compute(&self, facade: &impl Facade, env_map: &Cubemap) -> Cubemap {
        let resolution = 128;
        let cubemap = Cubemap::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F16F16F16,
            glium::texture::MipmapsOption::EmptyMipmapsMax(4),
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

        let vb = VertexBuffer::new(facade, &shapes::get_cube()).unwrap();
        let ib = IndexBuffer::new(
            facade,
            glium::index::PrimitiveType::TrianglesList,
            &(0..36u32).into_iter().collect::<Vec<_>>(),
        )
        .unwrap();

        let depth_buffer = DepthTexture2d::empty(facade, resolution, resolution).unwrap();
        let perspective = Matrix4::new_perspective(1.0, 90.0f32.to_radians(), 0.1, 10.0);

        for (layer_id, camera_dir) in layers.into_iter().zip(Self::camera_directions()) {
            let uniforms = uniform! {
                view: Into::<[[f32; 4]; 4]>::into(camera_dir),
                projection: Into::<[[f32; 4]; 4]>::into(perspective),
                environment_map: env_map
            };

            for mipmap_level in 0..cubemap.get_mipmap_levels() {
                let cubemap_image = cubemap.mipmap(mipmap_level).unwrap().image(layer_id);
                let mut fb =
                    SimpleFrameBuffer::with_depth_buffer(facade, cubemap_image, &depth_buffer)
                        .unwrap();
                fb.clear_depth(1.0);

                let roughness = mipmap_level as f32 / (cubemap.get_mipmap_levels() - 1) as f32;

                let uniforms = uniforms.add("roughness", roughness);

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
            }
        }

        cubemap
    }
    fn camera_directions() -> [Matrix4<f32>; 6] {
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
