use std::time::Duration;

use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::helpers::GlPosition;

const VEL: f32 = 0.01;
const SENS: f32 = 0.1;

pub enum CameraMovement {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down,
}

impl CameraMovement {
    pub fn from_keycode(value: KeyCode) -> Option<Self> {
        match value {
            KeyCode::KeyW => Some(Self::Front),
            KeyCode::KeyA => Some(Self::Left),
            KeyCode::KeyS => Some(Self::Back),
            KeyCode::KeyD => Some(Self::Right),
            KeyCode::Space => Some(Self::Up),
            KeyCode::ShiftLeft => Some(Self::Down),
            _ => None,
        }
    }
}

#[derive(Default, Debug)]
struct MovementState {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl MovementState {
    // Check that we are moving in some direction
    pub fn moves(&self) -> bool {
        (self.forward ^ self.back) || (self.right ^ self.left) || (self.up ^ self.down)
    }

    pub fn as_direction(&self, front: Vec3, up: Vec3) -> Vec3 {
        let mut t = Vec3::default();

        if !self.moves() {
            return t;
        };

        if self.forward {
            t += front
        }
        if self.back {
            t -= front
        }
        if self.left {
            t -= front.cross(up).normalize()
        }
        if self.right {
            t += front.cross(up).normalize()
        }
        if self.up {
            t += up
        }
        if self.down {
            t -= up
        }

        t.normalize()
    }

    pub fn want_move(&mut self, movement: CameraMovement) {
        match movement {
            CameraMovement::Front => self.forward = true,
            CameraMovement::Back => self.back = true,
            CameraMovement::Left => self.left = true,
            CameraMovement::Right => self.right = true,
            CameraMovement::Up => self.up = true,
            CameraMovement::Down => self.down = true,
        }
    }

    pub fn stop_move(&mut self, movement: CameraMovement) {
        match movement {
            CameraMovement::Front => self.forward = false,
            CameraMovement::Back => self.back = false,
            CameraMovement::Left => self.left = false,
            CameraMovement::Right => self.right = false,
            CameraMovement::Up => self.up = false,
            CameraMovement::Down => self.down = false,
        }
    }
}

pub struct Camera {
    pub pos: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    movement_state: MovementState,
}

impl Camera {
    pub fn from_pos(pos: GlPosition) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn as_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.pos, self.pos + self.front(), self.up)
    }

    pub fn want_move(&mut self, movement: CameraMovement) {
        self.movement_state.want_move(movement);
    }

    pub fn stop_move(&mut self, movement: CameraMovement) {
        self.movement_state.stop_move(movement);
    }

    pub fn update(&mut self, dt: &Duration) {
        let movement_dir =
            self.movement_state.as_direction(self.front(), self.up) * VEL * (dt.as_millis() as f32);
        self.pos += movement_dir;
    }

    pub fn front(&self) -> Vec3 {
        Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize()
    }

    pub fn mouse_moved(&mut self, dx: f32, dy: f32) {
        let dx = dx * SENS;
        let dy = dy * SENS;

        self.yaw += dx;
        self.pitch += dy;

        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            up: Vec3::with_y(Default::default(), 1.0),
            movement_state: Default::default(),
            yaw: -90.0,
            pitch: Default::default(),
        }
    }
}
