use crate::render::vertex::Vertex;

pub trait Widget {
    fn new(window_id: winit::window::WindowId, pattern:&Vec<Vertex>, texture:&Vec<u8>) -> Self
    where
        Self: Sized;

    fn set_pattern(&mut self, pattern: Vec<Vertex>);
    fn get_pattern(&self) -> &Vec<Vertex>;
    fn set_texture(&mut self, texture: Vec<u8>);
    fn get_texture(&self) -> &Vec<u8>;

    fn get_window(&self) -> winit::window::WindowId;

    fn show(&self) {
        let mut renderer = crate::render::state::State::global().lock().unwrap();
        // Submit the sample constants as a widget to the first available window
        let indices = crate::render::vertex::Vertex::generate_fan_indices(self.get_pattern());
        renderer.submit_to(
            self.get_window().clone(),
            self.get_pattern().clone(),
            indices,
            self.get_texture().clone(),
        );
    }
}
