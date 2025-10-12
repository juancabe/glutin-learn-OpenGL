use std::{
    ffi::CStr,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    gl::{self, Gles2},
    renderer::shader::GlslPass,
};

struct FpsCounter {
    last: Instant,
    acc: Duration,
    frames: u32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            last: Instant::now(),
            acc: Duration::ZERO,
            frames: 0,
        }
    }
    fn tick(&mut self) -> Option<f64> {
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

pub mod shader;

pub struct Renderer {
    window_dimensions: glam::USizeVec2,
    creation: Instant,
    gl: Arc<gl::Gl>,
    fps_counter: FpsCounter,
}

impl Renderer {
    pub fn new(gl_fns: Arc<Gles2>, window_dimensions: glam::USizeVec2) -> Self {
        if let Some(renderer) = get_gl_string(&gl_fns, gl::RENDERER) {
            log::info!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(&gl_fns, gl::VERSION) {
            log::info!("OpenGL Version {}", version.to_string_lossy());
        }

        if let Some(shaders_version) = get_gl_string(&gl_fns, gl::SHADING_LANGUAGE_VERSION) {
            log::info!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        unsafe { gl_fns.Enable(gl::DEPTH_TEST) }

        Self {
            window_dimensions,
            creation: Instant::now(),
            gl: gl_fns,
            fps_counter: FpsCounter::new(),
        }
    }

    pub fn draw<'a, I>(&mut self, objects: I)
    where
        I: Iterator<Item = &'a mut dyn GlslPass>,
    {
        let dt = self.creation.elapsed();

        if let Some(fps) = self.fps_counter.tick() {
            log::info!("FPS: {fps}");
        }

        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        for obj in objects {
            obj.update(Some(&dt), None);
            obj.draw();
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    pub fn get_window_dimensions(&self) -> glam::USizeVec2 {
        self.window_dimensions
    }
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

// #[rustfmt::skip]
// static VERTEX_DATA: [f32; 15] = [
//     -0.5, -0.5,  1.0,  0.0,  0.0,
//      0.0,  0.5,  0.0,  1.0,  0.0,
//      0.5, -0.5,  0.0,  0.0,  1.0,
// ];
