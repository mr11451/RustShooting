mod app;
mod assets;
mod audio;
mod background;
mod collision;
mod data;
mod fixed;
mod game;
mod input;
mod render;
mod runtime;
mod sprite;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");
    let mut app = app::App::default();
    event_loop.run_app(&mut app).expect("event loop failed");
}
