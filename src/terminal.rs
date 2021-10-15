use std::usize;
use wgpu::{Backends, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Face, Features, FragmentState, FrontFace, Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState, util::{BufferInitDescriptor, DeviceExt, StagingBelt}};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, GlyphCruncher, Section, Text, ab_glyph::{self, Rect}};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{characters::{BACK_CHAR, BELL_CHAR, CR_CHAR, ESC_CHAR, EscapeCode, NEWLINE_CHAR, SPACE_CHAR, TAB_CHAR}, constants::{TERMINAL_COLS, TERMINAL_ROWS, TITLEBAR_MARGIN}, cursor::{Cursor, CursorDirection}, screen::ScreenBuffer};

// REF: https://www.vt100.net/docs/la100-rm/chapter2.html

const TAB_STOP: usize = 8;
const FONT_SIZE: f32 = 20.0;
const CUR_CHAR: &str = "█";
const CUR_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 0.5];
const CHR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const CELL_COLOR: [f32; 3] = [0.015, 0.015, 0.015];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

pub struct Terminal {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub glyph_brush: GlyphBrush<()>,
    pub staging_belt: StagingBelt,
    pub buffer: ScreenBuffer,
    pub scale_factor: f32,
    pub cursor: Cursor,
    pub cell_size: Rect,
    pub quad_pipeline: RenderPipeline,
    start_line: usize
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
        let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);
        let bounds = glyph_brush.glyph_bounds(Section::default().add_text(Text::new("A").with_scale(FONT_SIZE))).unwrap();

        let staging_belt = StagingBelt::new(1024);

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        let quad_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[ Vertex::desc() ]
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL
                }]
            }),
            depth_stencil: None,
            multisample: MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false
            }
        });

        Self {
            surface, device, queue, config, size, glyph_brush, staging_belt,
            scale_factor: window.scale_factor() as f32,
            buffer: ScreenBuffer::new(TERMINAL_COLS as usize, TERMINAL_ROWS as usize),
            cell_size: bounds,
            cursor: Cursor::new(),
            quad_pipeline,
            start_line: 0
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
        let mut i = 0;
        while i < buf.len() {
            let b = buf[i] as char;
            if b == BACK_CHAR {
                self.cursor.move_to(CursorDirection::Left);
            } else if b == ESC_CHAR {
                let (remain, param, inter, final_byte) = EscapeCode::parse_csi(&buf[i+1..]);
                // Process code here
                let param = param.iter().map(|c| *c as char).collect::<String>();
                let inter = inter.iter().map(|c| *c as char).collect::<String>();
                let final_byte = final_byte as char;
                match (param.as_str(), inter.as_str(), final_byte) {
                    ("", "", 'J') | ("0", "", 'J') | ("1", "", 'J') | ("2", "", 'J') => {
                        // Just clear everything for now
                        self.buffer.clear();
                        self.start_line = 0;
                    },
                    ("", "", 'K') => {
                        self.buffer.set_char_at(0, self.cursor.row, self.cursor.col);
                    },
                    ("", "", 'H') => {
                        self.cursor.move_to(CursorDirection::BOF);
                    },
                    _ => println!("Unhandled CSI sequence: [{}{}{}", param, inter, final_byte)
                }
                let parsed_size = buf.len() - remain.len() - 1;
                if parsed_size > 0 {
                    i += parsed_size;
                } else {
                    i += 1;
                }
            } else if b != BELL_CHAR {
                if b == NEWLINE_CHAR {
                    self.cursor.move_to(CursorDirection::NextLine);
                } else if b == CR_CHAR {
                    self.cursor.move_to(CursorDirection::BOL);
                } else if b == TAB_CHAR {
                    let next = (1 + self.cursor.col / TAB_STOP) * TAB_STOP;
                    for _ in 0..(next - self.cursor.col) {
                        self.buffer.set_char_at(SPACE_CHAR as u8, self.cursor.row, self.cursor.col);
                        self.cursor.move_to(CursorDirection::Right);
                    }
                } else {
                    self.buffer.set_char_at(b as u8, self.cursor.row, self.cursor.col);
                    self.cursor.move_to(CursorDirection::Right);
                }
                if self.cursor.row >= self.start_line + TERMINAL_ROWS as usize {
                    self.start_line = 1 + self.cursor.row - TERMINAL_ROWS as usize;
                }
            }
            i += 1;
        }
    }

    pub fn put_char(&mut self, c: &str, color: [f32; 4], row: f32, col: f32) {
        let cell_width = self.cell_size.width() * self.scale_factor;
        let cell_height = self.cell_size.height() * self.scale_factor;
        let x = col * cell_width;
        let y = row * cell_height;
        self.glyph_brush.queue(Section {
            screen_position: (x, self.scale_factor * TITLEBAR_MARGIN + y),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(c)
                .with_color(color)
                .with_scale(cell_height)],
                ..Section::default()
        });
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render encoder") });

        let (origin_width, origin_height) = (
            self.size.width as f32 / self.scale_factor / 2.0,
            self.size.height as f32 / self.scale_factor / 2.0,
        );

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u16> = vec![];

        for row in 0..TERMINAL_ROWS {
            for col in 0..TERMINAL_COLS {
                let i = (row * TERMINAL_COLS + col) as f32;
                let [x, y] = [col as f32 * self.cell_size.width(), row as f32 * self.cell_size.height() + TITLEBAR_MARGIN];
                let [width, height] = [self.cell_size.width(), self.cell_size.height()];
                let v_top_left = Vertex {
                    position: [
                        (x - origin_width) / origin_width,
                        (origin_height - y) / origin_height,
                        0.0
                    ],
                    color: CELL_COLOR
                };
                let v_top_right = Vertex {
                    position: [
                        (x + width - origin_width) / origin_width,
                        (origin_height - y) / origin_height,
                        0.0
                    ],
                    color: CELL_COLOR
                };
                let v_bottom_right = Vertex {
                    position: [
                        (x + width - origin_width) / origin_width,
                        (origin_height - (y + height)) / origin_height,
                        0.0
                    ],
                    color: CELL_COLOR
                };
                let v_bottom_left = Vertex {
                    position: [
                        (x - origin_width) / origin_width,
                        (origin_height - (y + height)) / origin_height,
                        0.0
                    ],
                    color: CELL_COLOR
                };

                vertices.append(&mut vec![ v_top_left, v_top_right, v_bottom_right, v_bottom_left ]);
                let idx = i as u16 * 4;
                indices.append(&mut vec![ idx + 0, idx + 2, idx + 1, idx + 2, idx + 0, idx + 3 ]);
            }
        }

        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX
        });

        let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            render_pass.set_pipeline(&self.quad_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        for row in 0..TERMINAL_ROWS as usize {
            for col in 0..TERMINAL_COLS as usize {
                let c = self.buffer.get_char_at(row + self.start_line, col) as char;
                self.put_char(&c.to_string(), CHR_COLOR, row as f32, col as f32);
            }
        }

        self.put_char(CUR_CHAR, CUR_COLOR, (self.cursor.row - self.start_line) as f32, self.cursor.col as f32);

        self.glyph_brush.draw_queued(&self.device, &mut self.staging_belt, &mut encoder, &view, self.size.width, self.size.height).ok();
        self.staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

