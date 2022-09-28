# ez gl

Major refactorings in progress in the [Glutin](https://github.com/rust-windowing/glutin) project are underway to decouple it from [Winit](https://github.com/rust-windowing/winit). While this is a big win for those who want fine-grained control over how they get their GL context set up, it adds some complexity for others who don't particularly care (that is, myself).

This library aims to reduce the friction between the user and sweet, sweet GL calls, via [glow](https://github.com/grovesNL/glow). Here's how:

```rust
use ezgl::Ezgl;
use winit::{event_loop::EventLoop, window::WindowBuilder};

// do the standard winit stuff
let event_loop = EventLoop::new();
let window = WindowBuilder::new().build(&event_loop).unwrap();

// set up ezgl
let ezgl = Ezgl::new(&window);

// and off we go
event_loop.run(move |evt, _, flow| {
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
```

Todo:

- Increase support
  - [ ] Android
  - [ ] iOS
  - [ ] Web
