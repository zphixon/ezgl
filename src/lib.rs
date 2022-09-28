pub use glow as gl;
pub use glutin;
pub use raw_window_handle;

#[cfg(feature = "winit")]
pub use winit;

use gl::Context;
use glutin::{
    api::glx::XlibErrorHookRegistrar,
    config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext, PossiblyCurrentGlContext,
    },
    display::{Display, GlDisplay},
    error::Result,
    surface::{GlSurface, Surface, SurfaceAttributes, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::num::NonZeroU32;

pub struct Ezgl {
    surface: Surface<WindowSurface>,
    glutin: PossiblyCurrentContext,
    glow: Context,
}

impl Ezgl {
    #[cfg(feature = "winit")]
    pub fn with_winit_window(window: &winit::window::Window) -> Result<Self> {
        let winit::dpi::PhysicalSize { width, height } = window.inner_size();

        #[cfg(unix)]
        let reg = Some(Box::new(winit::platform::unix::register_xlib_error_hook)
            as glutin::api::glx::XlibErrorHookRegistrar);

        #[cfg(not(unix))]
        let reg = None;

        Self::new(window, width, height, reg)
    }

    pub fn new<H: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &H,
        width: u32,
        height: u32,
        reg: Option<XlibErrorHookRegistrar>,
    ) -> Result<Self> {
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let display = create_display(display_handle, window_handle, reg)?;
        let template = config_template(window_handle);

        let config = unsafe {
            display
                .find_configs(template)?
                .reduce(|accum, config| {
                    if config.sample_buffers() > accum.sample_buffers() {
                        config
                    } else {
                        accum
                    }
                })
                .expect("No configs found :(")
        };

        let attributes = surface_attributes(&window, width, height);
        let surface = unsafe { display.create_window_surface(&config, &attributes)? };
        let context_attributes = ContextAttributesBuilder::new().build(Some(window_handle));

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window_handle));

        let context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .or_else(|_| display.create_context(&config, &fallback_context_attributes))?
        };

        let glutin = context.make_current(&surface)?;
        let glow = unsafe {
            Context::from_loader_function(|symbol| {
                let cstring = std::ffi::CString::new(symbol).unwrap();
                glutin.get_proc_address(&cstring)
            })
        };

        Ok(Self {
            surface,
            glutin,
            glow,
        })
    }

    pub fn resize(&self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.surface.resize(
            &self.glutin,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> Result<()> {
        self.surface.swap_buffers(&self.glutin)
    }

    pub fn glow_context(&self) -> &Context {
        &self.glow
    }
}

impl std::ops::Deref for Ezgl {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.glow
    }
}

fn create_display(
    raw_display: RawDisplayHandle,
    _raw_window_handle: RawWindowHandle,
    _reg: Option<XlibErrorHookRegistrar>,
) -> Result<Display> {
    use glutin::display::DisplayApiPreference;

    #[cfg(all(unix, not(target_os = "macos")))]
    let preference = if let Some(reg) = _reg {
        DisplayApiPreference::GlxThenEgl(reg)
    } else {
        DisplayApiPreference::Egl
    };

    #[cfg(all(unix, target_os = "macos"))]
    let preference = DisplayApiPreference::Cgl;

    #[cfg(windows)]
    let preference = DisplayApiPreference::Wgl(Some(_raw_window_handle));

    unsafe { Display::from_raw(raw_display, preference) }
}

fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
    let builder = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .compatible_with_native_window(raw_window_handle)
        .with_surface_type(ConfigSurfaceTypes::WINDOW);

    builder.build()
}

fn surface_attributes<H: HasRawWindowHandle + HasRawDisplayHandle>(
    window: &H,
    width: u32,
    height: u32,
) -> SurfaceAttributes<WindowSurface> {
    let raw_window_handle = window.raw_window_handle();
    SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    )
}
