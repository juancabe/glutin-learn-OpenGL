use std::sync::Arc;

use crate::gl::{self, Gles2};

pub type GlPosition = glam::Vec3;

pub type GlColor = glam::Vec4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3D {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

impl Mat3D {
    pub fn default_from_dimensions(dimensions: &glam::Vec2) -> Self {
        let ar = dimensions.x / dimensions.y;

        Mat3D {
            model: glam::Mat4::from_rotation_x(15.0f32.to_radians())
                * glam::Mat4::from_rotation_y(15.0f32.to_radians()),
            view: glam::Mat4::from_translation(glam::Vec3::new(0.0f32, 0.0f32, -2.0f32)),
            projection: glam::Mat4::perspective_rh_gl(45.0f32.to_radians(), ar, 0.1f32, 100.0f32),
        }
    }

    /// # Context
    /// UseProgram(shader_program) must have being called before
    /// # Safety
    /// Calling ffi
    pub unsafe fn set_uniforms(&self, gl: &Gles2, shader_program: u32) {
        let model_loc = gl.GetUniformLocation(shader_program, c"model".as_ptr() as *const _);
        gl.UniformMatrix4fv(model_loc, 1, gl::FALSE, self.model.to_cols_array().as_ptr());

        let view_loc = gl.GetUniformLocation(shader_program, c"view".as_ptr() as *const _);
        gl.UniformMatrix4fv(view_loc, 1, gl::FALSE, self.view.to_cols_array().as_ptr());
    }

    /// # Context
    /// UseProgram(shader_program) must have being called before
    /// # Use case
    /// "the projection matrix rarely changes" [LearnOpenGL](https://learnopengl.com/code_viewer_gh.php?code=src/1.getting_started/6.1.coordinate_systems/coordinate_systems.cpp)
    /// # Safety
    /// Calling ffi
    pub unsafe fn set_uniforms_with_projection(&self, gl: &Gles2, shader_program: u32) {
        self.set_uniforms(gl, shader_program);
        let projection_loc =
            gl.GetUniformLocation(shader_program, c"projection".as_ptr() as *const _);
        gl.UniformMatrix4fv(
            projection_loc,
            1,
            gl::FALSE,
            self.projection.to_cols_array().as_ptr(),
        );
    }
}

#[derive(Clone)]
pub struct Shader {
    pub program: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub tex: Option<gl::types::GLuint>,
    pub mat3d: Option<Mat3D>,
    pub gl_fns: Arc<Gles2>,
}
