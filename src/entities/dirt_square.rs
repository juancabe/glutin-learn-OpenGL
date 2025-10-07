use crate::{
    entities::Entity,
    gl,
    helpers::{GlPosition, Shader},
    renderer::shader::{GlslPass, create_shader},
};

#[derive(Clone)]
pub struct Square {
    pub bottom_left: GlPosition,
    pub top_right: GlPosition,
}

impl Square {
    pub fn as_vertex_stride(&self) -> [GlPosition; 4] {
        [
            self.bottom_left,
            GlPosition::new_2d(self.bottom_left.x, self.top_right.y),
            GlPosition::new_2d(self.top_right.x, self.bottom_left.y),
            self.top_right,
        ]
    }

    pub fn as_vertex_stride_w_tex_mapping(&self) -> [(GlPosition, GlPosition); 4] {
        let [bl, tl, br, tr] = self.as_vertex_stride();
        [
            (bl, GlPosition::new_2d(0.0, 0.0)),
            (tl, GlPosition::new_2d(0.0, 1.0)),
            (br, GlPosition::new_2d(1.0, 0.0)),
            (tr, GlPosition::new_2d(1.0, 1.0)),
        ]
    }
}

pub struct DirtSquare {
    glsl_pass: Option<Shader>,
    instances: Vec<Square>,
}

impl DirtSquare {
    pub fn new(instances: Vec<Square>) -> Self {
        Self {
            glsl_pass: None,
            instances,
        }
    }
}

impl GlslPass for DirtSquare {
    fn init(&mut self, gl_fns: std::sync::Arc<crate::gl::Gles2>) {
        let image = image::ImageReader::open("./assets/dirt.webp")
            .expect("assets/dirt.webp should be readable")
            .decode()
            .expect("assets/dirt.webp should be decodable");

        let program;
        let mut vao;
        let mut vbo;

        let vertex_data: Vec<f32> = self
            .instances
            .iter()
            .flat_map(|sq| sq.as_vertex_stride_w_tex_mapping())
            .flat_map(|(p, t)| [p.x, p.y, t.x, t.y])
            .collect();

        let mut tex;

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

            let pos_attrib = gl_fns.GetAttribLocation(program, c"position".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                4 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            let tex_attrib =
                gl_fns.GetAttribLocation(program, c"textureCoord".as_ptr() as *const _);
            gl_fns.VertexAttribPointer(
                tex_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                4 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(tex_attrib as gl::types::GLuint);

            // -- TEXTURE
            tex = std::mem::zeroed();
            gl_fns.GenTextures(1, &mut tex);
            gl_fns.BindTexture(gl::TEXTURE_2D, tex);
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
            // --
        }

        self.glsl_pass = Some(Shader {
            program,
            vao,
            vbo,
            gl_fns,
            tex: Some(tex),
        })
    }

    fn update(&mut self, _dt: &std::time::Duration) {
        ()
    }

    fn draw(&self) {
        if let Some(glsl_pass) = &self.glsl_pass {
            let gl = &glsl_pass.gl_fns;
            let program = glsl_pass.program;
            let vao = glsl_pass.vao;
            let vbo = glsl_pass.vbo;
            let tex = glsl_pass.tex.expect("Tex should have being set");

            unsafe {
                gl.UseProgram(program);

                gl.BindTexture(gl::TEXTURE_2D, tex);
                gl.BindVertexArray(vao);
                gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

                gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            }
        } else {
            log::warn!("Tried to render DirtSquare before init")
        }
    }
}

impl Drop for DirtSquare {
    // NOTE: This is the same as HelloTriangle, maybe a clue for an
    // abstraction 🤔
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
            log::warn!("Dropped DirtSquare before even initializing it")
        }
    }
}

impl Entity for DirtSquare {}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 textureCoord;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    TexCoord = textureCoord;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

layout(location = 0) out vec4 FragColor;

in vec2 TexCoord;
uniform sampler2D dirtTexture;


void main() {
    FragColor = texture(dirtTexture, TexCoord);
}
\0";
