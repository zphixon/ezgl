use gl::Context;
pub use glow as gl;
pub extern crate glutin;
pub extern crate winit;

use glutin::{
    config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext, PossiblyCurrentGlContext,
    },
    display::{Display, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributes, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::num::NonZeroU32;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Ezgl {
    surface: Surface<WindowSurface>,
    glutin: PossiblyCurrentContext,
    glow: Context,
}

impl Ezgl {
    pub fn new(window: &Window) -> Self {
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let display = create_display(display_handle, window_handle);
        let template = config_template(window_handle);

        let config = unsafe {
            display
                .find_configs(template)
                .unwrap()
                .reduce(|accum, config| {
                    if config.sample_buffers() > accum.sample_buffers() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        };

        let attributes = surface_attributes(&window);
        let surface = unsafe { display.create_window_surface(&config, &attributes).unwrap() };
        let context_attributes = ContextAttributesBuilder::new().build(Some(window_handle));

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window_handle));

        let context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .unwrap_or_else(|_| {
                    display
                        .create_context(&config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        let glutin = context.make_current(&surface).unwrap();
        let glow = unsafe {
            Context::from_loader_function(|symbol| {
                let cstring = std::ffi::CString::new(symbol).unwrap();
                glutin.get_proc_address(&cstring)
            })
        };

        Self {
            surface,
            glutin,
            glow,
        }
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface.resize(
            &self.glutin,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> Result<(), glutin::error::Error> {
        self.surface.swap_buffers(&self.glutin)
    }
}

impl std::ops::Deref for Ezgl {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.glow
    }
}

fn create_display(raw_display: RawDisplayHandle, _raw_window_handle: RawWindowHandle) -> Display {
    use glutin::display::DisplayApiPreference;

    #[cfg(all(unix, not(target_os = "macos")))]
    let preference =
        DisplayApiPreference::GlxThenEgl(Box::new(winit::platform::unix::register_xlib_error_hook));

    #[cfg(all(unix, target_os = "macos"))]
    let preference = DisplayApiPreference::Cgl;

    #[cfg(windows)]
    let preference = DisplayApiPreference::Wgl(Some(_raw_window_handle));

    unsafe { Display::from_raw(raw_display, preference).unwrap() }
}

fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
    let builder = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .compatible_with_native_window(raw_window_handle)
        .with_surface_type(ConfigSurfaceTypes::WINDOW);

    builder.build()
}

fn surface_attributes(window: &Window) -> SurfaceAttributes<WindowSurface> {
    let (width, height): (u32, u32) = window.inner_size().into();
    let raw_window_handle = window.raw_window_handle();
    SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    )
}
