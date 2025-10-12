use std::{sync::Arc, time::Duration};

use crate::{
    entities::Entity,
    gl::{self, Gles2},
    helpers::{GlColor, GlPosition, Mat3D, Shader},
    renderer::shader::{GlslPass, create_shader},
};

#[derive(Clone)]
pub struct HelloTriangle {
    instances: Vec<[(GlPosition, GlColor); 3]>,
    last_second: u64,
    glsl_pass: Option<Shader>,
}

pub type Circumradius = f32;

/// NOTE: We currently only draw the first instance
impl HelloTriangle {
    pub fn new(instances: Vec<(glam::Vec3, Circumradius)>) -> HelloTriangle {
        let instances = instances
            .into_iter()
            .map(|(i, cr)| {
                [
                    (
                        GlPosition::new(-1.0f32, -1.0f32, 0.0f32).normalize() * cr + i,
                        GlColor::new(1.0f32, 0.0f32, 0.0f32, 1.0f32),
                    ),
                    (
                        GlPosition::new(0.0f32, 1.0f32, 0.0f32).normalize() * cr + i,
                        GlColor::new(0.0f32, 1.0f32, 0.0f32, 1.0f32),
                    ),
                    (
                        GlPosition::new(1.0f32, -1.0f32, 0.0f32).normalize() * cr + i,
                        GlColor::new(0.0f32, 0.0f32, 1.0f32, 1.0f32),
                    ),
                ]
            })
            .collect();

        Self {
            instances,
            last_second: 0,
            glsl_pass: None,
        }
    }

    fn apply_v_change_to_gpu(&self) {
        let vertex_data: Vec<f32> = self
            .instances
            .concat()
            .iter()
            .flat_map(|(p, c)| [p.x, p.y, p.z, c.x, c.y, c.z])
            .collect();

        if let Some(Shader {
            program: _,
            vao: _,
            gl_fns,
            vbo,
            mat3d: _,
            tex: _,
        }) = &self.glsl_pass
        {
            unsafe {
                gl_fns.BindBuffer(gl::ARRAY_BUFFER, *vbo);
                gl_fns.BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (vertex_data.len() * std::mem::size_of::<f32>()) as isize,
                    vertex_data.as_ptr() as *const _,
                );
            }
        } else {
            log::warn!("Called apply_v_change_to_gpu on HelloTriangle with uninitialized glsl_pass")
        }
    }

    fn rotate_vertex_colors_left(&mut self) {
        for vert in self.instances.iter_mut() {
            let mut colors = vert.map(|(_, c)| c);
            colors.rotate_left(1);
            vert[0].1 = colors[0];
            vert[1].1 = colors[1];
            vert[2].1 = colors[2];
        }
    }
}

enum DeferredUpdate {
    ApplyVChangeToGpu,
    SetUniformsWithProjection(Mat3D),
    SetUniforms(Mat3D),
}

