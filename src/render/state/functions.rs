use winit::event::WindowEvent;

use crate::render::state::WidgetSubmission;
use crate::render::state::{RenderTarget, State};
use crate::render::texture::Texture;
use crate::render::vertex::Vertex;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::WindowId;

impl State {
    /// Resize a specific window target.
    pub fn resize_target(&mut self, window_id: WindowId, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let Some(t) = self.targets.get_mut(&window_id) {
                t.size = new_size;
                t.config.width = new_size.width;
                t.config.height = new_size.height;
                t.surface.configure(&self.device, &t.config);
            }
        }
    }

    pub(crate) fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub(crate) fn update(&mut self) {}

    /// Submit a widget for rendering to a specific window id.
    pub fn submit_to(
        &mut self,
        window_id: WindowId,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        texture_bytes: Vec<u8>,
    ) {
        let s = WidgetSubmission {
            vertices,
            indices,
            texture_bytes,
        };
        self.pending_widgets.entry(window_id).or_default().push(s);
        // if we have a proxy registered for this window, request a redraw
        if let Some(proxy) = self.proxies.get(&window_id) {
            let _ = proxy.send_event(crate::render::state::RenderEvent::RequestRedraw(
                window_id,
            ));
        }
    }

    /// Add a new window target to the renderer. Returns the created WindowId.
    pub fn add_window(&mut self, window: &winit::window::Window) -> WindowId {
        let size = window.inner_size();
        let surface = unsafe { self.instance.create_surface(window) };
        // reuse the config format from any existing target if present
        let format = self
            .targets
            .values()
            .next()
            .map(|t| t.config.format)
            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&self.device, &config);

        let id = window.id();
        self.targets.insert(
            id,
            RenderTarget {
                surface,
                config,
                size,
            },
        );
        self.pending_widgets.entry(id).or_default();
        id
    }

    /// Render all targets.
    pub fn render_all(&mut self) -> Result<(), wgpu::SurfaceError> {
        // iterate over keys to avoid borrowing issues
        let ids: Vec<WindowId> = self.targets.keys().cloned().collect();
        for id in ids {
            self.render_target(id)?;
        }
        Ok(())
    }

    /// Render a specific target by window id.
    pub fn render_target(&mut self, window_id: WindowId) -> Result<(), wgpu::SurfaceError> {
        let target = match self.targets.get_mut(&window_id) {
            Some(t) => t,
            None => return Err(wgpu::SurfaceError::Lost),
        };

        let output = target.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare per-submission GPU resources for this window
        let mut gpu_subs = Vec::new();
        if let Some(list) = self.pending_widgets.remove(&window_id) {
            for submission in list {
                let tex = Texture::from_bytes(
                    &self.device,
                    &self.queue,
                    &submission.texture_bytes,
                    "widget-texture",
                )
                .unwrap();
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&tex.sampler),
                        },
                    ],
                    label: Some("widget_bind_group"),
                });

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("widget-vertex-buffer"),
                            contents: bytemuck::cast_slice(&submission.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("widget-index-buffer"),
                            contents: bytemuck::cast_slice(&submission.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                let num_indices = submission.indices.len() as u32;

                gpu_subs.push((tex, bind_group, vertex_buffer, index_buffer, num_indices));
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            // Draw default geometry as fallback first
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // Then draw submitted widgets on top
            for (_tex, bind_group, vbuf, ibuf, num_indices) in gpu_subs.iter() {
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vbuf.slice(..));
                render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Return any known window id (useful for examples/tests).
    pub fn first_window_id(&self) -> Option<WindowId> {
        self.targets.keys().cloned().next()
    }

    /// Return the stored size of the target if present.
    pub fn target_size(&self, window_id: WindowId) -> Option<winit::dpi::PhysicalSize<u32>> {
        self.targets.get(&window_id).map(|t| t.size)
    }
}
