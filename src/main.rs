use std::{thread::{self, sleep}, time::Duration};

use portable_pty::{CommandBuilder, PtyPair, PtySize, native_pty_system};
use wgpu::SurfaceError;
use winit::{event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop, EventLoopProxy}, platform::macos::WindowBuilderExtMacOS, window::WindowBuilder};

mod frontend;
use frontend::AppFrontend;

#[derive(Debug, Clone)]
pub enum CustomEvent {
    StdOut(String)
}

pub struct AppBackend {
    pair: PtyPair
}

impl AppBackend {
    pub fn new(proxy: EventLoopProxy<CustomEvent>) -> Self {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0
        }).unwrap();
        let cmd = CommandBuilder::new("/bin/bash");
        let _child = pair.slave.spawn_command(cmd).unwrap();

        let mut reader = pair.master.try_clone_reader().unwrap();
        let sender = proxy.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 128];
            while let Ok(len) = reader.read(&mut buf) {
                if len == 0 {
                    break;
                }
                if let Ok(chunk) = String::from_utf8(buf.to_vec()) {
                    sender.send_event(CustomEvent::StdOut(chunk)).ok();
                    buf = [0u8; 128];
                }
            }
        });

        Self {
            pair
        }
    }

    pub fn send(&mut self, data: &str) {
        write!(self.pair.master, "{}", data).unwrap();
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::<CustomEvent>::with_user_event();
    let window = WindowBuilder::new()
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
        .with_title_hidden(true)
        .build(&event_loop)
        .unwrap();

    let proxy = event_loop.create_proxy();

    let mut frontend = pollster::block_on(AppFrontend::new(&window));
    let mut backend = AppBackend::new(proxy);

    event_loop.run(move |event, _, control_flow| match event {
        Event::UserEvent(event) => {
            match event {
                CustomEvent::StdOut(data) => {
                    frontend.append_data(&data);
                }
            }
        },
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                frontend.resize(*physical_size, -1.0);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor, .. } => {
                frontend.resize(**new_inner_size, *scale_factor as f32);
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
            } => {
                if *key_state == ElementState::Pressed {
                    if *key == VirtualKeyCode::Space {
                        backend.send(" ");
                    } else if *key == VirtualKeyCode::Return {
                        backend.send("\r");
                    } else {
                        let c = format!("{:?}", key).to_lowercase();
                        backend.send(&c);
                    }
                }
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            match frontend.render() {
                Ok(_) => {},
                Err(SurfaceError::Lost) => frontend.resize(frontend.size, -1.0),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e)
            }
        }
        _ => {}
    });
}
