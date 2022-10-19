use egui::CtxRef;
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

pub struct SystemInfo<'a> {
    pub last_frame: &'a Instant,
    pub delta: &'a Duration,
    pub target: &'a mut Frame,
    pub display: &'a Rc<Display>,
    pub control_flow: &'a mut ControlFlow,
    pub event: &'a Event<'a, ()>,
    pub egui_ctx: &'a CtxRef,
}

pub struct System {
    pub event_loop: EventLoop<()>,
    pub display: Rc<Display>,
    render_events: Vec<Box<dyn FnMut(&mut SystemInfo<'_>)>>,
    event_handlers: Vec<Box<dyn FnMut(&Event<'_, ()>, &mut ControlFlow)>>,
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
            .with_vsync(true)
            //.with_double_buffer(Some(true))
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
            render_events: Vec::new(),
            event_handlers: Vec::new(),
        }
    }

    /// Subscribe a function to be ran every render iteration
    pub fn subscribe_render(&mut self, event: impl FnMut(&mut SystemInfo<'_>) + 'static) {
        self.render_events.push(Box::new(event));
    }

    /// Subscribe a function to be ran every render iteration
    pub fn subscribe_event_handler(
        &mut self,
        event: impl FnMut(&Event<'_, ()>, &mut ControlFlow) + 'static,
    ) {
        self.event_handlers.push(Box::new(event));
    }

    pub fn main_loop(self) {
        let System {
            event_loop,
            display,
            mut render_events,
            mut event_handlers,
            ..
        } = self;

        let mut last_frame = Instant::now();
        let mut egui_glium = egui_glium::EguiGlium::new(&display);

        event_loop.run(move |event, _, control_flow| {
            for handler in &mut event_handlers {
                handler(&event, control_flow);
            }

            match event {
                Event::RedrawRequested(_) => {
                    let mut target = display.draw();
                    let now = Instant::now();

                    let delta = now - last_frame;

                    target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

                    let (_repaint, shapes) = egui_glium.run(&display, |egui_ctx| {
                        let mut info = SystemInfo {
                            last_frame: &last_frame,
                            delta: &delta,
                            target: &mut target,
                            display: &display,
                            control_flow,
                            event: &event,
                            egui_ctx,
                        };
                        for event in &mut render_events {
                            event(&mut info)
                        }
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

                Event::WindowEvent { event, .. } => {
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
