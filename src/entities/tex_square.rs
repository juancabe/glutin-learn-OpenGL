use std::{path::PathBuf, rc::Rc};

use crate::{
    entities::Entity,
    gl,
    helpers::{GlPosition, Mat3DUpdate},
    renderer::shader::{Array, Drawable, GlslPass, Shader, Tex, create_shader, uniform::Uniform},
};

pub struct SquareVertex {
    pub position: glam::Vec3,
    pub tex_map: glam::Vec2,
    pub normal: glam::Vec3,
}

impl SquareVertex {
    pub fn new(position: GlPosition, tex_map: glam::Vec2, normal: glam::Vec3) -> Self {
        Self {
            position,
            tex_map,
            normal,
        }
    }

    pub const FLAT_SIZE: usize = 8;

    pub fn flatten(&self) -> [f32; Self::FLAT_SIZE] {
        [
            self.position.x,
            self.position.y,
            self.position.z,
            self.tex_map.x,
            self.tex_map.y,
            self.normal.x,
            self.normal.y,
            self.normal.z,
        ]
    }
}

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

        if self.bottom_left.x == self.top_right.x {
            // x = const
            [
                GlPosition::new(minx, miny, minz),
                GlPosition::new(minx, maxy, minz),
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(minx, maxy, maxz),
            ]
        } else if self.bottom_left.y == self.top_right.y {
            // y = const
            [
                GlPosition::new(minx, miny, minz),
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(maxx, miny, minz),
                GlPosition::new(maxx, miny, maxz),
            ]
        } else {
            // z = const
            [
                GlPosition::new(minx, miny, maxz),
                GlPosition::new(minx, maxy, maxz),
                GlPosition::new(maxx, miny, maxz),
                GlPosition::new(maxx, maxy, maxz),
            ]
        }
    }

    pub fn as_vertex_data(&self) -> [SquareVertex; 4] {
        let [bl, mut tl, mut br, tr] = self.as_vertex_stride();

        // Decide if we must flip winding so the normal points outward
        let x_const = self.bottom_left.x == self.top_right.x;
        let y_const = self.bottom_left.y == self.top_right.y;
        // let z_const = self.bottom_left.z == self.top_right.z;

        // Rules derived from how build_faces sets bottom_left/top_right
        let flip_winding = if x_const {
            // left face (x = -h) has bl.z < tr.z
            self.bottom_left.z < self.top_right.z
        } else if y_const {
            // bottom face (y = -h) has bl.z < tr.z
            self.bottom_left.z < self.top_right.z
        } else {
            // z = const: front face (z = +h) has bl.y > tr.y
            self.bottom_left.y > self.top_right.y
        };

        if flip_winding {
            // Keep texture orientation consistent and flip winding
            std::mem::swap(&mut tl, &mut br);
        }

        // Recompute normal after potential swap
        let normal = (tl - bl).cross(br - bl).normalize();

        [
            SquareVertex::new(bl, glam::Vec2::new(0.0, 0.0), normal),
            SquareVertex::new(tl, glam::Vec2::new(0.0, 1.0), normal),
            SquareVertex::new(br, glam::Vec2::new(1.0, 0.0), normal),
            SquareVertex::new(tr, glam::Vec2::new(1.0, 1.0), normal),
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
    fn init(
        &mut self,
        gl_fns: Rc<crate::gl::Gles2>,
        mat3d: Mat3DUpdate,
        init_uniforms: &[Box<dyn Uniform>],
    ) {
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
            .flat_map(|sq| sq.as_vertex_data())
            .flat_map(|sv| sv.flatten())
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
            assert_ne!(pos_attrib, -1);
            gl_fns.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                SquareVertex::FLAT_SIZE as i32 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            let tex_attrib =
                gl_fns.GetAttribLocation(program, c"textureCoord".as_ptr() as *const _);
            assert_ne!(tex_attrib, -1);
            gl_fns.VertexAttribPointer(
                tex_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                SquareVertex::FLAT_SIZE as i32 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            let norm_attrib = gl_fns.GetAttribLocation(program, c"normal".as_ptr() as *const _);
            assert_ne!(norm_attrib, -1);
            gl_fns.VertexAttribPointer(
                norm_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                SquareVertex::FLAT_SIZE as i32 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (5 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            gl_fns.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(tex_attrib as gl::types::GLuint);
            gl_fns.EnableVertexAttribArray(norm_attrib as gl::types::GLuint);

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

            for uniform in init_uniforms {
                uniform.set(&gl_fns, program);
            }
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

    fn update(&mut self, mat3d: Mat3DUpdate, to_set_uniforms: &[Box<dyn Uniform>]) {
        if let Some(shader) = &mut self.shader {
            if let Some(model_updated) = mat3d.model {
                shader.model_transform = model_updated;
            }
            unsafe { mat3d.set_uniforms(&shader.gl_fns, shader.program) };

            for uniform in to_set_uniforms {
                uniform.set(&shader.gl_fns, shader.program);
            }
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
layout(location = 2) in vec3 normal;

out vec2 TexCoord;
out vec3 fragNorm;
out vec3 fragPos;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    TexCoord = textureCoord;
    fragPos = vec3(model * vec4(position, 1.0));
    fragNorm = mat3(transpose(inverse(model))) * normal;  

}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core

layout(location = 0) out vec4 FragColor;

uniform vec3 uLightPos;
uniform vec3 uEyePos;

uniform float uFogNear;
uniform float uFogFar;
uniform vec3 uFogColor;

uniform float uAmbientStrenght;
uniform float uSpecularStrength;

in vec2 TexCoord;
in vec3 fragNorm;
in vec3 fragPos;

uniform sampler2D tex;

void main() {
    vec3 norm = normalize(fragNorm);

    vec3 lightDir = normalize(uLightPos - fragPos);
    float diffuse = max(dot(norm, lightDir), 0.0);

    vec3 viewDir = normalize(uEyePos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
    float specular = uSpecularStrength * spec;

    vec4 albedo = texture(tex, TexCoord);
    vec3 sceneColor = albedo.rgb * (uAmbientStrenght + diffuse + specular);
    
    float d = length(uEyePos - fragPos);
    float f = clamp((uFogFar - d) / (uFogFar - uFogNear), 0.0, 1.0);

    vec3 finalRgb = mix(uFogColor, sceneColor, f);

    FragColor = vec4(finalRgb, 1.0);
}
\0";
