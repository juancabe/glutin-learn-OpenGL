use crate::{
    gl::{self, Gles2},
    helpers::Mat3DUpdate,
};
use std::{ffi::CStr, sync::Arc};

pub trait GlslPass {
    // Create GPU resources; should be idempotent or guarded by internal state.
    fn init(&mut self, gl_fns: Arc<Gles2>, mat3d: Mat3DUpdate);

    // Per-frame updates (uniforms, buffers, animations). Caller ensures active shader
    fn update(&mut self, mat3d: Mat3DUpdate);

    // Issue draw calls. Caller ensures active shader
    fn draw(&self);

    // gl FFI getter
    fn get_shader(&self) -> u32;

    fn update_draw(&mut self, mat3d: Mat3DUpdate, gl: &Gles2) {
        unsafe { gl.UseProgram(self.get_shader()) };
        self.update(mat3d);
        self.draw();
    }
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
