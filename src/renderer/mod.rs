use std::{ffi::CStr, ops::Deref, sync::Arc, time::Instant};

use crate::{
    gl::{self, Gles2},
    renderer::shader::GlslPass,
};

pub mod shader;

pub struct Renderer {
    creation: Instant,
    gl: Arc<gl::Gl>,
}

impl Renderer {
    pub fn new(gl_fns: Arc<Gles2>) -> Self {
        if let Some(renderer) = get_gl_string(&gl_fns, gl::RENDERER) {
            log::info!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(&gl_fns, gl::VERSION) {
            log::info!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(&gl_fns, gl::SHADING_LANGUAGE_VERSION) {
            log::info!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        Self {
            creation: Instant::now(),
            gl: gl_fns,
        }
    }

    pub fn draw<'a, I>(&mut self, objects: I)
    where
        I: Iterator<Item = &'a mut dyn GlslPass>,
    {
        let dt = self.creation.elapsed();
        for obj in objects {
            obj.update(&dt);
            obj.draw();
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
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
