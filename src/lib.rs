use android_activity::{MainEvent, PollEvent};
use crossbeam::channel::{unbounded, Receiver, Sender};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::core::{mouse, renderer, Font, Pixels, Size, Theme};
use iced_winit::runtime::{program, Debug};
use iced_winit::{conversion, winit};
use lazy_static::lazy_static;
use log::{debug, error, LevelFilter};
use log::{info, warn};
use midi_control::{ControlEvent, KeyEvent, MidiMessage};
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{spawn, JoinHandle};
use std::{
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
    sync::Mutex,
    time::{Duration, SystemTime},
};
use stepper_synth_backend::synth_engines::SynthModule;
use synth::{make_synth, TabSynth};
// use stepper_synth_backend::synth_engines::organ::organ::Organ;
use stepper_synth_backend::{
    synth_engines::{Synth, SynthEngine},
    SampleGen, CHANNEL_SIZE, SAMPLE_RATE,
};
use stepper_synth_backend::{KnobCtrl, MidiControlled};
// use synth::make_synth;
use wgpu::{Device, Instance, Queue, TextureFormat};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::{Window, WindowId};

mod clipboard;
mod controls;
mod java;
mod scene;

lazy_static! {
    // pub static ref TAB_SYNTH: Arc<Mutex<Option<synth::TabSynth>>> = Arc::new(Mutex::new(None));
    pub static ref CBEAM_CHANNELS: (Sender<MidiMessage>, Receiver<MidiMessage>) = unbounded();
    pub static ref MIDI_SEND: Sender<MidiMessage> = CBEAM_CHANNELS.0.clone();
    pub static ref MIDI_RECV: Receiver<MidiMessage> = CBEAM_CHANNELS.1.clone();
}
// pub static TAB_SYNTH: Arc<Mutex<Option<synth::TabSynth>>> = Arc::new(Mutex::new(None));
pub static MIDI_DEVS: RwLock<Vec<String>> = RwLock::new(Vec::new());

use clipboard::Clipboard;
use controls::Controls;
use scene::Scene;
// #[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android;
pub mod synth;

// winit ime support
// https://github.com/rust-windowing/winit/pull/2993

// issue with android-activity crate default_motion_filter function
// https://github.com/rust-mobile/android-activity/issues/79

