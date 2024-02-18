use ezgl::{gl, Ezgl};
use gl::HasContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(not(feature = "winit"))]
fn no_winit_ezgl(window: &winit::window::Window, size: winit::dpi::PhysicalSize<u32>) -> Ezgl {
    #[cfg(unix)]
    let reg = Some(Box::new(winit::platform::unix::register_xlib_error_hook)
        as ezgl::glutin::api::glx::XlibErrorHookRegistrar);

    #[cfg(not(unix))]
    let reg = None;

    Ezgl::new(&window, size.width, size.height, reg, None).unwrap()
}

fn main() {
    env_logger::init();

    // 1. make a window with HasRawWindowHandle + HasRawDisplayHandle
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut size = window.inner_size();

    // 2. if it's a winit window, use with_winit_window
    #[cfg(feature = "winit")]
    let ezgl = Ezgl::with_winit_window(&window, None).unwrap();

    // 2a. or don't
    #[cfg(not(feature = "winit"))]
    let ezgl = no_winit_ezgl(&window, size);

    // 3. off we go!
    unsafe { ezgl.clear_color(0.1, 0.2, 0.3, 1.0) };

    let result = event_loop.run(move |evt, loop_target| {
        log::trace!("{:?}", evt);
        assert!(ezgl.surface().is_current(ezgl.glutin()));

        loop_target.set_control_flow(ControlFlow::Wait);

        match evt {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => unsafe {
                    ezgl.clear(gl::COLOR_BUFFER_BIT);
                    ezgl.swap_buffers().unwrap();
                },

                WindowEvent::CloseRequested => {
                    loop_target.exit();
                }

                WindowEvent::Resized(new_size) => {
                    ezgl.resize(new_size.width, new_size.height);
                    unsafe { ezgl.viewport(0, 0, new_size.width as i32, new_size.height as i32) };
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

    result.unwrap();
}
