use std::thread;
use portable_pty::{CommandBuilder, PtyPair, PtySize, native_pty_system};
use winit::event_loop::EventLoopProxy;

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
