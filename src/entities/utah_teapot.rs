use std::rc::Rc;

use glam::Vec3;
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
    color: Vec3,
    shader: Option<Shader>,
}

impl UtahTeapot {
    pub fn new(position: GlPosition, color: Vec3) -> Self {
        Self {
            color,
            position,
            shader: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

impl GlslPass for UtahTeapot {
    fn init(&mut self, gl_fns: Rc<crate::gl::Gles2>, mut mat3d: crate::helpers::Mat3DUpdate) {
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
                "Model name: {}; Indices: {}, Positions: {}, Normals: {}, N_Indices: {}, N_Indices_max: {:?}",
                model.name,
                model.mesh.indices.len(),
                model.mesh.positions.len(),
                model.mesh.normals.len(),
                model.mesh.normal_indices.len(),
                model.mesh.normal_indices.iter().max(),
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
            let positions = &model.mesh.positions;
            let position_i = &model.mesh.indices;
            let normals = &model.mesh.normals;
            let normal_i = &model.mesh.normal_indices;
            let mut ebo;
            let mut vbo;
            let mut vao;

            let vertex_data: Vec<Vertex> = position_i
                .iter()
                .zip(normal_i)
                .map(|(p_i, n_i)| (*p_i as usize, *n_i as usize))
                .map(|(p_i, n_i)| (p_i * 3, n_i * 3))
                .map(|(p_i, n_i)| Vertex {
                    position: Vec3::from_slice(&positions[p_i..p_i + 3]),
                    normal: Vec3::from_slice(&normals[n_i..n_i + 3]),
                })
                .collect();
            let final_indices: Vec<u32> = (0..vertex_data.len() as u32).collect();

            unsafe {
                vao = std::mem::zeroed();
                gl_fns.GenVertexArrays(1, &mut vao);
                gl_fns.BindVertexArray(vao);

                vbo = std::mem::zeroed();
                gl_fns.GenBuffers(1, &mut vbo);
                gl_fns.BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl_fns.BufferData(
                    gl::ARRAY_BUFFER,
                    (vertex_data.len() * 6 * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    vertex_data.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                // indices
                ebo = std::mem::zeroed();
                gl_fns.GenBuffers(1, &mut ebo);
                gl_fns.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                gl_fns.BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (final_indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                    final_indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                let pos_attrib =
                    gl_fns.GetAttribLocation(program, c"position".as_ptr() as *const _);
                assert_ne!(pos_attrib, -1);
                gl_fns.VertexAttribPointer(
                    pos_attrib as gl::types::GLuint,
                    3,
                    gl::FLOAT,
                    0,
                    6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    std::ptr::null(),
                );

                let norm_attrib = gl_fns.GetAttribLocation(program, c"normal".as_ptr() as *const _);
                assert_ne!(norm_attrib, -1);
                gl_fns.VertexAttribPointer(
                    norm_attrib as gl::types::GLuint,
                    3,
                    gl::FLOAT,
                    0,
                    6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    (3 * std::mem::size_of::<f32>()) as *const _,
                );

                gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
                gl_fns.EnableVertexAttribArray(norm_attrib as gl::types::GLuint);
            }

            drawables.push(Drawable::Indexed(IndexedElements {
                ebo,
                vbo,
                vao,
                index_count: position_i.len(),
            }));
        }

        mat3d.model = Some(
            glam::Mat4::from_scale(glam::Vec3::splat(0.5))
                * glam::Mat4::from_translation(self.position),
        );

        unsafe {
            mat3d.set_uniforms(&gl_fns, program);
        }

        // Color uniform
        unsafe {
            let color_loc = gl_fns.GetUniformLocation(program, c"uColor".as_ptr() as *const _);
            gl_fns.Uniform3f(color_loc, self.color.x, self.color.y, self.color.z);
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

    fn update(
        &mut self,
        mat3d: crate::helpers::Mat3DUpdate,
        light_pos: Option<Vec3>,
        eye_pos: Option<Vec3>,
    ) {
        if let Some(shader) = &self.shader {
            unsafe {
                mat3d.set_uniforms(&shader.gl_fns, shader.program);
            }
            // Light position uniform
            if let Some(light_pos) = light_pos {
                unsafe {
                    let light_pos_loc = shader
                        .gl_fns
                        .GetUniformLocation(shader.program, c"uLightPos".as_ptr() as *const _);
                    shader
                        .gl_fns
                        .Uniform3f(light_pos_loc, light_pos.x, light_pos.y, light_pos.z);
                }
            }

            if let Some(eye_pos) = eye_pos {
                unsafe {
                    let eye_pos_loc = shader
                        .gl_fns
                        .GetUniformLocation(shader.program, c"uEyePos".as_ptr() as *const _);
                    shader
                        .gl_fns
                        .Uniform3f(eye_pos_loc, eye_pos.x, eye_pos.y, eye_pos.z);
                }
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
layout(location = 1) in vec3 normal;

out vec3 fragNorm;
out vec3 fragPos;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    fragPos = vec3(model * vec4(position, 1.0));
    // Use the upper 3x3 of the model matrix for rotation/scaling
    fragNorm = mat3(transpose(inverse(model))) * normal;  
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

uniform vec3 uColor;
uniform vec3 uLightPos;
uniform vec3 viewPos;

layout(location = 0) out vec4 FragColor;

in vec3 fragNorm;
in vec3 fragPos;

void main() {
    float specularStrength = 0.5;

    vec3 norm = normalize(fragNorm);

    vec3 lightDir = normalize(uLightPos - fragPos);
    float diffuse = max(dot(norm, lightDir), 0.0);

    vec3 viewDir = normalize(viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
    vec3 specular = specularStrength * spec * vec3(1.0, 1.0, 1.0);

    float ambientStrenght = 0.1;

    vec3 result = uColor * (ambientStrenght + diffuse + specular);
    FragColor = vec4(result, 1.0);
}
\0";
