use std::io::Read;

use wgpu::{Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor, util::StagingBelt};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Text, ab_glyph};
use winit::{dpi::PhysicalSize, window::Window};

const TITLEBAR_MARGIN: f32 = 30.0;

pub struct AppFrontend {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub glyph_brush: GlyphBrush<()>,
    pub staging_belt: StagingBelt,
    pub buffer: Vec<u8>,
    pub scale_factor: f32,
}

impl AppFrontend {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor { label: None, features: Features::empty(), limits: Limits::default() },
            None
        ).await.unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };
        surface.configure(&device, &config);

        let render_format = TextureFormat::Bgra8UnormSrgb;
        let font = ab_glyph::FontArc::try_from_slice(include_bytes!("iosevka.ttf")).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

        let staging_belt = StagingBelt::new(1024);

        Self {
            surface, device, queue, config, size, glyph_brush, staging_belt,
            scale_factor: window.scale_factor() as f32,
            buffer: vec![]
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, scale_factor: f32) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            if scale_factor > 0.0 {
                self.scale_factor = scale_factor;
            }
        }
    }

    pub fn set_data(&mut self, buf: &mut Vec<u8>) {
        if buf == &[8, 27, 91, 75] {
            self.buffer.pop();
        } else if buf != &[7] {
            self.buffer.append(buf);
        }
    }


    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render encoder") });
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0
                        }),
                        store: true
                    }
                }],
                depth_stencil_attachment: None
            });
        }

        let cell_width = 10.0 * self.scale_factor;
        let cell_height = 20.0 * self.scale_factor;

        let mut i = 0;
        let mut j = 0;
        let mut row = 0.0;
        let mut col = 0.0;
        while i < 24 {
            while j < 80 {
                let k = i * 80 + j;
                if self.buffer.len() > k {
                    let c = String::from_utf8(vec![self.buffer[k]]).unwrap();
                    if c != "\r" && c != "\n" && c != "\t" {
                        let x = col * cell_width;
                        let y = row * cell_height;
                        self.glyph_brush.queue(Section {
                            screen_position: (30.0 + x, (30.0 + TITLEBAR_MARGIN) + y),
                            bounds: (self.size.width as f32, self.size.height as f32),
                            text: vec![Text::new(&c)
                                .with_color([1.0, 1.0, 1.0, 1.0])
                                .with_scale(cell_height)],
                                ..Section::default()
                        });
                    }
                    if c == "\n" {
                        col = 0.0;
                        row += 1.0;
                        j += 1;
                        continue;
                    }
                    if c == "\t" {
                        col += (col as i32 / 20) as f32;
                    }
                }
                j += 1;
                col += 1.0;
            }
            i += 1;
            row += 1.0;
            j = 0;
            col = 0.0;
        }

        self.glyph_brush.draw_queued(&self.device, &mut self.staging_belt, &mut encoder, &view, self.size.width, self.size.height).ok();
        self.staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

