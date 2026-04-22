use crate::render::vertex::Vertex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Widget: Send + Sync {
    // 构造函数需要接收 widget_id（由调用方提供）
    fn new(
        &mut self,
        widget_id: u64,
        window_id: winit::window::WindowId,
        pattern: &Vec<Vertex>,
        texture: &Vec<u8>,
    );

    // 每个具体 Widget 必须能返回自己的 id
    fn get_id(&self) -> u64;
    fn set_pattern(&mut self, pattern: Vec<Vertex>);
    fn get_pattern(&self) -> &Vec<Vertex>;
    fn set_texture(&mut self, texture: Vec<u8>);
    fn get_texture(&self) -> &Vec<u8>;

    fn get_window(&self) -> winit::window::WindowId;

    // 父子管理，交由宏生成的实现处理（只提供签名）
    fn set_father_widget(&mut self, father_id: u64);

    fn add(&mut self, child_id: u64);

    fn get_sons(&self) -> HashMap<u64, Arc<Mutex<Box<dyn Widget + Send + Sync>>>>;

    fn get_father(&self) -> Option<Arc<Mutex<Box<dyn Widget + Send + Sync>>>>;

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
