use wgpu::{Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor, util::StagingBelt};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Text, ab_glyph};
use winit::{dpi::PhysicalSize, event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};

struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    glyph_brush: GlyphBrush<()>,
    staging_belt: StagingBelt,
    data: String
}

impl State {
    async fn new(window: &Window) -> Self {
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
            data: "Press any key".to_string()
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, key_state: &ElementState, key: &VirtualKeyCode) {
        self.data = format!("Key {:?} {:?}", key, key_state);
    }

    fn update(&mut self) {
        // nothing yet
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: true
                    }
                }],
                depth_stencil_attachment: None
            });
        }

        self.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(&self.data)
                .with_color([0.0, 0.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });

        self.glyph_brush.draw_queued(&self.device, &mut self.staging_belt, &mut encoder, &view, self.size.width, self.size.height).ok();
        self.staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            },
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: key_state,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => { state.input(key_state, key); },
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {},
                Err(SurfaceError::Lost) => state.resize(state.size),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e)
            }
        }
        _ => {}
    });
}
