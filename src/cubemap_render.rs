use cgmath::Matrix4;
use cgmath::Rad;
use glium::{
    backend::Facade, framebuffer::SimpleFrameBuffer, texture::DepthTexture2d, texture::Texture2d,
    uniforms::Uniforms, vertex::VertexBuffer, DrawParameters, Program, Surface,
};
use image::{DynamicImage, ImageBuffer};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

use crate::{camera::Camera, vertex::Vertex};

// Renders all 6 sides of a cubemap to individual textures
// Currently just saves the textures to the file system
/// Converts a cubemap to 6 textures.
///
/// This will render the given cubemap into 6 individual textures. An example of it being useful is when generating the
/// prefilter map. It renders the skybox multiple times with each time getting blurrier. I used the
/// CubemapRender to render the skybox and save it to the file system.
pub struct CubemapRender {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: glium::index::NoIndices,
}

impl CubemapRender {
    // Directions and positions for the camera to face when rendering sides of the cube to a
    // texture buffer
    //const CAMERA_DIRECTIONS: [[[f32; 3]; 2]; 6] = [
    //Center         Up
    //[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],   // back -> right
    //[[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],  // front -> left
    //[[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],   // top
    //[[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]], // bottom
    //[[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],   // right -> front
    //[[0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],  // left -> back
    //];
    const CAMERA_DIRECTIONS: [[[f32; 3]; 2]; 6] = [
        [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],   // right
        [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],  // left
        [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],   // top
        [[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]], // bottom
        [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],  // front
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],   // back
    ];
    const FILE_NAMES: [&'static str; 6] = ["right", "left", "top", "bottom", "front", "back"];

    /// Create a CubemapRender.
    ///
    /// You should probably refrain from calling this very often since it creates a [`VertexBuffer`]
    /// which can be expensive. It is better to create one CubemapRender and pass it around.
    ///
    /// [`VertexBuffer`]: glium::VertexBuffer
    pub fn new(facade: &impl Facade) -> Self {
        let vertex_buffer = VertexBuffer::new(facade, &get_cube_vertices()).unwrap();

        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    /// Called to render to the file system
    ///
    /// output_dimensions is the dimensions of the desired texture. gen_uniforms is called once for
    /// every side of the cubemap.
    pub fn render<'a, 'b, U, P>(
        &self,
        output_dimensions: (u32, u32),
        output_directory: P,
        extension: &str,
        facade: &impl Facade,
        mut camera: Camera,
        gen_uniforms: impl Fn([[f32; 4]; 4], [[f32; 4]; 4]) -> U,
        program: &Program,
    ) -> Result<(), Box<dyn Error>>
    where
        U: Uniforms,
        P: AsRef<Path>,
    {
        create_dir_all(&output_directory).unwrap();

        let mut output_directory = output_directory.as_ref().to_path_buf();
        if output_directory.is_dir() {
            output_directory.push("output.random");
        }

        let buffer_texture = Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F16F16F16,
            glium::texture::MipmapsOption::NoMipmap,
            output_dimensions.0,
            output_dimensions.1,
        )
        .unwrap();
        let buffer_depth = DepthTexture2d::empty(facade, output_dimensions.0, output_dimensions.1)?;

        let mut frame_buffer =
            SimpleFrameBuffer::with_depth_buffer(facade, &buffer_texture, &buffer_depth)?;

        let camera_directions: Vec<Matrix4<f32>> = Self::CAMERA_DIRECTIONS
            .into_iter()
            .map(|item| Matrix4::look_at_rh([0.0; 3].into(), item[0].into(), item[1].into()))
            .collect();
        camera.set_width(output_dimensions.0);
        camera.set_height(output_dimensions.1);
        camera.set_fovy(Rad(std::f32::consts::FRAC_PI_2));

        for index in 0..6 {
            let projection: [[f32; 4]; 4] = camera.get_matrix().into();
            let view: [[f32; 4]; 4] = camera_directions[index].into();

            frame_buffer.clear_color(1.0, 0.0, 0.0, 0.0);
            frame_buffer.clear_depth(1.0);

            frame_buffer
                .draw(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    &program,
                    &gen_uniforms(projection, view),
                    &DrawParameters {
                        depth: glium::Depth {
                            test: glium::DepthTest::IfLessOrEqual,
                            write: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .unwrap();

            let mut output = Vec::new();
            for pixel in buffer_texture.read_to_pixel_buffer().read()? {
                output.push(pixel.0);
                output.push(pixel.1);
                output.push(pixel.2);
                output.push(pixel.3);
            }

            let output_image = DynamicImage::ImageRgba8(
                ImageBuffer::from_raw(output_dimensions.0, output_dimensions.1, output).unwrap(),
            );

            output_image
                .save(
                    output_directory
                        .with_file_name(Self::FILE_NAMES[index])
                        .with_extension(extension),
                )
                .unwrap();
        }
        Ok(())
    }
}

/// Used to generate cube vertices for the [`CubemapRender`] to use, it is not entirely meant to be
/// used by the crate users. Use at your own risk.
pub fn get_cube_vertices() -> Vec<Vertex> {
    let output = vec![
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
    ];

    output
}
