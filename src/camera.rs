use glam::Vec3;
use winit::keyboard::KeyCode;

const VEL: f32 = 0.10;

pub enum CameraMovementXZ {
    Front,
    Back,
    Left,
    Right,
}

impl CameraMovementXZ {
    pub fn from_keycode(value: KeyCode) -> Option<Self> {
        log::info!("Decoding KeyCode: {value:?}");
        match value {
            KeyCode::KeyW => Some(Self::Front),
            KeyCode::KeyA => Some(Self::Left),
            KeyCode::KeyS => Some(Self::Back),
            KeyCode::KeyD => Some(Self::Right),
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
}

impl MovementState {
    // Check that we are moving in some direction
    pub fn moves(&self) -> bool {
        (self.forward ^ self.back) || (self.right ^ self.left)
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

        t.normalize()
    }

    pub fn want_move(&mut self, movement: CameraMovementXZ) {
        match movement {
            CameraMovementXZ::Front => self.forward = true,
            CameraMovementXZ::Back => self.back = true,
            CameraMovementXZ::Left => self.left = true,
            CameraMovementXZ::Right => self.right = true,
        }
    }

    pub fn stop_move(&mut self, movement: CameraMovementXZ) {
        match movement {
            CameraMovementXZ::Front => self.forward = false,
            CameraMovementXZ::Back => self.back = false,
            CameraMovementXZ::Left => self.left = false,
            CameraMovementXZ::Right => self.right = false,
        }
    }
}

pub struct Camera {
    pub pos: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub movement_state: MovementState,
}

impl Camera {
    pub fn as_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.pos, self.pos + self.front, self.up)
    }

    pub fn want_move(&mut self, movement: CameraMovementXZ) {
        self.movement_state.want_move(movement);
    }

    pub fn stop_move(&mut self, movement: CameraMovementXZ) {
        self.movement_state.stop_move(movement);
    }

    pub fn update(&mut self) {
        let movement_dir = self.movement_state.as_direction(self.front, self.up) * VEL;
        self.pos += movement_dir;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            front: Vec3::with_z(Default::default(), -1.0),
            up: Vec3::with_y(Default::default(), 1.0),
            movement_state: Default::default(),
        }
    }
}
