use wgpu::{Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor, util::StagingBelt};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Text, ab_glyph};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{characters::{BACK_CHAR, BELL_CHAR, CR_CHAR, ESC_CHAR}, constants::{TERMINAL_COLS, TERMINAL_ROWS, TITLEBAR_MARGIN}, cursor::Cursor};

const BG_CHAR: &str = "█";
const BGR_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.0];
const CUR_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const CHR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub struct Terminal {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub glyph_brush: GlyphBrush<()>,
    pub staging_belt: StagingBelt,
    pub buffer: Vec<u8>,
    pub scale_factor: f32,
    pub cursor: Cursor
}

impl Terminal {
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
            buffer: vec![],
            cursor: Cursor::new()
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
        for b in buf {
            let b = *b as char;
            if b == BACK_CHAR {
                self.buffer.pop();
            } else if b == ESC_CHAR {
                return;
            } else if b != CR_CHAR && b != BELL_CHAR {
                self.buffer.push(b as u8);
            }
        }
    }

    pub fn put_char(&mut self, c: &str, color: [f32; 4], row: f32, col: f32) {
        let cell_width = 9.0 * self.scale_factor;
        let cell_height = 20.0 * self.scale_factor;
        let x = col * cell_width;
        let y = row * cell_height;
        self.glyph_brush.queue(Section {
            screen_position: (30.0 + x, (30.0 + TITLEBAR_MARGIN) + y),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(c)
                .with_color(color)
                .with_scale(cell_height)],
                ..Section::default()
        });
    }

    pub fn fill_line_at(&mut self, row: f32, col: f32) {
        let mut col = col;
        while col < TERMINAL_COLS as f32 {
            self.put_char(BG_CHAR, BGR_COLOR, row, col);
            col += 1.0;
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


        let mut row = 0.0;
        let mut col = 0.0;
        let mut i = 0;
        while row < TERMINAL_ROWS as f32 {
            while col < TERMINAL_COLS as f32 {
                let b = if i < self.buffer.len() { self.buffer[i] } else { 0 };
                let c = String::from_utf8(vec![b]).unwrap();
                // Draw background or cursor
                if i == self.buffer.len() {
                    self.put_char(BG_CHAR, CUR_COLOR, row, col);
                } else {
                    self.put_char(BG_CHAR, BGR_COLOR, row, col);
                }
                // Draw character
                if c.starts_with("\n") {
                    self.fill_line_at(row, col);
                    col = -1.0;
                    row += 1.0;
                } else {
                    self.put_char(&c, CHR_COLOR, row, col);
                }
                i += 1;
                col += 1.0;
            }
            row += 1.0;
            col = 0.0;
        }

        self.glyph_brush.draw_queued(&self.device, &mut self.staging_belt, &mut encoder, &view, self.size.width, self.size.height).ok();
        self.staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

