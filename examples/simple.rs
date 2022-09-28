use ezgl::{gl, winit};
use gl::HasContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut size = window.inner_size();
    let ezgl = ezgl::Ezgl::new(&window);

    unsafe { ezgl.clear_color(0.1, 0.2, 0.3, 1.0) };

    event_loop.run(move |evt, _, flow| {
        log::trace!("{:?}", evt);

        flow.set_wait();

        match evt {
            Event::RedrawRequested(_) => unsafe {
                ezgl.clear(gl::COLOR_BUFFER_BIT);
                ezgl.swap_buffers().unwrap();
            },

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    flow.set_exit();
                }

                WindowEvent::Resized(new_size) => {
                    ezgl.resize(new_size);
                    size = new_size;
                }

                WindowEvent::CursorMoved { position, .. } => unsafe {
                    ezgl.clear_color(
                        position.x as f32 / size.width as f32,
                        position.y as f32 / size.height as f32,
                        0.3,
                        1.0,
                    );
                    window.request_redraw();
                },

                _ => {}
            },

            _ => {}
        }
    });
}
