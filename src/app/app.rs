use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use winit::window::{Window, WindowBuilder};

use crate::render::state::{RenderEvent, State};

pub struct AppRunner {
    event_loop: EventLoop<RenderEvent>,
    window: Window,
}

impl AppRunner {
    pub async fn new() -> Self {
        let event_loop: EventLoop<RenderEvent> =
            EventLoopBuilder::<RenderEvent>::with_user_event().build();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        // initialize global State singleton
        State::init_singleton(&window).await;

        // register proxy
        let proxy: EventLoopProxy<RenderEvent> = event_loop.create_proxy();
        {
            let mut state = State::global().lock().unwrap();
            state.register_event_proxy(window.id(), proxy);
            // if submissions were queued before proxy registration, request a redraw
            if let Some(list) = state.pending_widgets.get(&window.id()) {
                if !list.is_empty() {
                    let _ = state
                        .proxies
                        .get(&window.id())
                        .map(|p| p.send_event(RenderEvent::RequestRedraw(window.id())));
                }
            }
        }

        Self { event_loop, window }
    }

    pub fn run(self) {
        let AppRunner { event_loop, window } = self;

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let mut state = State::global().lock().unwrap();
                state.update();
                match state.render_target(window_id) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        if let Some(sz) = state.target_size(window_id) {
                            state.resize_target(window.id(), sz);
                        }
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // 已改为由提交（submit_to）触发重绘，避免无条件每帧重绘引起 Vulkan 信号量复用错误
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                let mut state = State::global().lock().unwrap();
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize_target(window.id(), *physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize_target(window.id(), **new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::UserEvent(e) => match e {
                RenderEvent::RequestRedraw(id) => {
                    if id == window.id() {
                        window.request_redraw();
                    }
                }
                RenderEvent::Submit(id, submission) => {
                    // push submission into pending and request redraw
                    let mut state = State::global().lock().unwrap();
                    state
                        .pending_widgets
                        .entry(id)
                        .or_default()
                        .push(submission);
                    if id == window.id() {
                        window.request_redraw();
                    }
                }
            },
            _ => {}
        });
    }

    /// Run the event loop and execute `startup` once on the event-loop thread when the loop starts.
    pub fn run_with_startup<F: FnOnce() + 'static>(self, startup: F) {
        let AppRunner { event_loop, window } = self;
        let mut startup_opt = Some(Box::new(startup));

        event_loop.run(move |event, _, control_flow| {
            // call startup on the Init event exactly once
            if let Event::NewEvents(StartCause::Init) = event {
                if let Some(cb) = startup_opt.take() {
                    cb();
                }
            }

            match event {
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let mut state = State::global().lock().unwrap();
                    state.update();
                    match state.render_target(window_id) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(sz) = state.target_size(window_id) {
                                state.resize_target(window.id(), sz);
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    // no-op; redraws triggered by submissions
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    let mut state = State::global().lock().unwrap();
                    if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                state.resize_target(window.id(), *physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                state.resize_target(window.id(), **new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::UserEvent(e) => match e {
                    RenderEvent::RequestRedraw(id) => {
                        if id == window.id() {
                            window.request_redraw();
                        }
                    }
                    RenderEvent::Submit(id, submission) => {
                        let mut state = State::global().lock().unwrap();
                        state
                            .pending_widgets
                            .entry(id)
                            .or_default()
                            .push(submission);
                        if id == window.id() {
                            window.request_redraw();
                        }
                    }
                },
                _ => {}
            }
        });
    }
}
