use std::rc::Rc;

use glium::glutin;
use glium::Display;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::ContextCurrentState;

pub struct Window {
    pub display: Rc<Display>,
    pub event_loop: Option<EventLoop<()>>,
}

impl Window {
    pub fn create<T>(window_builder: WindowBuilder, context_builder: ContextBuilder<T>) -> Self
    where
        T: ContextCurrentState,
    {
        let event_loop = EventLoop::new();

        let display = Display::new(window_builder, context_builder, &event_loop).unwrap();

        Self {
            event_loop: Some(event_loop),
            display: Rc::new(display),
        }
    }
}
