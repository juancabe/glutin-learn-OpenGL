use std::{ffi::CStr, sync::Arc, time::Duration};

use crate::{
    gl::{self, Gles2},
    helpers::Mat3D,
};

pub trait GlslPass {
    // Create GPU resources; should be idempotent or guarded by internal state.
    fn init(&mut self, gl_fns: Arc<Gles2>, mat3d: Option<Mat3D>);

    // Per-frame updates (uniforms, buffers, animations).
    fn update(&mut self, dt: Option<&Duration>, mat3d: Option<Mat3D>);

    // Issue draw calls. Caller ensures framebuffer and other global state.
    fn draw(&self);
}

/// # Safety
/// its doing ffi
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
        // print compile errors if any
        let mut success: gl::types::GLint = 0;
        gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success as *mut i32);
        if success == 0 {
            let mut info_log = [0i8; 512];
            gl.GetShaderInfoLog(shader, 512, std::ptr::null_mut(), &mut info_log as *mut i8);
            let cstr = CStr::from_ptr(info_log.as_mut_ptr());
            log::error!("Error compiling SHADER: {:?}", cstr.to_str());
        }

        shader
    }
}
