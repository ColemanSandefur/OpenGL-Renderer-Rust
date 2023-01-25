use nalgebra::Matrix4;
use nalgebra::Perspective3;
use nalgebra::Vector3;
use opengl_renderer::renderer::RenderScene;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use std::rc::Rc;

use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin;
use glium::texture::DepthTexture2d;
use glium::texture::SrgbTexture2d;
use glium::IndexBuffer;
use glium::Surface;
use glium::VertexBuffer;
use opengl_renderer::camera::Camera;
use opengl_renderer::renderer::Renderable;
use opengl_renderer::renderer::Renderer;
use opengl_renderer::shaders::pbr::PBR;
use opengl_renderer::vertex::Vertex;
use opengl_renderer::{system_loop::SystemLoop, window::Window};

fn main() {
    let window = create_window();
    let facade = window.display.clone().get_context().clone();

    let mut event_loop = SystemLoop::new(window);
    let mut debug_open = true;

    let mut render_texture = RenderSurface::new(&facade, 100, 100).unwrap();
    let egui_texture: egui::TextureId = event_loop
        .get_egui_glium_mut()
        .painter
        .register_native_texture(render_texture.texture.clone(), Default::default());

    let mut renderer = Renderer::new();

    let sphere = Object::from_fs(&facade).unwrap();

    let mut camera = Camera::new();
    camera.position = [0.0, 0.0, 3.0].into();

    event_loop.subscribe_render(move |render_info| {
        render_info.target.clear_color(0.0, 0.0, 0.0, 1.0);

        egui::TopBottomPanel::top("topbar").show(&render_info.egui_glium.egui_ctx, |ui| {
            if ui.button("show debug").clicked() {
                debug_open = true;
            }
        });

        egui::Window::new("debug")
            .collapsible(false)
            .anchor(egui::Align2::RIGHT_TOP, [0.0; 2])
            .open(&mut debug_open)
            .show(&render_info.egui_glium.egui_ctx, |ui| {
                ui.label(&format!(
                    "time: {:.2}",
                    render_info.delta.as_secs_f32() * 1000.0,
                ));

                ui.label(&format!(
                    "fps: {:.2}",
                    1.0 / render_info.delta.as_secs_f32()
                ));

                ui.label(&format!(
                    "res: {}x{}",
                    render_texture.width(),
                    render_texture.height()
                ));
            });

        egui::SidePanel::new(egui::panel::Side::Left, "model").show(
            &render_info.egui_glium.egui_ctx,
            |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    let mut position: [f32; 3] = camera.position.into();
                    ui.add(egui::DragValue::new(&mut position[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position[1]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position[2]).speed(0.1));

                    camera.position = position.into();
                })
            },
        );

        egui::CentralPanel::default().show(&render_info.egui_glium.egui_ctx, |ui| {
            let size = ui.available_size();

            // resize the render texture if the window size changed
            let mut size_px = ui.available_size();
            size_px.x *= render_info.egui_glium.egui_ctx.pixels_per_point();
            size_px.y *= render_info.egui_glium.egui_ctx.pixels_per_point();

            if size_px.x != render_texture.width() as f32
                || size_px.y != render_texture.height() as f32
            {
                render_texture
                    .resize(&facade, size_px.x as u32, size_px.y as u32)
                    .unwrap();
                render_info.egui_glium.painter.replace_native_texture(
                    egui_texture,
                    render_texture.texture.clone(),
                    egui::TextureOptions::default(),
                );
            }

            // render to 'render_texture'
            let mut buffer = render_texture.frame_buffer(&facade).unwrap();

            buffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            let mut scene = renderer.begin_scene();
            scene.scene_data.projection = Perspective3::new(
                render_texture.width() as f32 / render_texture.height() as f32,
                std::f32::consts::PI / 4.0,
                0.1,
                100.0,
            )
            .as_matrix()
            .clone()
            .into();

            scene.scene_data.camera = camera.clone();

            sphere.publish(&mut scene);

            scene.finish(&mut Renderable::from(&mut buffer));

            // show our rendered texture, but the image is upside, down so let's change the uv
            // coords of the image
            ui.add(
                egui::widgets::Image::new(egui_texture, size).uv(egui::Rect {
                    min: [0.0, 1.0].into(),
                    max: [1.0, 0.0].into(),
                }),
            );
        });
    });

    event_loop.start();
}

fn create_window() -> Window {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(0)
        .with_vsync(true);

    Window::create(window_builder, context_builder)
}

pub struct RenderSurface {
    pub texture: Rc<SrgbTexture2d>,
    pub depth: DepthTexture2d,
}

impl RenderSurface {
    pub fn new(
        facade: &impl Facade,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let texture = SrgbTexture2d::empty(facade, width, height)?;
        let depth = DepthTexture2d::empty(facade, width, height)?;

        Ok(Self {
            texture: Rc::new(texture),
            depth,
        })
    }

    pub fn resize(
        &mut self,
        facade: &impl Facade,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.texture = Rc::new(SrgbTexture2d::empty(facade, width, height)?);
        self.depth = DepthTexture2d::empty(facade, width, height)?;

        Ok(())
    }

    pub fn frame_buffer(
        &self,
        facade: &impl Facade,
    ) -> Result<SimpleFrameBuffer, Box<dyn std::error::Error>> {
        Ok(SimpleFrameBuffer::with_depth_buffer(
            facade,
            &*self.texture,
            &self.depth,
        )?)
    }

    pub fn size(&self) -> (u32, u32) {
        (self.texture.width(), self.texture.height())
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.height()
    }
}

struct Object {
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u32>,
    pbr: PBR,
}

impl Object {
    pub fn from_fs(facade: &impl Facade) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::from_file(
            "resources/sphere.glb",
            vec![
                // Quick fix, should change later
                PostProcess::PreTransformVertices,
                PostProcess::GenerateNormals,
            ],
        )?;

        let mesh = &scene.meshes[0];
        let vertices = (0..mesh.vertices.len())
            .map(|index| {
                let vertex = mesh.vertices[index as usize];
                let position: [f32; 3] = [vertex.x, vertex.y, vertex.z];
                let normal_vec = mesh.normals[index as usize];
                let normal = [normal_vec.x, normal_vec.y, normal_vec.z];
                let tex_coords = match mesh.texture_coords[0].as_ref() {
                    Some(texture_coords) => {
                        let vec3 = texture_coords[index as usize];
                        [vec3.x, vec3.y]
                    }
                    None => [0.0; 2],
                };

                return Vertex {
                    position,
                    normal,
                    tex_coords,
                    ..Default::default()
                };
            })
            .collect::<Vec<_>>();

        let indices = mesh
            .faces
            .iter()
            .flat_map(|face| face.0.clone())
            .collect::<Vec<_>>();

        let index_buffer =
            IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)?;
        let vertex_buffer = VertexBuffer::new(facade, &vertices)?;

        Ok(Self {
            indices: index_buffer,
            vertices: vertex_buffer,
            pbr: PBR::load_from_fs(facade),
        })
    }

    pub fn publish<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertices, &self.indices, &self.pbr);
    }
}
