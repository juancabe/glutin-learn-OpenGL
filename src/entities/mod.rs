use crate::renderer::shader::GlslPass;

pub mod dirt_square;
pub mod hello_triangle;
pub trait Entity: GlslPass {}
