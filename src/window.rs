use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::wayland::WindowAttributesExtWayland,
    window::{Window, WindowId},
};

use pollster::FutureExt;

#[derive(Default)]
struct App<'a> {
    state: Option<State<'a>>,
}

use crate::state::State;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");

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

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: WindowId,
            event: WindowEvent,
        ) {
            if !self.state.as_mut().unwrap().input(&event) {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("The close button was pressed; stopping");
                        event_loop.exit();
                    }

                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => event_loop.exit(),

                    WindowEvent::RedrawRequested => {
                        let state = self.state.as_mut().unwrap();
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    WindowEvent::MouseInput { button, .. }
                        if button == winit::event::MouseButton::Left =>
                    {
                        let window = self.state.as_mut().unwrap().window();
                        window.drag_window().unwrap();
                    }

                    WindowEvent::Resized(physical_size) => {
                        let state = self.state.as_mut().unwrap();
                        state.resize(physical_size);
                    }
                    _ => (),
                }
            }
        }
    }
    let mut app = App::default();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}
