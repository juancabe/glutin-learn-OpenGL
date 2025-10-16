use std::time::Instant;

use crate::{
    helpers::{GlPosition, Shader},
    renderer::shader::GlslPass,
};

#[derive(Clone)]
pub struct UtahTeapot {
    instances: Vec<GlPosition>,
    last_second: u64,
    init: Instant,
    glsl_pass: Option<Shader>,
}

impl GlslPass for UtahTeapot {
    fn init(
        &mut self,
        gl_fns: std::sync::Arc<crate::gl::Gles2>,
        mat3d: crate::helpers::Mat3DUpdate,
    ) {
        todo!("Load obj");

        self.glsl_pass = Some(Shader {
            program: (),
            vao: (),
            vbo: (),
            tex: (),
            model_transform: (),
            gl_fns: (),
        });
    }

    fn update(&mut self, mat3d: crate::helpers::Mat3DUpdate) {
        todo!()
    }

    fn draw(&self) {
        todo!()
    }

    fn get_shader(&self) -> u32 {
        todo!()
    }
}
