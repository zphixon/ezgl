use ezgl::{gl, winit, Ezgl};
use gl::HasContext;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

fn main() {
    env_logger::init();

    // 1. make an event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut State::default()).unwrap();

    // 2. create a struct to hold everything
    #[derive(Default)]
    struct State {
        ezgl: Option<Ezgl>,
        window: Option<Window>,
        size: PhysicalSize<u32>,
    }

    // 3. implement ApplicationHandler for your struct (new in winit 0.30)
    impl ApplicationHandler for State {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            // 4. this function is typically called on startup. create your window:
            let window = event_loop
                .create_window(WindowAttributes::default())
                .unwrap();

            // 5. call with_winit_window
            let ezgl = Ezgl::with_winit_window(&window, None).unwrap();

            // 6. off we go!
            unsafe { ezgl.clear_color(0.1, 0.2, 0.3, 1.0) };

            self.size = window.inner_size();
            self.window = Some(window);
            self.ezgl = Some(ezgl);
        }

        fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
            debug_assert!(self.window.is_some());
            debug_assert!(self.ezgl.is_some());

            let ezgl = self.ezgl.as_ref().unwrap();

            match event {
                WindowEvent::RedrawRequested => unsafe {
                    ezgl.clear(gl::COLOR_BUFFER_BIT);
                    ezgl.swap_buffers().unwrap();
                },

                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }

                WindowEvent::Resized(new_size) => {
                    ezgl.resize(new_size.width, new_size.height);
                    unsafe { ezgl.viewport(0, 0, new_size.width as i32, new_size.height as i32) };
                    self.size = new_size;
                }

                WindowEvent::CursorMoved { position, .. } => unsafe {
                    ezgl.clear_color(
                        position.x as f32 / self.size.width as f32,
                        position.y as f32 / self.size.height as f32,
                        0.3,
                        1.0,
                    );
                    self.window.as_ref().unwrap().request_redraw();
                },

                WindowEvent::ScaleFactorChanged {
                    mut inner_size_writer,
                    ..
                } => {
                    inner_size_writer.request_inner_size(self.size).unwrap();
                }

                _ => {}
            }
        }
    }
}
