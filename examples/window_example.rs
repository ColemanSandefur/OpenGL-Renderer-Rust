use egui::style::Margin;
use glium::IndexBuffer;
use glium::VertexBuffer;
use nalgebra::Perspective3;
use opengl_renderer::shaders::brdf::BRDF;
use opengl_renderer::shaders::equi_rect_to_cubemap::EquiRectCubemap;
use opengl_renderer::shaders::irradiance_convolution::IrradianceConvolution;
use opengl_renderer::shaders::pbr::PBR;
use opengl_renderer::shaders::prefilter::Prefilter;
use opengl_renderer::shaders::skybox::Skybox;
use opengl_renderer::utils::model::ModelLoad;
use opengl_renderer::utils::pbr_skybox::PBRSkybox;
use opengl_renderer::utils::texture_loader::TextureLoader;
use std::rc::Rc;

use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin;
use glium::texture::DepthTexture2d;
use glium::texture::SrgbTexture2d;
use glium::Surface;
use opengl_renderer::renderer::Renderable;
use opengl_renderer::renderer::Renderer;
use opengl_renderer::utils::camera::Camera;
use opengl_renderer::{system_loop::SystemLoop, window::Window};

use opengl_renderer::utils::model::Model;
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

    let mut sphere = Model::load_from_fs(&facade, "resources/objects/sphere.glb").unwrap();

    sphere.get_shader_mut().get_pbr_params_mut().set_albedo(
        TextureLoader::from_memory_f32(&facade, &[0.0, 1.0, 1.0], 1, 1)
            .unwrap()
            .into(),
    );

    let mut cube = Model::new(
        VertexBuffer::new(&facade, &opengl_renderer::utils::shapes::get_cube()).unwrap(),
        IndexBuffer::new(
            &facade,
            glium::index::PrimitiveType::TrianglesList,
            &(0..36).into_iter().collect::<Vec<_>>(),
        )
        .unwrap(),
        PBR::load_from_fs(&facade),
    );

    cube.set_position([-2.0, 0.0, -1.0].into());
    cube.get_shader_mut()
        .get_pbr_params_mut()
        .set_albedo(BRDF::load_from_fs(&facade).compute(&facade).into());

    let pbr_skybox = {
        let skybox_cubemap = EquiRectCubemap::load_from_fs(&facade).compute(
            &facade,
            &TextureLoader::from_fs_hdr(&facade, "resources/textures/newport_loft.hdr").unwrap(),
            512,
        );

        let irradiance =
            IrradianceConvolution::load_from_fs(&facade).calculate(&facade, &skybox_cubemap);

        let prefilter = Prefilter::load_from_fs(&facade).compute(&facade, &skybox_cubemap);

        PBRSkybox::new(
            skybox_cubemap.into(),
            irradiance.into(),
            prefilter.into(),
            BRDF::load_from_fs(&facade).compute(&facade).into(),
        )
    };

    let skybox = Model::new(
        VertexBuffer::new(&facade, &opengl_renderer::utils::shapes::get_cube()).unwrap(),
        IndexBuffer::new(
            &facade,
            glium::index::PrimitiveType::TrianglesList,
            &(0..36).into_iter().collect::<Vec<_>>(),
        )
        .unwrap(),
        Skybox::load_from_fs(&facade),
    );

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

        egui::SidePanel::new(egui::panel::Side::Left, "model")
            .default_width(0.0)
            .show(&render_info.egui_glium.egui_ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.scope(|ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);

                        ui.label("Camera");
                    });

                    ui.label("position");
                    ui.horizontal(|ui| {
                        let mut position: [f32; 3] = camera.position.into();
                        ui.add(
                            egui::DragValue::new(&mut position[0])
                                .speed(0.1)
                                .prefix("x: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut position[1])
                                .speed(0.1)
                                .prefix("y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut position[2])
                                .speed(0.1)
                                .prefix("z: "),
                        );
                        camera.position = position.into();
                    });

                    ui.label("rotation");
                    ui.horizontal(|ui| {
                        let mut pitch = camera.get_pitch_rad().to_degrees();
                        if ui
                            .add(egui::DragValue::new(&mut pitch).prefix("pitch: ").speed(1))
                            .changed()
                        {
                            camera.set_pitch_rad(pitch.to_radians());
                        }
                        let mut yaw = camera.get_yaw_rad().to_degrees();
                        if ui
                            .add(egui::DragValue::new(&mut yaw).prefix("yaw: ").speed(1))
                            .changed()
                        {
                            camera.set_yaw_rad(yaw.to_radians());
                        }
                    });
                    ui.separator();

                    ui.push_id("sphere", |ui| {
                        sphere.debug_ui(ui);
                    });
                    ui.push_id("cube", |ui| {
                        cube.debug_ui(ui);
                    });
                })
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(Margin::same(0.0)))
            .show(&render_info.egui_glium.egui_ctx, |ui| {
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
                    render_texture.width().max(1) as f32 / render_texture.height().max(1) as f32,
                    70.0f32.to_radians(),
                    0.1,
                    100.0,
                )
                .as_matrix()
                .clone()
                .into();

                scene.scene_data.camera = camera.clone();
                scene.scene_data.set_scene_object(pbr_skybox.clone());

                sphere.publish(&mut scene);
                cube.publish(&mut scene);
                skybox.publish(&mut scene);

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
