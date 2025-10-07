use std::sync::Arc;

use crate::gl::{self, Gles2};

#[derive(Clone, Copy, Debug)]
pub struct GlPosition {
    pub x: gl::types::GLfloat,
    pub y: gl::types::GLfloat,
    pub z: gl::types::GLfloat,
    pub w: gl::types::GLfloat,
}

impl GlPosition {
    pub fn new_2d(x: gl::types::GLfloat, y: gl::types::GLfloat) -> GlPosition {
        GlPosition {
            x,
            y,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn sub(&self, other: &GlPosition) -> GlPosition {
        GlPosition {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }

    pub fn add(&self, other: &GlPosition) -> GlPosition {
        GlPosition {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GlColor {
    pub r: gl::types::GLfloat,
    pub g: gl::types::GLfloat,
    pub b: gl::types::GLfloat,
}

#[derive(Clone)]
pub struct Shader {
    pub program: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub tex: Option<gl::types::GLuint>,
    pub gl_fns: Arc<Gles2>,
}
