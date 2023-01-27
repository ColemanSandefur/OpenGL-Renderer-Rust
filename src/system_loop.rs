use std::time::{Duration, Instant};

use crate::window::Window;
use egui::FontDefinitions;
use egui_glium::EguiGlium;
use glium::{glutin, Frame, Surface};
use glutin::event::Event;
use glutin::event_loop::EventLoop;

pub struct RenderInfo<'a> {
    pub target: &'a mut Frame,
    pub window: &'a Window,
    pub delta: &'a Duration,
    pub egui_glium: &'a mut EguiGlium,
}

pub struct SystemLoop {
    window: Window,
    render_handlers: Vec<Box<dyn FnMut(&mut RenderInfo)>>,
    event_handlers: Vec<Box<dyn FnMut(&Event<'_, ()>)>>,
    egui_glium: EguiGlium,
    event_loop: EventLoop<()>,
}

impl SystemLoop {
    pub fn new(mut window: Window) -> Self {
        let event_loop = window.event_loop.take().unwrap();
        let egui_glium = egui_glium::EguiGlium::new(&window.display, &event_loop);

        // Load raw opengl library for later use where glium doesn't provide needed functions
        // Currently the only way to use cubemaps (that I can find) is to crate one with gl and
        // then give it to glium
        gl::load_with(|s| window.display.gl_window().get_proc_address(s));

        Self {
            window,
            render_handlers: Vec::new(),
            event_handlers: Vec::new(),
            egui_glium,
            event_loop,
        }
    }

    pub fn subscribe_render(&mut self, event: impl FnMut(&mut RenderInfo) + 'static) {
        self.render_handlers.push(Box::new(event));
    }

    pub fn get_egui_glium(&self) -> &EguiGlium {
        &self.egui_glium
    }
    pub fn get_egui_glium_mut(&mut self) -> &mut EguiGlium {
        &mut self.egui_glium
    }

    pub fn start(self) -> ! {
        let SystemLoop {
            window,
            mut render_handlers,
            mut event_handlers,
            mut egui_glium,
            event_loop,
        } = self;

        let mut last_frame = Instant::now();

        let font_defs = {
            let mut fonts = FontDefinitions::default();

            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_static(include_bytes!("../resources/fonts/Roboto-Medium.ttf")),
            ); // .ttf and .otf supported

            // Put my font first (highest priority):
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "my_font".to_owned());

            // Put my font as last fallback for monospace:
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("my_font".to_owned());

            fonts
        };

        egui_glium.egui_ctx.set_fonts(font_defs);

        event_loop.run(move |event, _, control_flow| {
            for event_handler in &mut event_handlers {
                event_handler(&event);
            }

            match event {
                Event::RedrawRequested(_) => {
                    let mut target = window.display.draw();
                    let now = Instant::now();
                    let delta = now - last_frame;

                    target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

                    let _duration = egui_glium.run(&window.display, |egui_ctx| {});
                    {
                        let mut render_info = RenderInfo {
                            window: &window,
                            target: &mut target,
                            delta: &delta,
                            egui_glium: &mut egui_glium,
                        };

                        for event in &mut render_handlers {
                            event(&mut render_info);
                        }
                    }

                    egui_glium.paint(&window.display, &mut target);

                    target.finish().expect("Failed to swap buffers");

                    last_frame = now;
                    window.display.gl_window().window().request_redraw();
                }
                Event::WindowEvent {
                    event: glutin::event::WindowEvent::CloseRequested,
                    ..
                } => *control_flow = glutin::event_loop::ControlFlow::Exit,
                Event::WindowEvent { event, .. } => {
                    let _ = egui_glium.on_event(&event);
                }
                _ => (),
            }
        });
    }
}