impl GlslPass for HelloTriangle {
    fn init(&mut self, gl_fns: Arc<Gles2>, mat3d: Option<Mat3D>) {
        let program;
        let mut vao;
        let mut vbo;

        let vertex_data: Vec<f32> = self
            .instances
            .clone()
            .concat()
            .into_iter()
            .flat_map(|(p, c)| [p.x, p.y, p.z, c.x, c.y, c.z])
            .collect();

        unsafe {
            program = gl_fns.CreateProgram();
            gl_fns.UseProgram(program);

            let vertex_shader = create_shader(&gl_fns, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader =
                create_shader(&gl_fns, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            gl_fns.AttachShader(program, vertex_shader);
            gl_fns.AttachShader(program, fragment_shader);

            gl_fns.LinkProgram(program);

            gl_fns.DeleteShader(vertex_shader);
            gl_fns.DeleteShader(fragment_shader);

            gl_fns.UseProgram(program);

            vao = std::mem::zeroed();
            gl_fns.GenVertexArrays(1, &mut vao);
            gl_fns.BindVertexArray(vao);

            vbo = std::mem::zeroed();
            gl_fns.GenBuffers(1, &mut vbo);
            gl_fns.BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl_fns.BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if let Some(trans_uniforms) = mat3d {
                trans_uniforms.set_uniforms_with_projection(&gl_fns, program);
            }

            let pos_attrib = gl_fns.GetAttribLocation(program, c"position".as_ptr() as *const _);
            let color_attrib = gl_fns.GetAttribLocation(program, c"color".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl_fns.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
        }

        self.glsl_pass = Some(Shader {
            program,
            vao,
            vbo,
            tex: None,
            mat3d,
            gl_fns,
        })
    }

    fn draw(&self) {
        if let Some(glsl_pass) = &self.glsl_pass {
            log::debug!("Drawing HelloTriangle with mat3d: {:?}", glsl_pass.mat3d);
            let gl = &glsl_pass.gl_fns;
            let program = glsl_pass.program;
            unsafe {
                glsl_pass.gl_fns.UseProgram(program);

                gl.BindVertexArray(glsl_pass.vao);
                gl.BindBuffer(gl::ARRAY_BUFFER, glsl_pass.vbo);

                // gl.DrawArrays(gl::TRIANGLES, 0, 3);

                // NOTE: This should be batched
                for i in (0..self.instances.len()).rev() {
                    let offset = i * 3;
                    gl.DrawArrays(gl::TRIANGLES, offset as i32, 3);
                }
            }
        } else {
            log::warn!("Tried to draw HelloTriangle before even initializing it")
        }
    }

    fn update(&mut self, dt: Option<&Duration>, mat3d: Option<Mat3D>) {
        // NOTE: Getting rid of this alloc for vec doesn't improve FPS
        let mut defer_needs_use_program = vec![];

        if let Some(dt) = dt
            && dt.as_secs() > self.last_second
        {
            self.last_second = dt.as_secs();
            self.rotate_vertex_colors_left();

            defer_needs_use_program.push(DeferredUpdate::ApplyVChangeToGpu);
        }

        if let Some(shader) = &mut self.glsl_pass
            && let Some(new_mat) = mat3d
        {
            if let Some(old_mat) = shader.mat3d
                && old_mat.projection == new_mat.projection
            {
                log::debug!("SetUniforms...");
                defer_needs_use_program.push(DeferredUpdate::SetUniforms(new_mat));
            } else {
                log::debug!("SetUniformsWithProjection...");
                defer_needs_use_program.push(DeferredUpdate::SetUniformsWithProjection(new_mat));
            }

            shader.mat3d = mat3d;
        }

        if !defer_needs_use_program.is_empty()
            && let Some(gl) = &self.glsl_pass
        {
            unsafe { gl.gl_fns.UseProgram(gl.program) }
            let mut applied_vchange = false;
            for deferred in defer_needs_use_program {
                match deferred {
                    DeferredUpdate::ApplyVChangeToGpu => {
                        if !applied_vchange {
                            self.apply_v_change_to_gpu();
                            applied_vchange = true;
                        }
                    }
                    DeferredUpdate::SetUniforms(mat3_d) => unsafe {
                        mat3_d.set_uniforms(&gl.gl_fns, gl.program)
                    },
                    DeferredUpdate::SetUniformsWithProjection(mat3_d) => unsafe {
                        mat3_d.set_uniforms_with_projection(&gl.gl_fns, gl.program)
                    },
                }
            }
        }
    }
}

impl Entity for HelloTriangle {}

impl Drop for HelloTriangle {
    fn drop(&mut self) {
        if let Some(glsl_pass) = &self.glsl_pass {
            let gl_fns = &glsl_pass.gl_fns;
            let program = glsl_pass.program;
            let vao = glsl_pass.vao;
            let vbo = glsl_pass.vbo;
            unsafe {
                gl_fns.DeleteProgram(program);
                gl_fns.DeleteBuffers(1, &vbo);
                gl_fns.DeleteVertexArrays(1, &vao);
            }
        } else {
            log::warn!("Dropped HelloTriangle before even initializing it")
        }
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

out vec3 v_color;  // goes to the fragment shader

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    // gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

in vec3 v_color;

layout(location = 0) out vec4 FragColor;

void main() {
    FragColor = vec4(v_color, 1.0);
}
\0";
