use characters::InputChar;
use wgpu::SurfaceError;
use winit::{event::{ElementState, Event, KeyboardInput, ModifiersState, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::macos::WindowBuilderExtMacOS, window::WindowBuilder};

mod characters;
mod frontend;
mod backend;
use frontend::AppFrontend;
use backend::{AppBackend, CustomEvent};

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

    let mut modifiers: ModifiersState = ModifiersState::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::UserEvent(event) => {
            match event {
                CustomEvent::StdOut(mut data) => {
                    frontend.set_data(&mut data);
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
                frontend.resize(*physical_size, -1.0);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor, .. } => {
                frontend.resize(**new_inner_size, *scale_factor as f32);
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
                        backend.send(&[c as u8]);
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
