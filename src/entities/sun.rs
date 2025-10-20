use std::{rc::Rc, time::Instant};

use glam::Vec3;

use crate::{
    entities::{
        Entity,
        tex_square::{Square, TexSquare},
    },
    helpers::{GlPosition, Mat3DUpdate},
    renderer::shader::{GlslPass, Shader},
};

pub struct Sun {
    square: TexSquare,
    initial_pos: GlPosition,
    actual_pos: GlPosition,
    init: Instant,
}

const SIDE_LEN: f32 = 3.0;
const ORBIT_R: f32 = 10.0;
const ORBIT_T_S: f32 = 10.0;

impl Sun {
    pub fn new(position: GlPosition) -> Self {
        Sun {
            square: TexSquare::new(
                vec![Square {
                    bottom_left: GlPosition::new(
                        position.x - SIDE_LEN,
                        position.y,
                        position.z - SIDE_LEN,
                    ),
                    top_right: GlPosition::new(
                        position.x + SIDE_LEN,
                        position.y,
                        position.z + SIDE_LEN,
                    ),
                }],
                Some("./assets/sun.png".into()),
            ),
            initial_pos: position,
            actual_pos: position,
            init: Instant::now(),
        }
    }

    pub fn get_pos(&self) -> GlPosition {
        self.actual_pos
    }
}

impl GlslPass for Sun {
    fn init(&mut self, gl_fns: Rc<crate::gl::Gles2>, mat3d: crate::helpers::Mat3DUpdate) {
        self.square.init(gl_fns.clone(), mat3d);
    }

    fn update(&mut self, mat3d: crate::helpers::Mat3DUpdate, light_pos: Option<glam::Vec3>) {
        let elapsed_s_f32 = self.init.elapsed().as_secs_f32();
        let next_spin_stop = f32::ceil(elapsed_s_f32 / ORBIT_T_S) * ORBIT_T_S;
        let perc_of_rotation = (next_spin_stop - elapsed_s_f32) / ORBIT_T_S;
        let rotation = 360.0 * perc_of_rotation;
        let rotation = rotation.to_radians();
        let dx = rotation.cos() * ORBIT_R;
        let dz = rotation.sin() * ORBIT_R;

        let model = Some(glam::Mat4::from_translation(Vec3::new(dx, 0.0, dz)));

        self.actual_pos = GlPosition {
            x: self.initial_pos.x + dx,
            z: self.initial_pos.z + dz,
            ..self.actual_pos
        };

        self.square
            .update(Mat3DUpdate { model, ..mat3d }, light_pos);
    }

    unsafe fn draw(&self) {
        self.square.draw();
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.square.get_shader()
    }
}

impl Entity for Sun {}
