use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::wayland::WindowAttributesExtWayland,
    window::{Window, WindowId},
};

use pollster::FutureExt;

#[derive(Default)]
struct App<'a> {
    state: Option<State<'a>>,
}

use crate::state::State;

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let buttons = winit::window::WindowButtons::all();

        let window_attributes = Window::default_attributes()
            .with_title("lolitop!")
            .with_decorations(false)
            .with_enabled_buttons(buttons)
            .with_name("se.frikod.lolitop", "main")
            .with_transparent(true)
            .with_resizable(true);

        let window: Window = event_loop.create_window(window_attributes).unwrap();
        let state = State::new(window).block_on();
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let input_response = self.state.as_mut().unwrap().input(&event);
        if input_response.exit_requested {
            event_loop.exit();
            return;
        }

        if !input_response.consumed {
            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                }

                WindowEvent::RedrawRequested => {
                    let state = self.state.as_mut().unwrap();
                    state.update();
                    state.render();
                }

                WindowEvent::Resized(physical_size) => {
                    let state = self.state.as_mut().unwrap();
                    state.resize(physical_size);
                }
                _ => (),
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.state = None;
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}