#[no_mangle]
fn android_main(android_app: AndroidApp) {
    let logger_config = android_logger::Config::default().with_max_level(LevelFilter::Info);
    android_logger::init_once(logger_config);

    log::info!("android_main started");

    let event_loop = EventLoop::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("Should build event loop");

    log::info!("eventloop made");

    let proxy = event_loop.create_proxy();

    log::info!("proxy event loop made");

    // needed bc audio output will fail if its started too soon.
    // TAB_SYNTH.lock().unwrap().replace(make_synth());
    let (synth, output_dev) = make_synth();
    // let synth = Organ::new();
    log::info!("synth made");

    let synth = Arc::new(RwLock::new(synth));
    log::info!("synth stored in a mutex/rw_lock");

    let mut app = App::new(proxy, synth.clone());

    log::info!("app made");

    let _jh = spawn({
        let synth = synth.clone();
        log::info!("synth cloned");

        move || {
            while let Ok(msg) = MIDI_RECV.recv() {
                if let Ok(ref mut tab_synth) = synth.write() {
                    if let Ok(ref mut synth) = tab_synth.synth.write() {
                        // synth.midi_input(&msg);
                        match msg {
                            MidiMessage::Invalid => {
                                error!("system received an invalid MIDI message.");
                            }
                            MidiMessage::NoteOn(_, KeyEvent { key, value }) => {
                                debug!("playing note: {key}");
                                synth.engine.play(key, value)
                            }
                            MidiMessage::NoteOff(_, KeyEvent { key, value: _ }) => {
                                synth.engine.stop(key)
                            }
                            MidiMessage::PitchBend(_, lsb, msb) => {
                                let bend =
                                    i16::from_le_bytes([lsb, msb]) as f32 / (32_000.0 * 0.5) - 1.0;

                                if bend > 0.02 || bend < -0.020 {
                                    synth.engine.bend(bend);
                                    // send();
                                } else {
                                    synth.engine.unbend();
                                    // send();
                                }
                            }
                            MidiMessage::ControlChange(_, ControlEvent { control, value }) => {
                                let value = value as f32 / 127.0;
                                // let effects = self.target_effects;

                                match synth.engine {
                                    SynthModule::WaveTable(ref mut wt) => {
                                        wt.synth.midi_input(&msg);
                                    }
                                    ref mut engine => {
                                        match control {
                                            // 70 if effects => self.get_effect().knob_1(value),
                                            // 71 if effects => self.get_effect().knob_2(value),
                                            // 72 if effects => self.get_effect().knob_3(value),
                                            // 73 if effects => self.get_effect().knob_4(value),
                                            // 70 if !effects => self.get_engine().knob_1(value),
                                            // 71 if !effects => self.get_engine().knob_2(value),
                                            // 72 if !effects => self.get_engine().knob_3(value),
                                            // 73 if !effects => self.get_engine().knob_4(value),
                                            70 => engine.knob_1(value),
                                            71 => engine.knob_2(value),
                                            72 => engine.knob_3(value),
                                            73 => engine.knob_4(value),
                                            74 => engine.knob_5(value),
                                            75 => engine.knob_6(value),
                                            76 => engine.knob_7(value),
                                            77 => engine.knob_8(value),
                                            1 => engine.volume_swell(value),
                                            _ => {
                                                // info!("CC message => {control}-{value}");
                                                false
                                            }
                                        };
                                    }
                                }
                                // if self.engine_type == SynthEngineType::WaveTable {
                                // } else {
                                // }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });

    log::info!("starting main event loop...");

    event_loop.run_app(&mut app).expect("Should run event loop");
}

#[derive(Debug)]
enum UserEvent {
    ShowKeyboard,
    HideKeyboard,
    Tick,
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    app_data: Option<AppData>,
    resized: bool,
    cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    modifiers: ModifiersState,
    value: AtomicU32,
    running: Arc<AtomicBool>,
    synth: Arc<RwLock<TabSynth>>,
}

struct AppData {
    state: program::State<Controls>,
    scene: Scene,
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: wgpu::Surface<'static>,
    format: TextureFormat,
    engine: Engine,
    renderer: Renderer,
    clipboard: Clipboard,
    viewport: Viewport,
    debug: Debug,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>, synth: Arc<RwLock<TabSynth>>) -> Self {
        Self {
            proxy,
            app_data: None,
            resized: false,
            cursor_position: None,
            modifiers: ModifiersState::default(),
            value: AtomicU32::new(0),
            running: Arc::new(AtomicBool::new(false)),
            synth,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        // log::info!("New events cause {:?}", cause);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");
        if self.app_data.is_some() {
            log::info!("Already initialized, skipping");
            return;
        }

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let attrs = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let clipboard = Clipboard {};

        let surface = instance
            .create_surface(window.clone())
            .expect("Create window surface");

        let (format, adapter, device, queue) = futures::executor::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("Create adapter");

            let adapter_features = adapter.features();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: adapter_features & wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::MemoryUsage,
                    },
                    None,
                )
                .await
                .expect("Request device");

            (
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(wgpu::TextureFormat::is_srgb)
                    .or_else(|| capabilities.formats.first().copied())
                    .expect("Get preferred format"),
                adapter,
                device,
                queue,
            )
        });

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );

        let scene = Scene::new(&device, format);
        let controls = Controls::new(self.proxy.clone(), self.synth.clone());

        let mut debug = Debug::new();
        let engine = Engine::new(&adapter, &device, &queue, format, None);
        let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

        let state =
            program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

        event_loop.set_control_flow(ControlFlow::Wait);

        self.cursor_position = None;
        self.modifiers = ModifiersState::default();

        let app_data = AppData {
            state,
            scene,
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            clipboard,
            viewport,
            debug,
        };
        self.app_data = Some(app_data);

        let event_loop_running = self.running.load(Ordering::SeqCst);
        if event_loop_running {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        let event_proxy = self.proxy.clone();
        let is_running = self.running.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if let Err(_e) = event_proxy.send_event(UserEvent::Tick) {
                is_running.store(false, Ordering::SeqCst);
                break;
            }
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ShowKeyboard => {
                java::call_instance_method("showKeyboard");
            }
            UserEvent::HideKeyboard => {
                java::call_instance_method("hideKeyboard");
            }
            UserEvent::Tick => {
                let value = self.value.fetch_add(1, Ordering::SeqCst);
                // log::info!("Tick event, counter value: {}", value);
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        log::info!("DeviceEvent {:?}", event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        log::info!("Window event: {:?}", event);

        let Some(app_data) = self.app_data.as_mut() else {
            return;
        };

        let AppData {
            state,
            scene,
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            clipboard,
            debug,
            ..
        } = app_data;

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.resized {
                    let size = window.inner_size();

                    app_data.viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    self.resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass =
                                Scene::clear(&view, &mut encoder, program.background_color());
                            scene.draw(&mut render_pass);
                        }

                        renderer.present::<String>(
                            engine,
                            device,
                            queue,
                            &mut encoder,
                            None,
                            frame.texture.format(),
                            &view,
                            &app_data.viewport,
                            &[],
                        );

                        engine.submit(queue, encoder);
                        frame.present();

                        window.set_cursor(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                            Rendering cannot continue."
                            )
                        }
                        _ => {
                            window.request_redraw();
                        }
                    },
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
            }
            WindowEvent::Touch(touch) => {
                self.cursor_position = Some(touch.location);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                ref event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => match event.state {
                            ElementState::Pressed => self.modifiers |= ModifiersState::SHIFT,
                            ElementState::Released => self.modifiers &= !ModifiersState::SHIFT,
                        },
                        KeyCode::ControlLeft | KeyCode::ControlRight => match event.state {
                            ElementState::Pressed => self.modifiers |= ModifiersState::CONTROL,
                            ElementState::Released => self.modifiers &= !ModifiersState::CONTROL,
                        },
                        _ => (),
                    }
                }
            }
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            _ => (),
        }

        if let Some(event) =
            iced_winit::conversion::window_event(event, window.scale_factor(), self.modifiers)
        {
            state.queue_event(event);
        }

        if !state.is_queue_empty() {
            let _ = state.update(
                app_data.viewport.logical_size(),
                self.cursor_position
                    .map(|p| conversion::cursor_position(p, app_data.viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                renderer,
                &Theme::Ferra,
                &renderer::Style {
                    text_color: Theme::Ferra.palette().text,
                },
                clipboard,
                debug,
            );

            window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

// static SERIAL_THREAD: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);
// static FLAG_EXIT: Mutex<bool> = Mutex::new(false);
