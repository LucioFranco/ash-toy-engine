#[macro_use]
extern crate ash;
extern crate winit;

#[cfg(target_os = "windows")]
extern crate winapi;

mod application;
mod engine;
mod shader;

use application::Application;

fn main() {
    let mut app = Application::new();

    app.run().unwrap();
}
