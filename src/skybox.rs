use crate::cubemap_loader::CubemapType;
use glium::texture::Texture2d;
use crate::renderer::RenderScene;
use glium::backend::Facade;
use glium::index::NoIndices;
use glium::VertexBuffer;

use crate::material::SkyboxMat;
use crate::vertex::Vertex;

pub struct Skybox {
    skybox: SkyboxMat,
    ibl: Option<CubemapType>,
    prefilter: Option<CubemapType>,
    brdf: Option<Texture2d>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
}

impl PartialEq for Skybox {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: [x, y, z],
        ..Default::default()
    }
}

impl Skybox {
    pub fn new(facade: &impl Facade, skybox: SkyboxMat) -> Self {
        let vertex_buffer = VertexBuffer::new(
            facade,
            &[
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, -1.0, 1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(1.0, 1.0, -1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(1.0, 1.0, 1.0),
                vertex(-1.0, 1.0, 1.0),
                vertex(-1.0, 1.0, -1.0),
                vertex(-1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(1.0, -1.0, -1.0),
                vertex(-1.0, -1.0, 1.0),
                vertex(1.0, -1.0, 1.0),
            ],
        )
        .unwrap();
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        Self {
            index_buffer,
            vertex_buffer,
            ibl: None,
            prefilter: None,
            brdf: None,
            skybox,
        }
    }
    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.skybox);
    }

    pub fn get_skybox(&self) -> &SkyboxMat {
        &self.skybox
    }

    pub fn set_ibl(&mut self, cubemap: Option<CubemapType>) {
        self.ibl = cubemap;
    }

    pub fn get_ibl(&self) -> &Option<CubemapType> {
        &self.ibl
    }

    pub fn set_prefilter(&mut self, cubemap: Option<CubemapType>) {
        self.prefilter = cubemap;
    }

    pub fn get_prefilter(&self) -> &Option<CubemapType> {
        &self.prefilter
    }

    pub fn set_brdf(&mut self, texture: Option<Texture2d>) {
        self.brdf = texture;
    }

    pub fn get_brdf(&self) -> &Option<Texture2d> {
        &self.brdf
    }
}
