//! Easy GL setup via [glutin]/[glow] for the user who doesn't care how they get their context.
//!
//! This crate reexports [glow] as `gl`, as well as [glutin] and [raw_window_handle]. Additionally
//! [winit](docs.rs/winit) is available if `feature = "winit"` is enabled.

pub use glow as gl;
pub use glutin;
pub use raw_window_handle;

#[cfg(feature = "winit")]
pub use winit;

use gl::Context;
use glutin::{
    config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext,
    },
    display::{Display, GlDisplay},
    error::Result,
    surface::{GlSurface, Surface, SurfaceAttributes, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::{num::NonZeroU32, sync::Arc};

/// Duplicate of [glutin::api::glx::XlibErrorHookRegistrar], except without the OS-based feature
/// gate.
pub type Reg =
    Box<dyn Fn(Box<dyn Fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> bool + Send + Sync>)>;

/// Struct handling GL information.
///
/// This type implements Deref into [Context]. Note that [ezgl::gl::HasContext](glow::HasContext)
/// must be in scope for GL functions to be available.
pub struct Ezgl {
    surface: Surface<WindowSurface>,
    glutin: PossiblyCurrentContext,
    glow: Arc<Context>,
}

impl Ezgl {
    /// Requires `feature = "winit"`.
    ///
    /// Set up ezgl using a winit [Window](winit::window::Window) directly, rather than through
    /// [HasRawWindowHandle] + [HasRawDisplayHandle] as in [Ezgl::new].
    #[cfg(feature = "winit")]
    pub fn with_winit_window(
        window: &winit::window::Window,
        prefer_samples: Option<u8>,
    ) -> Result<Self> {
        let winit::dpi::PhysicalSize { width, height } = window.inner_size();

        #[cfg(unix)]
        let reg = Some(Box::new(winit::platform::unix::register_xlib_error_hook) as Reg);

        #[cfg(not(unix))]
        let reg = None;

        Self::new(window, width, height, reg, prefer_samples)
    }

    /// Set up ezgl.
    ///
    /// Requires a window that implements [HasRawWindowHandle] + [HasRawDisplayHandle]. If
    /// `prefer_samples` is None, the context configuration with the greatest number of sample
    /// buffers is preferred.
    pub fn new<H: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &H,
        width: u32,
        height: u32,
        reg: Option<Reg>,
        prefer_samples: Option<u8>,
    ) -> Result<Self> {
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let display = create_display(display_handle, window_handle, reg)?;
        let template = config_template(window_handle);

        let config = unsafe {
            display
                .find_configs(template)?
                .reduce(|accum, config| {
                    if let Some(samples) = prefer_samples {
                        if config.num_samples() == samples {
                            config
                        } else {
                            accum
                        }
                    } else {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
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
        let glow = Arc::new(unsafe {
            Context::from_loader_function(|symbol| {
                let cstring = std::ffi::CString::new(symbol).unwrap();
                display.get_proc_address(&cstring)
            })
        });

        Ok(Self {
            surface,
            glutin,
            glow,
        })
    }

    /// Resize the GL surface.
    ///
    /// This method does not resize the GL viewport. If width or height are zero this method does
    /// nothing. Delegates to [Surface::resize].
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

    /// Display the next frame.
    ///
    /// Delegates to [Surface::swap_buffers].
    pub fn swap_buffers(&self) -> Result<()> {
        self.surface.swap_buffers(&self.glutin)
    }

    /// Increase the reference count of the inner glow [Context].
    pub fn glow_context(&self) -> Arc<Context> {
        Arc::clone(&self.glow)
    }

    /// Get the (possibly) current glutin context.
    pub fn glutin(&self) -> &PossiblyCurrentContext {
        &self.glutin
    }

    /// Get the surface corresponding with the window.
    pub fn surface(&self) -> &Surface<WindowSurface> {
        &self.surface
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
    _reg: Option<Reg>,
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

    unsafe { Display::new(raw_display, preference) }
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
    SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        )
}
