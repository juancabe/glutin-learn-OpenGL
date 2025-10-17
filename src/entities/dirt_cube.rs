use std::rc::Rc;

use crate::{
    entities::{
        Entity,
        dirt_square::{DirtSquare, Square},
    },
    helpers::GlPosition,
    renderer::shader::{GlslPass, Shader},
};

fn build_faces(pos: &GlPosition, side_len: f32) -> [Square; 6] {
    let half_side = side_len / 2.0;
    [
        // Front face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z + half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z + half_side),
        },
        // Back face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z - half_side),
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
            bottom_left: GlPosition::new(pos.x - half_side, pos.y + half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y + half_side, pos.z + half_side),
        },
        // Bottom face
        Square {
            bottom_left: GlPosition::new(pos.x - half_side, pos.y - half_side, pos.z - half_side),
            top_right: GlPosition::new(pos.x + half_side, pos.y - half_side, pos.z + half_side),
        },
    ]
}

pub struct DirtCube {
    squares: DirtSquare,
}

impl DirtCube {
    pub fn new(positions: Vec<GlPosition>, side_len: f32) -> Self {
        DirtCube {
            squares: DirtSquare::new(
                positions
                    .into_iter()
                    .flat_map(|p| build_faces(&p, side_len))
                    .collect(),
            ),
        }
    }
}

impl GlslPass for DirtCube {
    fn init(&mut self, gl_fns: Rc<crate::gl::Gles2>, mat3d: crate::helpers::Mat3DUpdate) {
        self.squares.init(gl_fns.clone(), mat3d);
    }

    fn update(&mut self, mat3d: crate::helpers::Mat3DUpdate) {
        self.squares.update(mat3d);
    }

    unsafe fn draw(&self) {
        self.squares.draw();
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.squares.get_shader()
    }
}

impl Entity for DirtCube {}
