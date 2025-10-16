use crate::renderer::shader::GlslPass;

pub mod dirt_cube;
pub mod dirt_square;
pub mod hello_triangle;
pub mod utah_teapot;
pub trait Entity: GlslPass {}
