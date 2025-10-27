use crate::{
    gl::{self, Gles2},
    helpers::Mat3DUpdate,
    renderer::shader::uniform::Uniform,
};
use std::{ffi::CStr, rc::Rc};

pub mod uniform;

#[derive(Clone, Debug, Default)]
pub struct IndexedElements {
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub ebo: gl::types::GLuint,
    pub index_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Array {
    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,

    /// Total amount of DrawArrays calls
    pub len: usize,
    /// Offset between DrawArrays call's vertex targets
    pub offset: usize,
    /// Count of vertices used per call to draw triangles
    pub count: usize,
}

#[derive(Clone)]
pub enum Drawable {
    Indexed(IndexedElements),
    Array(Array),
}

#[derive(Default, Clone)]
pub struct Tex {
    pub tex: gl::types::GLuint,
    pub target: gl::types::GLuint,
}

#[derive(Clone)]
pub struct Shader {
    pub program: gl::types::GLuint,
    pub drawables: Vec<Drawable>,
    pub tex: Option<Tex>,
    pub model_transform: glam::Mat4,
    pub gl_fns: Rc<Gles2>,
}

impl Shader {
    /// # Safety
    /// FFI call
    pub unsafe fn use_program(&self) {
        self.gl_fns.UseProgram(self.program);
    }

    /// # Safety
    /// FFI call
    pub unsafe fn delete_gl(&self) {
        let gl = &self.gl_fns;
        gl.DeleteProgram(self.program);

        // Delete buffers
        for drawable in &self.drawables {
            match drawable {
                Drawable::Indexed(indexed_elements) => {
                    gl.DeleteBuffers(1, &indexed_elements.ebo);
                    gl.DeleteBuffers(1, &indexed_elements.vbo);
                    gl.DeleteBuffers(1, &indexed_elements.vao);
                }
                Drawable::Array(array) => {
                    gl.DeleteBuffers(1, &array.vbo);
                    gl.DeleteBuffers(1, &array.vao);
                }
            }
        }

        // Delete Texture
        if let Some(t) = &self.tex {
            gl.DeleteTextures(1, &t.tex);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.delete_gl() };
    }
}

pub trait GlslPass {
    // Create GPU resources; should be idempotent or guarded by internal state.
    fn init(
        &mut self,
        gl_fns: Rc<Gles2>,
        mat3d: Mat3DUpdate,
        initial_uniforms: &[Box<dyn Uniform>],
    );

    // Per-frame updates (uniforms, buffers, animations). Caller ensures active shader
    fn update(&mut self, mat3d: Mat3DUpdate, to_set_uniforms: &[Box<dyn Uniform>]);

    /// Issue draw calls. Caller ensures active shader
    /// # Safety
    /// FFI calls
    unsafe fn draw(&self) {
        let Some(glsl_pass) = self.get_shader() else {
            log::warn!("Tried to render /TODO: name/ before init");
            return;
        };
        let gl = &glsl_pass.gl_fns;

        if let Some(tex) = &glsl_pass.tex {
            gl.BindTexture(tex.target, tex.tex);
        }

        for drawable in &glsl_pass.drawables {
            match drawable {
                Drawable::Indexed(indexed_elements) => {
                    gl.BindVertexArray(indexed_elements.vao);
                    gl.BindBuffer(gl::ARRAY_BUFFER, indexed_elements.vbo);
                    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexed_elements.ebo);
                    gl.DrawElements(
                        gl::TRIANGLES,
                        indexed_elements.index_count as i32,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                }
                Drawable::Array(array) => {
                    gl.BindVertexArray(array.vao);
                    gl.BindBuffer(gl::ARRAY_BUFFER, array.vbo);
                    for i in 0..array.len {
                        gl.DrawArrays(
                            gl::TRIANGLE_STRIP,
                            (i * array.offset) as i32,
                            array.count as i32,
                        );
                    }
                }
            }
        }
    }

    // gl FFI getter
    fn get_shader(&self) -> Option<&Shader>;

    fn update_draw(&mut self, mat3d: Mat3DUpdate, to_set_uniforms: &[Box<dyn Uniform>]) {
        let Some(shader) = self.get_shader() else {
            log::warn!("Called update_draw on unitialized GlslPass");
            return;
        };
        unsafe { shader.use_program() }
        self.update(mat3d, to_set_uniforms);
        unsafe { self.draw() };
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
