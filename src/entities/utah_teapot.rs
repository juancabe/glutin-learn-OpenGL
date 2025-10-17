use tobj::LoadOptions;

use crate::{
    entities::Entity,
    gl,
    helpers::GlPosition,
    renderer::shader::{Drawable, GlslPass, IndexedElements, Shader, create_shader},
};

#[derive(Clone)]
pub struct UtahTeapot {
    position: GlPosition,
    shader: Option<Shader>,
}

impl UtahTeapot {
    pub fn new(position: GlPosition) -> Self {
        Self {
            position,
            shader: None,
        }
    }
}

impl GlslPass for UtahTeapot {
    fn init(
        &mut self,
        gl_fns: std::sync::Arc<crate::gl::Gles2>,
        mut mat3d: crate::helpers::Mat3DUpdate,
    ) {
        let lo = LoadOptions {
            triangulate: true,
            ..Default::default()
        };

        let (models, materials) =
            tobj::load_obj("./assets/teapot.obj", &lo).expect("The asset should be available");
        let materials = materials.unwrap_or_else(|_| Default::default());

        log::info!("Number of models          = {}", models.len());
        log::info!("Number of materials       = {}", materials.len());

        for model in &models {
            log::info!(
                "Model name: {}; Vertices: {}",
                model.name,
                model.mesh.indices.len()
            );
        }

        let program;

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
        }

        let mut drawables = vec![];

        for model in &models {
            let vertex_data = &model.mesh.positions;
            let indices = &model.mesh.indices;
            let mut ebo;
            let mut vbo;
            let mut vao;

            unsafe {
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

                // indices
                ebo = std::mem::zeroed();
                gl_fns.GenBuffers(1, &mut ebo);
                gl_fns.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                gl_fns.BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                    indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                // attribute descriptor for this VBO
                let pos_attrib =
                    gl_fns.GetAttribLocation(program, c"position".as_ptr() as *const _);
                gl_fns.VertexAttribPointer(
                    pos_attrib as gl::types::GLuint,
                    3,
                    gl::FLOAT,
                    0,
                    3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    std::ptr::null(),
                );

                gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            }

            drawables.push(Drawable::Indexed(IndexedElements {
                ebo,
                vbo,
                vao,
                index_count: indices.len(),
            }));
        }

        mat3d.model = Some(
            glam::Mat4::from_scale(glam::Vec3::splat(0.5))
                * glam::Mat4::from_translation(self.position),
        );

        unsafe {
            mat3d.set_uniforms(&gl_fns, program);
        }

        self.shader = Some(Shader {
            program,
            model_transform: mat3d
                .model
                .expect("mat3d as init should at least be IDENTITY"),
            drawables,
            gl_fns,
            tex: Default::default(),
        })
    }

    fn update(&mut self, mat3d: crate::helpers::Mat3DUpdate) {
        if let Some(shader) = &self.shader {
            unsafe {
                mat3d.set_uniforms(&shader.gl_fns, shader.program);
            }
        }
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }
}

impl Entity for UtahTeapot {}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout(location = 0) in vec3 position;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

layout(location = 0) out vec4 FragColor;

void main() {
    FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
\0";
