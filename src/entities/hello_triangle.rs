use std::{sync::Arc, time::Duration};

use crate::{
    entities::Entity,
    gl::{self, Gles2},
    helpers::{GlColor, GlPosition, Shader},
    renderer::shader::{GlslPass, create_shader},
};

type TriangleVertex = (GlPosition, GlColor);
#[derive(Clone)]
pub struct HelloTriangle {
    instances: Vec<[TriangleVertex; 3]>,
    last_second: u64,
    glsl_pass: Option<Shader>,
}

impl HelloTriangle {
    pub fn new(instances: Vec<[TriangleVertex; 3]>) -> HelloTriangle {
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
            .flat_map(|(p, c)| [p.x, p.y, c.r, c.g, c.b])
            .collect();

        if let Some(Shader {
            program: _,
            vao: _,
            gl_fns,
            vbo,
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

    fn rotate_left(&mut self) {
        for vert in self.instances.iter_mut() {
            let mut colors = vert.map(|(_, c)| c);
            colors.rotate_left(1);
            vert[0].1 = colors[0];
            vert[1].1 = colors[1];
            vert[2].1 = colors[2];
        }

        self.apply_v_change_to_gpu();
    }

    fn draw_with_clear_color(
        &self,
        glsl_data: &Shader,
        red: gl::types::GLfloat,
        green: gl::types::GLfloat,
        blue: gl::types::GLfloat,
        alpha: gl::types::GLfloat,
    ) {
        let gl = &glsl_data.gl_fns;
        let program = glsl_data.program;
        let vao = glsl_data.vao;
        let vbo = glsl_data.vbo;
        unsafe {
            glsl_data.gl_fns.UseProgram(program);

            gl.BindVertexArray(vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl.ClearColor(red, green, blue, alpha);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl GlslPass for HelloTriangle {
    fn init(&mut self, gl_fns: Arc<Gles2>) {
        let program;
        let mut vao;
        let mut vbo;

        let vertex_data: Vec<f32> = self
            .instances
            .clone()
            .concat()
            .into_iter()
            .flat_map(|(p, c)| [p.x, p.y, c.r, c.g, c.b])
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

            let pos_attrib = gl_fns.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl_fns.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl_fns.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
        }

        self.glsl_pass = Some(Shader {
            program,
            vao,
            vbo,
            tex: None,
            gl_fns,
        })
    }

    fn draw(&self) {
        if let Some(glsl_pass) = &self.glsl_pass {
            self.draw_with_clear_color(glsl_pass, 0.1, 0.1, 0.1, 0.9);
        } else {
            log::warn!("Tried to draw HelloTriangle before even initializing it")
        }
    }

    fn update(&mut self, dt: &Duration) {
        log::debug!("Updating triangles");
        if dt.as_secs() > self.last_second {
            self.last_second = dt.as_secs();
            self.rotate_left();
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

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

out vec3 v_color;  // goes to the fragment shader

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
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
