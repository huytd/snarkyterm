use characters::InputChar;
use wgpu::SurfaceError;
use winit::{event::{ElementState, Event, KeyboardInput, ModifiersState, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::macos::WindowBuilderExtMacOS, window::WindowBuilder};

mod constants;
mod cursor;
mod characters;
mod terminal;
mod device;

use terminal::Terminal;
use device::{Shell, CustomEvent};

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

    let mut terminal = pollster::block_on(Terminal::new(&window));
    let mut shell = Shell::new(proxy);

    let mut modifiers: ModifiersState = ModifiersState::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::UserEvent(event) => {
            match event {
                CustomEvent::StdOut(mut data) => {
                    terminal.set_data(&mut data);
                },
                CustomEvent::Terminate => {
                    *control_flow = ControlFlow::Exit;
                }
            }
        },
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                terminal.resize(*physical_size, -1.0);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor, .. } => {
                terminal.resize(**new_inner_size, *scale_factor as f32);
            },
            WindowEvent::ModifiersChanged(current_modifiers) => modifiers = *current_modifiers,
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: key_state,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => {
                if *key_state == ElementState::Pressed {
                    if let Some(c) = InputChar::from(*key, modifiers) {
                        shell.send(&[c as u8]);
                    }
                }
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            match terminal.render() {
                Ok(_) => {},
                Err(SurfaceError::Lost) => terminal.resize(terminal.size, -1.0),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e)
            }
        }
        _ => {}
    });
}
