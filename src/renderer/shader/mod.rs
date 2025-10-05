use std::{sync::Arc, time::Duration};

use crate::gl::{self, Gles2};

pub trait GlslPass {
    // Create GPU resources; should be idempotent or guarded by internal state.
    fn init(&mut self, gl_fns: Arc<Gles2>);

    // Per-frame updates (uniforms, buffers, animations).
    fn update(&mut self, dt: &Duration);

    // Issue draw calls. Caller ensures framebuffer and other global state.
    fn draw(&self);
}

pub unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    unsafe {
        let shader = gl.CreateShader(shader);
        gl.ShaderSource(
            shader,
            1,
            [source.as_ptr().cast()].as_ptr(),
            std::ptr::null(),
        );
        gl.CompileShader(shader);
        shader
    }
}
