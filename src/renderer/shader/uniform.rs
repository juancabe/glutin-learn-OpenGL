use crate::gl::Gles2;

pub type ShaderProgram = u32;

pub trait Uniform {
    fn set(&self, gl: &Gles2, program: ShaderProgram);
}

pub struct Fog {
    pub fog_near: f32,
    pub fog_far: f32,
    pub fog_color: glam::Vec3,
}

impl Fog {
    pub fn new(clear_color: glam::Vec3) -> Self {
        Self {
            fog_near: 1.0,
            fog_far: 50.0,
            fog_color: clear_color,
        }
    }
    /// # Safety
    /// Unsafe bacuse we are calling ffi!
    pub unsafe fn set_uniforms(&self, gl_fns: &Gles2, program: u32) {
        // Fog uniforms
        let fog_near_loc = gl_fns.GetUniformLocation(program, c"uFogNear".as_ptr() as *const _);
        gl_fns.Uniform1f(fog_near_loc, self.fog_near);

        let fog_far_loc = gl_fns.GetUniformLocation(program, c"uFogFar".as_ptr() as *const _);
        gl_fns.Uniform1f(fog_far_loc, self.fog_far);

        let fog_color_loc = gl_fns.GetUniformLocation(program, c"uFogColor".as_ptr() as *const _);
        gl_fns.Uniform3f(
            fog_color_loc,
            self.fog_color.x,
            self.fog_color.y,
            self.fog_color.z,
        );
    }
}

impl Uniform for Fog {
    fn set(&self, gl: &Gles2, program: ShaderProgram) {
        unsafe { self.set_uniforms(gl, program) }
    }
}

pub struct Lighting {
    pub ambient_strenght: f32,
    pub specular_strenght: f32,
}

impl Lighting {
    /// # Safety
    /// Unsafe bacuse we are calling ffi!
    pub unsafe fn set_uniforms(&self, gl_fns: &Gles2, program: u32) {
        // Ligthing uniforms
        let ambient_loc =
            gl_fns.GetUniformLocation(program, c"uAmbientStrenght".as_ptr() as *const _);
        gl_fns.Uniform1f(ambient_loc, self.ambient_strenght);

        let specular_loc =
            gl_fns.GetUniformLocation(program, c"uSpecularStrength".as_ptr() as *const _);
        gl_fns.Uniform1f(specular_loc, self.specular_strenght);
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            ambient_strenght: 0.1,
            specular_strenght: 0.5,
        }
    }
}

impl Uniform for Lighting {
    fn set(&self, gl: &Gles2, program: ShaderProgram) {
        unsafe {
            self.set_uniforms(gl, program);
        }
    }
}

pub struct LightPos {
    pub pos: glam::Vec3,
}

impl LightPos {
    /// # Safety
    /// Unsafe bacuse we are calling ffi!
    pub unsafe fn set_uniforms(&self, gl_fns: &Gles2, program: u32) {
        // Ligthing pos uniform
        let ambient_loc = gl_fns.GetUniformLocation(program, c"uLightPos".as_ptr() as *const _);
        gl_fns.Uniform3f(ambient_loc, self.pos.x, self.pos.y, self.pos.z);
    }

    #[allow(clippy::new_without_default)]
    pub fn new(pos: glam::Vec3) -> Self {
        Self { pos }
    }
}

impl Uniform for LightPos {
    fn set(&self, gl: &Gles2, program: ShaderProgram) {
        unsafe {
            self.set_uniforms(gl, program);
        }
    }
}

pub struct EyePos {
    pub pos: glam::Vec3,
}

impl EyePos {
    /// # Safety
    /// Unsafe bacuse we are calling ffi!
    pub unsafe fn set_uniforms(&self, gl_fns: &Gles2, program: u32) {
        // Eye pos uniform
        let ambient_loc = gl_fns.GetUniformLocation(program, c"uEyePos".as_ptr() as *const _);
        gl_fns.Uniform3f(ambient_loc, self.pos.x, self.pos.y, self.pos.z);
    }

    #[allow(clippy::new_without_default)]
    pub fn new(pos: glam::Vec3) -> Self {
        Self { pos }
    }
}

impl Uniform for EyePos {
    fn set(&self, gl: &Gles2, program: ShaderProgram) {
        unsafe {
            self.set_uniforms(gl, program);
        }
    }
}
