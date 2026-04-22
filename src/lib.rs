pub mod app;
pub mod render;
pub mod ui;
use bael_macro::externed;

/// Convenience re-exports so other crates can depend only on `bael`.
/// Use `use bael::prelude::*;` to import common types and macros.
pub mod prelude {
    pub use crate::render::vertex::Vertex;
    pub use crate::render::texture::Texture;
    pub use crate::render::state::State;
    pub use anyhow::Result;
    pub use pretty_env_logger;
    pub use pollster;
    pub use winit;
}

#[externed]
pub mod controller{
    pub use bael_macro::widget;
    pub use crate::ui::widget::Widget;
    pub use winit::window::WindowId;
    pub use crate::ui::widget_creator::{create, register, get_handle};
}