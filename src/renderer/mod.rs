use std::{ffi::CStr, sync::Arc};

use glam::USizeVec2;

use crate::{
    gl::{self, Gles2},
    helpers::Mat3DUpdate,
    renderer::shader::GlslPass,
};

pub mod shader;

pub struct Renderer {
    window_dimensions: glam::USizeVec2,
    gl: Arc<gl::Gl>,
}

impl Renderer {
    pub fn new(gl_fns: Arc<Gles2>, window_dimensions: glam::USizeVec2) -> Self {
        if let Some(renderer) = get_gl_string(&gl_fns, gl::RENDERER) {
            log::info!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(&gl_fns, gl::VERSION) {
            log::info!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(&gl_fns, gl::SHADING_LANGUAGE_VERSION) {
            log::info!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        unsafe { gl_fns.Enable(gl::DEPTH_TEST) }

        Self {
            window_dimensions,
            gl: gl_fns,
        }
    }

    pub fn draw<'a, I>(&mut self, objects: I, mat3d: Mat3DUpdate)
    where
        I: Iterator<Item = &'a mut dyn GlslPass>,
    {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for obj in objects {
            obj.update_draw(mat3d, &self.gl);
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.window_dimensions = USizeVec2::new(width as usize, height as usize);
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    pub fn get_window_dimensions(&self) -> glam::USizeVec2 {
        self.window_dimensions
    }
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

// #[rustfmt::skip]
// static VERTEX_DATA: [f32; 15] = [
//     -0.5, -0.5,  1.0,  0.0,  0.0,
//      0.0,  0.5,  0.0,  1.0,  0.0,
//      0.5, -0.5,  0.0,  0.0,  1.0,
// ];
