use winit::window::WindowId;

use crate::render::vertex::Vertex;

pub trait Widget {
    fn set_pattern(&mut self, pattern: Vec<Vertex>);
    fn get_pattern(&self) -> &Vec<Vertex>;
    fn set_texture(&mut self, texture: Vec<u8>);
    fn get_texture(&self) -> &Vec<u8>;

    fn get_window(&self) -> WindowId;

    fn show(&self){
        let mut renderer = crate::render::state::State::global().lock().unwrap();
        // Submit the sample constants as a widget to the first available window
        if let Some(window_id) = renderer.first_window_id() {
            let indices = crate::render::vertex::Vertex::generate_fan_indices(self.get_pattern());
            renderer.submit_to(
                window_id,
                self.get_pattern().clone(),
                indices,
                self.get_texture().clone(),
            );
        }
    }
}
