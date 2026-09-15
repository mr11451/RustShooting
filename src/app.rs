use gilrs::{Axis, Button, EventType, Gilrs};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    audio::{RodioAudio, SoundCatalog},
    data::{LOGICAL_HEIGHT, LOGICAL_WIDTH},
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
    gilrs: Option<Gilrs>,
    paused: bool,
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
                        .with_min_inner_size(winit::dpi::LogicalSize::new(
                            LOGICAL_WIDTH,
                            LOGICAL_HEIGHT,
                        )),
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
        self.gilrs = Gilrs::new().ok();
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
                self.handle_key(event.state, event.physical_key, event.repeat)
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if self.next_frame.is_none_or(|deadline| now >= deadline) {
                    self.poll_gamepad();
                    let prev_state = self.world.state;
                    if !self.paused {
                        game::update(&mut self.world, self.input);
                    }
                    if let Some(audio) = &mut self.audio {
                        if matches!(
                            self.world.state,
                            game::GameState::StageIntro
                                | game::GameState::Playing
                                | game::GameState::StageClear
                        ) {
                            audio.play_stage_bgm(self.world.stage_id);
                        } else if matches!(
                            self.world.state,
                            game::GameState::Title
                                | game::GameState::Demo
                                | game::GameState::NameEntry
                                | game::GameState::GameOver
                                | game::GameState::Ending
                        ) && prev_state != self.world.state
                        {
                            audio.stop_bgm();
                        }
                        for event in self.world.drain_audio_events() {
                            audio.play_event(event);
                        }
                    } else {
                        self.world.drain_audio_events();
                    }
                    self.input.start = false;
                    self.input.fire_trigger = false;
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
        if Instant::now() >= deadline
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
}

impl App {
    fn handle_key(&mut self, state: ElementState, key: PhysicalKey, repeat: bool) {
        let is_pressed = state == ElementState::Pressed;
        match key {
            PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::KeyZ) => {
                self.input
                    .set_keyboard_fire(is_pressed, is_pressed && !repeat);
                if is_pressed && !repeat {
                    self.input.start = true;
                }
            }
            PhysicalKey::Code(KeyCode::Enter) | PhysicalKey::Code(KeyCode::KeyX) => {
                if is_pressed && !repeat {
                    self.input.set_keyboard_fire(true, true);
                    self.input.start = true;
                } else {
                    self.input.set_keyboard_fire(false, false);
                }
            }
            PhysicalKey::Code(KeyCode::Escape) if is_pressed => {
                self.input.quit = true;
            }
            PhysicalKey::Code(KeyCode::KeyP) if is_pressed && !repeat => {
                self.toggle_pause();
            }
            PhysicalKey::Code(KeyCode::ArrowLeft) | PhysicalKey::Code(KeyCode::KeyA) => {
                self.input.set_move_key(MoveAxis::Left, is_pressed);
            }
            PhysicalKey::Code(KeyCode::ArrowRight) | PhysicalKey::Code(KeyCode::KeyD) => {
                self.input.set_move_key(MoveAxis::Right, is_pressed);
            }
            PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                self.input.set_move_key(MoveAxis::Up, is_pressed);
            }
            PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                self.input.set_move_key(MoveAxis::Down, is_pressed);
            }
            _ => {}
        }
    }

    fn poll_gamepad(&mut self) {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return;
        };
        let mut toggle_pause_requested = false;
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(Button::South, _) => {
                    self.input.set_controller_fire(true, true);
                    self.input.start = true;
                }
                EventType::ButtonReleased(Button::South, _) => {
                    self.input.set_controller_fire(false, false);
                }
                EventType::ButtonPressed(Button::Start, _) => {
                    toggle_pause_requested = true;
                }
                EventType::ButtonPressed(Button::DPadLeft, _) => {
                    self.input.set_controller_move_key(MoveAxis::Left, true);
                }
                EventType::ButtonReleased(Button::DPadLeft, _) => {
                    self.input.set_controller_move_key(MoveAxis::Left, false);
                }
                EventType::ButtonPressed(Button::DPadRight, _) => {
                    self.input.set_controller_move_key(MoveAxis::Right, true);
                }
                EventType::ButtonReleased(Button::DPadRight, _) => {
                    self.input.set_controller_move_key(MoveAxis::Right, false);
                }
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    self.input.set_controller_move_key(MoveAxis::Up, true);
                }
                EventType::ButtonReleased(Button::DPadUp, _) => {
                    self.input.set_controller_move_key(MoveAxis::Up, false);
                }
                EventType::ButtonPressed(Button::DPadDown, _) => {
                    self.input.set_controller_move_key(MoveAxis::Down, true);
                }
                EventType::ButtonReleased(Button::DPadDown, _) => {
                    self.input.set_controller_move_key(MoveAxis::Down, false);
                }
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    self.input.set_controller_axis(MoveAxis::Right, value);
                    self.input.set_controller_axis(MoveAxis::Left, -value);
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    self.input.set_controller_axis(MoveAxis::Down, value);
                    self.input.set_controller_axis(MoveAxis::Up, -value);
                }
                _ => {}
            }
        }
        if toggle_pause_requested {
            if self.world.state == game::GameState::Playing {
                self.toggle_pause();
            } else {
                self.input.start = true;
            }
        }
    }

    fn toggle_pause(&mut self) {
        if self.world.state == game::GameState::Playing {
            self.paused = !self.paused;
        }
    }
}

const TARGET_FPS: u64 = 60;

const fn frame_duration() -> Duration {
    Duration::from_nanos(1_000_000_000 / TARGET_FPS)
}
