#![allow(unused)]

pub mod new;
pub mod functions;

use std::collections::HashMap;
use wgpu::Device;
use winit::{event::WindowEvent, window::{Window, WindowId}};
use crate::render::vertex::Vertex;

use winit::event_loop::EventLoopProxy;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

/// Events sent from renderer to the event loop (used for requesting redraws and submissions).
#[derive(Clone, Debug)]
pub enum RenderEvent {
    RequestRedraw(WindowId),
    Submit(WindowId, WidgetSubmission),
}

/// A submission from a widget describing geometry and texture pixels to render.
#[derive(Clone, Debug)]
pub struct WidgetSubmission {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_bytes: Vec<u8>,
}

struct RenderTarget {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

/// Central renderer state. Keeps shared GPU resources and per-window targets.
pub struct State {
    instance: wgpu::Instance,
    // per-window render targets
    targets: HashMap<WindowId, RenderTarget>,

    // shared GPU resources
    device: wgpu::Device,
    queue: wgpu::Queue,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,

    index_buffer: wgpu::Buffer, 
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup, 
    // the bind group layout used to create the pipeline; reuse for widget bind groups
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // submissions from UI/widgets mapped by window id
    pub(crate) pending_widgets: HashMap<WindowId, Vec<WidgetSubmission>>,
    // optional per-window event proxies used to request redraws automatically
    pub(crate) proxies: HashMap<WindowId, EventLoopProxy<RenderEvent>>,
}

impl State {
    /// Register an EventLoopProxy for a window so the renderer can request redraws.
    pub fn register_event_proxy(&mut self, window_id: WindowId, proxy: EventLoopProxy<RenderEvent>) {
        self.proxies.insert(window_id, proxy);
    }
}

// Global singleton holder for State
static GLOBAL_STATE: OnceCell<Mutex<State>> = OnceCell::new();

impl State {
    /// Initialize the global singleton. Call once at startup.
    pub async fn init_singleton(window: &Window) {
        let s = Self::new(window).await;
        let _ = GLOBAL_STATE.set(Mutex::new(s));
    }

    /// Get the global `Mutex<State>`. Panics if `init_singleton` was not called.
    pub fn global() -> &'static Mutex<State> {
        GLOBAL_STATE.get().expect("State not initialized; call State::init_singleton")
    }

    /// Submit a widget from any thread. If an `EventLoopProxy` is registered for the window,
    /// this will send a `RenderEvent::Submit` to the event loop; otherwise it will push
    /// directly into the pending queue.
    pub fn submit_threadsafe(window_id: WindowId, vertices: Vec<Vertex>, indices: Vec<u16>, texture_bytes: Vec<u8>) {
        let submission = WidgetSubmission { vertices, indices, texture_bytes };
        if let Some(cell) = GLOBAL_STATE.get() {
            // try to fetch a proxy briefly
            let proxy_opt = {
                let state = cell.lock().unwrap();
                state.proxies.get(&window_id).cloned()
            };
            if let Some(proxy) = proxy_opt {
                let _ = proxy.send_event(RenderEvent::Submit(window_id, submission));
                return;
            }
            // fallback: push into pending directly
            let mut state = cell.lock().unwrap();
            state.submit_to(window_id, submission.vertices, submission.indices, submission.texture_bytes);
        }
    }
}