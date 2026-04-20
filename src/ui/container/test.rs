use crate::render::vertex::Vertex;
use crate::ui::widget::Widget;

// widget-local constants (moved from render backend)
const WIDGET_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    },
];

// include a sample texture inside the widget module
const WIDGET_TEXTURE_BYTES: &'static [u8] =
    include_bytes!("../../render/state/happy-tree.png");

pub struct Test {
    pub pattern: Vec<Vertex>,
    pub texture: Vec<u8>,
    pub window_id: winit::window::WindowId,
}

impl Test {
    pub fn new(window_id: winit::window::WindowId) -> Self {
        Self {
            pattern: WIDGET_VERTICES.to_vec(),
            texture: WIDGET_TEXTURE_BYTES.to_vec(),
            window_id,
        }
    }
}

impl Widget for Test {
    fn set_pattern(&mut self, pattern: Vec<Vertex>) {
        self.pattern = pattern;
    }

    fn set_texture(&mut self, texture: Vec<u8>) {
        self.texture = texture;
    }

    fn get_pattern(&self) -> &Vec<Vertex> {
        return &self.pattern;
    }

    fn get_texture(&self) -> &Vec<u8> {
        return &self.texture;
    }

    fn get_window(&self) -> winit::window::WindowId {
        return self.window_id;
    }
}
