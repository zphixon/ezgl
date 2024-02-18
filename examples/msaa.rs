#[cfg(not(feature = "winit"))]
fn main() {
    panic!("this example requires winit");
}

#[cfg(feature = "winit")]
fn main() {
    use ezgl::{gl, Ezgl};
    use gl::HasContext;
    use winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    env_logger::init();

    // normal setup to begin with
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut size = window.inner_size();

    // pass preferred samples to constructor
    let samples = 4i32;
    let ezgl = Ezgl::with_winit_window(&window, Some(samples as u8)).unwrap();

    // do msaa setup (see https://learnopengl.com/Advanced-OpenGL/Anti-Aliasing)
    let fb = unsafe {
        ezgl.enable(gl::DEBUG_OUTPUT);

        ezgl.clear_color(0.1, 0.2, 0.3, 1.0);
        ezgl.enable(gl::MULTISAMPLE);

        // create framebuffer
        let fb = ezgl.create_framebuffer().unwrap();
        ezgl.bind_framebuffer(gl::FRAMEBUFFER, Some(fb));

        // set up multisampled color attachment texture
        let tex = ezgl.create_texture().unwrap();
        ezgl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, Some(tex));
        ezgl.tex_image_2d_multisample(
            gl::TEXTURE_2D_MULTISAMPLE,
            samples,
            gl::RGBA8 as i32,
            size.width as i32,
            size.height as i32,
            true,
        );
        ezgl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, None);
        ezgl.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D_MULTISAMPLE,
            Some(tex),
            0,
        );

        assert_eq!(
            ezgl.check_framebuffer_status(gl::FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE
        );

        ezgl.bind_framebuffer(gl::FRAMEBUFFER, None);

        let vert = ezgl.create_shader(gl::VERTEX_SHADER).unwrap();
        let frag = ezgl.create_shader(gl::FRAGMENT_SHADER).unwrap();

        ezgl.shader_source(
            vert,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/triangle.vert"
            )),
        );
        ezgl.compile_shader(vert);
        assert!(ezgl.get_shader_compile_status(vert));

        ezgl.shader_source(
            frag,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/triangle.frag"
            )),
        );
        ezgl.compile_shader(frag);
        assert!(ezgl.get_shader_compile_status(frag));

        let program = ezgl.create_program().unwrap();
        ezgl.attach_shader(program, vert);
        ezgl.attach_shader(program, frag);
        ezgl.link_program(program);
        assert!(
            ezgl.get_program_link_status(program),
            "{}",
            ezgl.get_program_info_log(program)
        );

        ezgl.use_program(Some(program));

        // now requires a vertex array be bound?
        let triangle_vertex_array = ezgl.create_vertex_array().unwrap();
        ezgl.bind_vertex_array(Some(triangle_vertex_array));

        fb
    };

    let result = event_loop.run(move |evt, loop_target| {
        log::trace!("{:?}", evt);

        loop_target.set_control_flow(ControlFlow::Wait);

        match evt {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => unsafe {
                    // 1. bind multisampled framebuffer
                    ezgl.bind_framebuffer(gl::FRAMEBUFFER, Some(fb));

                    // 2. draw scene like normal
                    ezgl.clear(gl::COLOR_BUFFER_BIT);
                    ezgl.draw_arrays(gl::TRIANGLES, 0, 3);

                    // 3. copy multisampled buffer to backbuffer
                    ezgl.bind_framebuffer(gl::READ_FRAMEBUFFER, Some(fb));
                    ezgl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, None);
                    ezgl.blit_framebuffer(
                        0,
                        0,
                        size.width as i32,
                        size.height as i32,
                        0,
                        0,
                        size.width as i32,
                        size.height as i32,
                        gl::COLOR_BUFFER_BIT,
                        gl::NEAREST,
                    );

                    ezgl.swap_buffers().unwrap();
                },

                WindowEvent::CloseRequested => {
                    loop_target.exit();
                }

                WindowEvent::Resized(new_size) => {
                    size = new_size;
                    ezgl.resize(size.width, size.height);

                    unsafe {
                        ezgl.viewport(0, 0, size.width as i32, size.height as i32);
                        ezgl.bind_framebuffer(gl::FRAMEBUFFER, Some(fb));

                        // re-make multisampled color attachment texture
                        let tex = ezgl.create_texture().unwrap();
                        ezgl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, Some(tex));
                        ezgl.tex_image_2d_multisample(
                            gl::TEXTURE_2D_MULTISAMPLE,
                            samples,
                            gl::RGBA8 as i32,
                            size.width as i32,
                            size.height as i32,
                            true,
                        );
                        ezgl.bind_texture(gl::TEXTURE_2D_MULTISAMPLE, None);
                        ezgl.framebuffer_texture_2d(
                            gl::FRAMEBUFFER,
                            gl::COLOR_ATTACHMENT0,
                            gl::TEXTURE_2D_MULTISAMPLE,
                            Some(tex),
                            0,
                        );
                    }
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
