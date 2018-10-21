#[macro_use]
extern crate ash;
extern crate winit;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate metal_rs as metal;
#[cfg(target_os = "macos")]
extern crate objc;

mod application;
mod engine;
mod shader;

use application::Application;

fn main() {
    let mut app = Application::new();

    app.run().unwrap();
}
