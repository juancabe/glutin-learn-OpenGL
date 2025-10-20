use std::{path::PathBuf, rc::Rc};

use crate::{
    entities::Entity,
    gl,
    helpers::{GlPosition, Mat3DUpdate},
    renderer::shader::{Array, Drawable, GlslPass, Shader, Tex, create_shader},
};

#[derive(Clone)]
pub struct Square {
    pub bottom_left: GlPosition,
    pub top_right: GlPosition,
}

impl Square {
    pub fn as_vertex_stride(&self) -> [GlPosition; 4] {
        let (minx, maxx) = (
            self.bottom_left.x.min(self.top_right.x),
            self.bottom_left.x.max(self.top_right.x),
        );
        let (miny, maxy) = (
            self.bottom_left.y.min(self.top_right.y),
            self.bottom_left.y.max(self.top_right.y),
        );
        let (minz, maxz) = (
            self.bottom_left.z.min(self.top_right.z),
            self.bottom_left.z.max(self.top_right.z),
        );

        if minx == maxx {
            // x = const (left/right). Strip order: (y-,z-), (y+,z-), (y-,z+), (y+,z+)
            [
                GlPosition::new(minx, miny, minz),
                GlPosition::new(minx, maxy, minz),
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(minx, maxy, maxz),
            ]
        } else if miny == maxy {
            // y = const (top/bottom). Vary x,z.
            [
                GlPosition::new(minx, miny, minz),
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(maxx, miny, minz),
                GlPosition::new(maxx, miny, maxz),
            ]
        } else {
            // z = const (front/back). Vary x,y.
            [
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(minx, maxy, maxz),
                GlPosition::new(maxx, miny, maxz),
                GlPosition::new(maxx, maxy, maxz),
            ]
        }
    }

    pub fn as_vertex_stride_w_tex_mapping(&self) -> [(GlPosition, GlPosition); 4] {
        let [bl, tl, br, tr] = self.as_vertex_stride();
        [
            (bl, GlPosition::new(0.0, 0.0, 0.0f32)),
            (tl, GlPosition::new(0.0, 1.0, 0.0f32)),
            (br, GlPosition::new(1.0, 0.0, 0.0f32)),
            (tr, GlPosition::new(1.0, 1.0, 0.0f32)),
        ]
    }
}

pub struct TexSquare {
    shader: Option<Shader>,
    instances: Vec<Square>,
    texture: Option<PathBuf>,
}

impl TexSquare {
    pub fn new(instances: Vec<Square>, texture: Option<PathBuf>) -> Self {
        Self {
            shader: None,
            instances,
            texture,
        }
    }
}

impl GlslPass for TexSquare {
    fn init(&mut self, gl_fns: Rc<crate::gl::Gles2>, mat3d: Mat3DUpdate) {
        let image = self.texture.as_ref().map(|path| {
            image::ImageReader::open(path)
                .unwrap_or_else(|_| panic!("{path:?} should be readable"))
                .decode()
                .unwrap_or_else(|_| panic!("{path:?} should be decodable"))
        });

        let program;
        let mut vao;
        let mut vbo;

        let mat3d = mat3d.as_init();

        let vertex_data: Vec<f32> = self
            .instances
            .iter()
            .flat_map(|sq| sq.as_vertex_stride_w_tex_mapping())
            .flat_map(|(p, t)| [p.x, p.y, p.z, t.x, t.y])
            .collect();

        let tex;

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
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            let tex_attrib =
                gl_fns.GetAttribLocation(program, c"textureCoord".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                tex_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(tex_attrib as gl::types::GLuint);

            // -- TEXTURE
            tex = image.map(|image| {
                let mut tex_id = std::mem::zeroed();
                gl_fns.GenTextures(1, &mut tex_id);
                gl_fns.BindTexture(gl::TEXTURE_2D, tex_id);
                gl_fns.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB as i32,
                    image.width() as i32,
                    image.height() as i32,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    image.to_rgb8().as_raw().as_ptr() as *const _,
                );
                gl_fns.GenerateMipmap(gl::TEXTURE_2D);
                tex_id
            });
            // --
        }

        let tex = tex.map(|tex| Tex {
            tex,
            target: gl::TEXTURE_2D,
        });

        let drawable = Drawable::Array(Array {
            vao,
            vbo,
            len: self.instances.len(),
            offset: 4,
            count: 4,
        });

        let drawables = vec![drawable];

        self.shader = Some(Shader {
            program,
            model_transform: mat3d
                .model
                .expect("mat3d as_init should be at least IDENTITY"),
            tex,
            drawables,
            gl_fns,
        })
    }

    fn update(&mut self, mat3d: Mat3DUpdate, _light_pos: Option<glam::Vec3>) {
        if let Some(shader) = &mut self.shader {
            if let Some(model_updated) = mat3d.model {
                shader.model_transform = model_updated;
            }
            unsafe { mat3d.set_uniforms(&shader.gl_fns, shader.program) };
        }
    }

    fn get_shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }
}

impl Entity for TexSquare {}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 textureCoord;

out vec2 TexCoord;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    TexCoord = textureCoord;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

layout(location = 0) out vec4 FragColor;

in vec2 TexCoord;
uniform sampler2D tex;


void main() {
    FragColor = texture(tex, TexCoord);
}
\0";
