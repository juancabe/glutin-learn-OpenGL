use std::{rc::Rc, time::Instant};

use crate::{
    entities::Entity,
    gl::{self, Gles2},
    helpers::{GlColor, GlPosition, Mat3DUpdate},
    renderer::shader::{Array, Drawable, GlslPass, Shader, create_shader},
};

#[derive(Clone)]
pub struct HelloTriangle {
    instances: Vec<[(GlPosition, GlColor); 3]>,
    last_second: u64,
    init: Instant,
    shader: Option<Shader>,
}

pub type Circumradius = f32;

/// NOTE: We currently only draw the first instance
impl HelloTriangle {
    pub fn new(instances: Vec<(GlPosition, Circumradius)>) -> HelloTriangle {
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
            init: Instant::now(),
            shader: None,
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
            gl_fns, drawables, ..
        }) = &self.shader
        {
            let drawable = drawables.first().expect("It should be there");
            let Drawable::Array(array) = drawable else {
                panic!("The drawable in HelloTriangle mutated illegally");
            };

            unsafe {
                gl_fns.BindBuffer(gl::ARRAY_BUFFER, array.vbo);
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

impl GlslPass for HelloTriangle {
    fn init(&mut self, gl_fns: Rc<Gles2>, mat3d: Mat3DUpdate) {
        let program;
        let mut vao;
        let mut vbo;

        let mat3d = mat3d.as_init();

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

            mat3d.set_uniforms(&gl_fns, program);

            let pos_attrib = gl_fns.GetAttribLocation(program, c"position".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );

            let color_attrib = gl_fns.GetAttribLocation(program, c"color".as_ptr() as *const _);
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

        let drawable = Drawable::Array(Array {
            vao,
            vbo,
            len: self.instances.len(),
            offset: 3,
            count: 3,
        });

        let drawables = vec![drawable];

        self.shader = Some(Shader {
            program,
            model_transform: mat3d
                .model
                .expect("mat3d as init should be at least IDENTITY"),
            drawables,
            tex: Default::default(),
            gl_fns,
        })
    }

    // unsafe fn draw(&self) {
    //     if let Some(glsl_pass) = &self.glsl_pass {
    //         log::debug!(
    //             "Drawing HelloTriangle with model_transform: {:?}",
    //             glsl_pass.model_transform
    //         );
    //         let gl = &glsl_pass.gl_fns;
    //         unsafe {
    //             gl.BindVertexArray(glsl_pass.vao);
    //             gl.BindBuffer(gl::ARRAY_BUFFER, glsl_pass.vbo);
    //
    //             // gl.DrawArrays(gl::TRIANGLES, 0, 3);
    //
    //             // NOTE: This should be batched
    //             for i in (0..self.instances.len()).rev() {
    //                 let offset = i * 3;
    //                 gl.DrawArrays(gl::TRIANGLES, offset as i32, 3);
    //             }
    //         }
    //     } else {
    //         log::warn!("Tried to draw HelloTriangle before even initializing it")
    //     }
    // }

    fn update(&mut self, mut mat3d: Mat3DUpdate) {
        let elapsed = self.init.elapsed();
        if elapsed.as_secs() > self.last_second {
            self.last_second = elapsed.as_secs();
            self.rotate_vertex_colors_left();
            self.apply_v_change_to_gpu();
        }

        if let Some(shader) = &mut self.shader {
            if let Some(model) = mat3d.model {
                shader.model_transform = model;
            } else {
                let elapsed_s_f32 = self.init.elapsed().as_secs_f32();
                let spin_duration_s = 1.5;
                let next_spin_stop = f32::ceil(elapsed_s_f32 / spin_duration_s) * spin_duration_s;
                let perc_of_rotation = (next_spin_stop - elapsed_s_f32) / spin_duration_s;
                let rotation = 360.0 * perc_of_rotation;
                shader.model_transform = glam::Mat4::from_rotation_y((rotation).to_radians());
                mat3d.model = Some(shader.model_transform);
            }
            unsafe { mat3d.set_uniforms(&shader.gl_fns, shader.program) };
        }
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }
}

impl Entity for HelloTriangle {}

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
