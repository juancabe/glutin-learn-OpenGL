use std::{path::PathBuf, rc::Rc};

use crate::{
    entities::{
        Entity,
        tex_square::{Square, TexSquare},
    },
    helpers::GlPosition,
    renderer::shader::{GlslPass, Shader, uniform::Uniform},
};

fn build_faces(pos: &GlPosition, side_len: f32) -> [Square; 6] {
    let half_side = side_len / 2.0;
    [
        // Front face
        Square {
            bottom_left: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z + half_side),
            top_right: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z + half_side),
        },
        // Back face
        Square {
            bottom_left: GlPosition::new(pos.x + half_side, pos.y - half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x - half_side, pos.y + half_side, pos.z - half_side),
        },
        // Right face
        Square {
            bottom_left: GlPosition::new(pos.x + half_side, pos.y - half_side, pos.z + half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z - half_side),
        },
        // Left face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x - half_side, pos.y + half_side, pos.z + half_side),
        },
        // Top face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y + half_side, pos.z + half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z - half_side),
        },
        // Bottom face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y - half_side, pos.z + half_side),
        },
    ]
}

pub struct TexCube {
    squares: TexSquare,
}

impl TexCube {
    pub fn new(positions: Vec<GlPosition>, side_len: f32, tex: Option<PathBuf>) -> Self {
        TexCube {
            squares: TexSquare::new(
                positions
                    .into_iter()
                    .flat_map(|p| build_faces(&p, side_len))
                    .collect(),
                tex,
            ),
        }
    }
}

impl GlslPass for TexCube {
    fn init(
        &mut self,
        gl_fns: Rc<crate::gl::Gles2>,
        mat3d: crate::helpers::Mat3DUpdate,
        init_uniforms: &[Box<dyn Uniform>],
    ) {
        self.squares.init(gl_fns.clone(), mat3d, init_uniforms);
    }

    fn update(&mut self, mat3d: crate::helpers::Mat3DUpdate, to_set_uniforms: &[Box<dyn Uniform>]) {
        self.squares.update(mat3d, to_set_uniforms);
    }

    unsafe fn draw(&self) {
        self.squares.draw();
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.squares.get_shader()
    }
}

impl Entity for TexCube {}
