use crate::renderer::shader::GlslPass;

pub mod hello_triangle;
pub mod sun;
pub mod tex_cube;
pub mod tex_square;
pub mod utah_teapot;
pub trait Entity: GlslPass {}
