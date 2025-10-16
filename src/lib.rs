#![allow(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod camera;
pub mod entities;
pub mod helpers;
pub mod renderer;

use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use std::error::Error;
use winit::window::{Window, WindowAttributes};

use crate::app::App;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

pub fn main(event_loop: winit::event_loop::EventLoop<()>) -> Result<(), Box<dyn Error>> {
    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes()));

    let mut app = App::new(template, display_builder);
    event_loop.run_app(&mut app)?;

    app.exit_state
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_transparent(true)
        .with_title("Glutin triangle gradient example (press Escape to exit)")
}

#[allow(clippy::large_enum_variant)]
enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(DisplayBuilder),
    /// The display was already created for the application.
    Init,
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event_loop::EventLoop;

    #[test]
    fn test_run_main() {
        main(EventLoop::new().unwrap()).unwrap();
    }
}
