use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    audio::{RodioAudio, SoundCatalog},
    game,
    input::{InputState, MoveAxis},
    render::GpuRenderer,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<GpuRenderer>,
    world: game::World,
    input: InputState,
    next_frame: Option<Instant>,
    audio: Option<RodioAudio>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Rust Shooting")
                        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 320.0)),
                )
                .unwrap_or_else(|error| {
                    eprintln!("failed to create window: {error}");
                    event_loop.exit();
                    std::process::exit(1);
                }),
        );
        let renderer = match pollster::block_on(GpuRenderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("failed to initialize GPU renderer: {error}");
                event_loop.exit();
                return;
            }
        };
        self.renderer = Some(renderer);
        self.window = Some(window);
        let sounds = SoundCatalog::load_at_start(&[
            "assets/audio/fire.wav",
            "assets/audio/hit.wav",
            "assets/audio/destroy.wav",
            "assets/audio/damage.wav",
        ]);
        self.audio = RodioAudio::new(sounds).ok();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event.state, event.physical_key)
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if self.next_frame.is_none_or(|deadline| now >= deadline) {
                    game::update(&mut self.world, self.input);
                    if let Some(audio) = &self.audio {
                        for event in self.world.drain_audio_events() {
                            audio.play_event(event);
                        }
                    } else {
                        self.world.drain_audio_events();
                    }
                    self.input.start = false;
                    self.input.quit = false;
                    self.next_frame = Some(now + frame_duration());
                }
                if let Some(renderer) = &mut self.renderer {
                    match renderer.render(&self.world) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(window) = &self.window {
                                renderer.resize(window.inner_size());
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("GPU surface is out of memory");
                            event_loop.exit();
                        }
                        Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Timeout) => {}
                        Err(wgpu::SurfaceError::Other) => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let deadline = self.next_frame.unwrap_or_else(Instant::now);
        event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        if Instant::now() >= deadline {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

impl App {
    fn handle_key(&mut self, state: ElementState, key: PhysicalKey) {
        match key {
            PhysicalKey::Code(KeyCode::Space) if state == ElementState::Pressed => {
                self.input.start = true
            }
            PhysicalKey::Code(KeyCode::Escape) if state == ElementState::Pressed => {
                self.input.quit = true
            }
            PhysicalKey::Code(KeyCode::ArrowLeft) => self
                .input
                .set_move_key(MoveAxis::Left, state == ElementState::Pressed),
            PhysicalKey::Code(KeyCode::ArrowRight) => self
                .input
                .set_move_key(MoveAxis::Right, state == ElementState::Pressed),
            PhysicalKey::Code(KeyCode::ArrowUp) => self
                .input
                .set_move_key(MoveAxis::Up, state == ElementState::Pressed),
            PhysicalKey::Code(KeyCode::ArrowDown) => self
                .input
                .set_move_key(MoveAxis::Down, state == ElementState::Pressed),
            _ => {}
        }
    }
}

const fn frame_duration() -> Duration {
    Duration::from_nanos(33_333_333)
}
