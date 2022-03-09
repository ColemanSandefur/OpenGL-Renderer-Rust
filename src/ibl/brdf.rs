use glium::{
    backend::Facade, framebuffer::SimpleFrameBuffer, texture::DepthTexture2d, texture::Texture2d,
    IndexBuffer, Program, Surface, VertexBuffer,
};
use image::{DynamicImage, ImageBuffer};
use std::{error::Error, path::Path, sync::Arc};

use crate::vertex::Vertex;

#[derive(Clone)]
pub struct BRDF {
    program: Arc<Program>,
    vertex_buffer: Arc<VertexBuffer<Vertex>>,
    index_buffer: Arc<IndexBuffer<u32>>,
}

impl BRDF {
    pub fn new(facade: &impl Facade) -> Self {
        let program = crate::material::insert_program!(
            "../shaders/brdf/vertex.glsl",
            "../shaders/brdf/fragment.glsl",
            facade
        );

        let vertex_buffer = VertexBuffer::new(facade, &get_quad_vertices()).unwrap();
        let index_buffer = IndexBuffer::new(
            facade,
            glium::index::PrimitiveType::TrianglesList,
            &[0, 1, 2, 1, 3, 2],
        )
        .unwrap();

        Self {
            program: Arc::new(program),
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
        }
    }

    pub fn calculate_to_fs<P>(
        &self,
        facade: &impl Facade,
        output_file: P,
    ) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        const TARGET_RESOLUTION: (u32, u32) = (512, 512);
        let (width, height) = TARGET_RESOLUTION;

        // Buffers that will be written to
        let buffer_texture = Texture2d::empty(facade, width, height)?;
        let buffer_depth = DepthTexture2d::empty(facade, width, height)?;

        // Makes the buffers writable
        let mut buffer =
            SimpleFrameBuffer::with_depth_buffer(facade, &buffer_texture, &buffer_depth)?;

        let uniforms = uniform! {};

        buffer.draw(
            &*self.vertex_buffer,
            &*self.index_buffer,
            &*self.program,
            &uniforms,
            &Default::default(),
        )?;

        // Store to fs
        let mut output = Vec::new();
        for pixel in buffer_texture.read_to_pixel_buffer().read()? {
            output.push(pixel.0);
            output.push(pixel.1);
            output.push(pixel.2);
            output.push(pixel.3);
        }

        let image_buffer = ImageBuffer::from_raw(width, height, output)
            .ok_or("Failed to create image buffer when saving to fs")?;
        let output_image = DynamicImage::ImageRgba8(image_buffer);

        output_image.save(output_file)?;

        Ok(())
    }
}

fn get_quad_vertices() -> Vec<Vertex> {
    vec![
        Vertex {
            // Top Left
            position: [-1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        },
        Vertex {
            // Top Right
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        },
        Vertex {
            // Bottom Left
            position: [-1.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        },
        Vertex {
            // Bottom Right
            position: [1.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        },
    ]
}
