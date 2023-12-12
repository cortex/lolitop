use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), impl std::error::Error> {
    // wgpu::Instance::new();

    let event_loop = EventLoop::new().unwrap(); // Loop provided by winit for handling window events
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();
    event_loop.run(move |event, _| {
        println!("{event:?}");
    })
}
