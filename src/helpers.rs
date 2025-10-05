use crate::gl;

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
}

#[derive(Clone, Copy, Debug)]
pub struct GlColor {
    pub r: gl::types::GLfloat,
    pub g: gl::types::GLfloat,
    pub b: gl::types::GLfloat,
}
