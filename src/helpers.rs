use std::time::{Duration, Instant};

use winit::keyboard::KeyCode;

use crate::gl::{self, Gles2};

pub type GlPosition = glam::Vec3;

pub type GlColor = glam::Vec4;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Mat3DUpdate {
    pub model: Option<glam::Mat4>,
    pub view: Option<glam::Mat4>,
    pub projection: Option<glam::Mat4>,
}

impl Mat3DUpdate {
    pub fn default_from_dimensions(dimensions: &glam::Vec2) -> Self {
        let ar = dimensions.x / dimensions.y;

        Self {
            // model: glam::Mat4::from_rotation_x(15.0f32.to_radians()),
            model: Some(glam::Mat4::IDENTITY),
            view: Some(glam::Mat4::from_translation(glam::Vec3::new(
                0.0f32, 0.0f32, -1.5f32,
            ))),
            projection: Some(glam::Mat4::perspective_rh_gl(
                90.0f32.to_radians(),
                ar,
                0.1f32,
                100.0f32,
            )),
        }
    }

    // As init transforms the Update semantic into Initialize semantic
    pub fn as_init(self) -> Self {
        Self {
            model: Some(self.model.unwrap_or(glam::Mat4::IDENTITY)),
            view: Some(self.view.unwrap_or(glam::Mat4::IDENTITY)),
            projection: Some(self.projection.unwrap_or(glam::Mat4::IDENTITY)),
        }
    }

    /// # Context
    /// UseProgram(shader_program) must have being called before
    /// # Safety
    /// Calling ffi
    pub unsafe fn set_uniforms(&self, gl: &Gles2, shader_program: u32) {
        if let Some(model) = self.model {
            let model_loc = gl.GetUniformLocation(shader_program, c"model".as_ptr() as *const _);
            gl.UniformMatrix4fv(model_loc, 1, gl::FALSE, model.to_cols_array().as_ptr());
        }

        if let Some(view) = self.view {
            let view_loc = gl.GetUniformLocation(shader_program, c"view".as_ptr() as *const _);
            gl.UniformMatrix4fv(view_loc, 1, gl::FALSE, view.to_cols_array().as_ptr());
        }

        if let Some(projection) = self.projection {
            let proj_loc =
                gl.GetUniformLocation(shader_program, c"projection".as_ptr() as *const _);
            gl.UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());
        }
    }

    pub fn has_some(&self) -> bool {
        self.model.is_some() || self.projection.is_some() || self.view.is_some()
    }
}

pub struct FpsCounter {
    last: Instant,
    acc: Duration,
    frames: u32,
}

impl FpsCounter {
    pub fn tick(&mut self) -> Option<f64> {
        let now = Instant::now();
        let dt = now - self.last;
        self.last = now;
        self.acc += dt;
        self.frames += 1;
        if self.acc >= Duration::from_secs(1) {
            let fps = self.frames as f64 / self.acc.as_secs_f64();
            self.acc = Duration::ZERO;
            self.frames = 0;
            Some(fps)
        } else {
            None
        }
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            last: Instant::now(),
            acc: Duration::ZERO,
            frames: 0,
        }
    }
}

pub enum RendererControl {
    EnableLight,
    DisableLight,
    EnableFog,
    DisableFog,
}

impl RendererControl {
    pub fn from_keycode(value: KeyCode) -> Option<Self> {
        match value {
            KeyCode::KeyL => Some(Self::EnableLight),
            KeyCode::KeyO => Some(Self::DisableLight),
            KeyCode::KeyF => Some(Self::EnableFog),
            KeyCode::KeyC => Some(Self::DisableFog),
            _ => None,
        }
    }
}
