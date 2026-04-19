mod render;
mod app;
mod error;

use log::error;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use crate::render::window_manager::WindowManager;

fn main() {
    pretty_env_logger::init();

    let event_loop: EventLoop<()> = EventLoop::new().unwrap();

    let wm = match WindowManager::new(
        &event_loop, 
        &"Test Window".to_string(), 
        800, 
        600) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to create WindowManager, error: {}", e);
            return;
        }
    };

    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => wm.get_window().request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() => {},
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                _ => {}
            }
            _ => {}
        }
    }).unwrap();
}
