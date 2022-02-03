use glium::glutin;
use glium::Display;
use glium::Frame;
use glium::Surface;
use glutin::event::Event;
use glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

pub struct System {
    pub event_loop: EventLoop<()>,
    pub display: Rc<Display>,
}

impl System {
    pub fn init(title: &str) -> Self {
        let title = match Path::new(&title).file_name() {
            Some(file_name) => file_name.to_str().unwrap(),
            None => title,
        };

        let event_loop = EventLoop::new();
        let context = glutin::ContextBuilder::new()
            .with_depth_buffer(24)
            .with_multisampling(16)
            .with_vsync(false)
            .with_double_buffer(Some(true))
            .with_srgb(true)
            .with_hardware_acceleration(Some(true));
        let builder = WindowBuilder::new()
            .with_title(title.to_owned())
            .with_inner_size(glutin::dpi::LogicalSize::new(512f64, 384f64));
        let display = Rc::new(
            Display::new(builder, context, &event_loop).expect("Failed to initialize display"),
        );

        // Load raw opengl library for later use where glium doesn't provide needed functions
        // Currently the only way to use cubemaps (that I can find) is to crate one with gl and
        // then give it to glium
        gl::load_with(|s| display.gl_window().get_proc_address(s));

        Self {
            event_loop,
            display,
        }
    }

    pub fn main_loop(
        self,
        mut event_loop_fn: impl FnMut(&Event<'_, ()>, &mut ControlFlow) + 'static,
        mut draw_fn: impl FnMut(&mut Frame, Duration, &egui::CtxRef) + 'static,
        mut gui_fn: impl FnMut(&egui::CtxRef) + 'static,
    ) {
        let System {
            event_loop,
            display,
            ..
        } = self;

        let mut last_frame = Instant::now();
        let mut egui_glium = egui_glium::EguiGlium::new(&display);

        event_loop.run(move |event, _, control_flow| {
            event_loop_fn(&event, control_flow);

            match event {
                Event::RedrawRequested(_) => {
                    let mut target = display.draw();

                    target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

                    let now = Instant::now();

                    let delta = now - last_frame;

                    // Render behind egui

                    // Render egui
                    let (_repaint, shapes) = egui_glium.run(&display, |egui_ctx| {
                        draw_fn(&mut target, delta, egui_ctx);
                        gui_fn(egui_ctx);
                    });

                    egui_glium.paint(&display, &mut target, shapes);

                    target.finish().expect("Failed to swap buffers");

                    last_frame = now;
                    display.gl_window().window().request_redraw();
                }
                Event::WindowEvent {
                    event: glutin::event::WindowEvent::CloseRequested,
                    ..
                } => *control_flow = glutin::event_loop::ControlFlow::Exit,

                Event::WindowEvent {event, ..} => {
                    use glutin::event::WindowEvent;

                    if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }

                    egui_glium.on_event(&event);
                }

                _event => {}
            }
        })
    }
}
