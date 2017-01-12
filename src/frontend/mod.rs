pub mod fps;

use time;

use sdl2;
use std::sync::{Arc, Mutex};
use sdl2::Sdl;
use sdl2::EventPump;
use sdl2::VideoSubsystem;
use std::time::{Duration, Instant};
use std::thread;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::controller::{Axis, Button};

use std::error::Error;
use std::fmt;
use std::path::Path;

use chrono::Local;

#[macro_use]
use chan;
use chan::{Receiver, Sender};

use renderer;
use px8;
use config;
use gfx::{Scale};

struct FrameTimes {
    frame_duration: Duration,
    last_time: Instant,
    target_time: Instant,
}

impl FrameTimes {
    pub fn new(frame_duration: Duration) -> FrameTimes {
        let now = Instant::now();
        FrameTimes {
            frame_duration: frame_duration,
            last_time: now,
            target_time: now + frame_duration,
        }
    }

    pub fn reset(&mut self) {
        let now = Instant::now();
        self.last_time = now;
        self.target_time = now + self.frame_duration;
    }

    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now - self.last_time;
        self.last_time = now;
        self.target_time += self.frame_duration;
        delta
    }

    pub fn limit(&self) {
        let now = Instant::now();
        if now < self.target_time {
            thread::sleep(self.target_time - now);
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum PX8Key {
    Right, Left, Up, Down, O, X, Pause, Enter
}

impl fmt::Debug for PX8Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PX8Key::*;

        write!(f, "{}", match *self {
            Right => "RIGHT", Left => "LEFT", Up => "UP", Down => "DOWN", O => "O", X => "X", Pause => "Pause", Enter => "Enter"
        })

    }
}

#[derive(Clone, Debug)]
pub enum FrontendError {
    Sdl(String),
    Renderer(String),
    Other(String),
}

pub type FrontendResult<T> = Result<T, FrontendError>;

impl From<sdl2::IntegerOrSdlError> for FrontendError {
    fn from(e: sdl2::IntegerOrSdlError) -> FrontendError {
        FrontendError::Sdl(format!("{:?}", e))
    }
}

impl From<sdl2::video::WindowBuildError> for FrontendError {
    fn from(e: sdl2::video::WindowBuildError) -> FrontendError {
        FrontendError::Renderer(format!("{:?}", e))
    }
}

impl From<String> for FrontendError {
    fn from(e: String) -> FrontendError {
        FrontendError::Other(e)
    }
}

pub struct Channels {
    tx_input: Sender<Vec<u8>>,
    rx_input: Receiver<Vec<u8>>,
    tx_output: Sender<Vec<u8>>,
    rx_output: Receiver<Vec<u8>>,
}

impl Channels {
    pub fn new() -> Channels {
        let (tx_input, rx_input): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = chan::sync(0);
        let (tx_output, rx_output): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = chan::sync(0);

        Channels {
            tx_input: tx_input,
            rx_input: rx_input,
            tx_output: tx_output,
            rx_output: rx_output,
        }
    }
}

pub struct SdlFrontend {
    sdl: Sdl,
    event_pump: EventPump,
    renderer: renderer::renderer::Renderer,
    times: FrameTimes,
    px8: px8::Px8New,
    info: Arc<Mutex<px8::info::Info>>,
    channels: Channels,
    start_time: time::Tm,
    elapsed_time: f64,
    delta: Duration,
    scale: Scale,
}

impl SdlFrontend {
    pub fn init(scale: Scale, fullscreen: bool) -> FrontendResult<SdlFrontend> {
        info!("Frontend: SDL2 init");
        let sdl = try!(sdl2::init());

        info!("Frontend: SDL2 Video init");
        let sdl_video = try!(sdl.video());

        info!("Frontend: SDL2 event pump");
        let event_pump = try!(sdl.event_pump());

        info!("Frontend: creating renderer");
        let renderer = renderer::renderer::Renderer::new(sdl_video, fullscreen, scale).unwrap();

        // Disable mouse in the window
        sdl.mouse().show_cursor(true);

        Ok(SdlFrontend {
            sdl: sdl,
            event_pump: event_pump,
            renderer: renderer,
            times: FrameTimes::new(Duration::from_secs(1) / 60),
            px8: px8::Px8New::new(),
            info: Arc::new(Mutex::new(px8::info::Info::new())),
            channels: Channels::new(),
            start_time: time::now(),
            elapsed_time: 0.,
            delta: Duration::from_secs(0),
            scale: scale,
        })
    }

    pub fn main(&mut self, filename: String, editor: bool) {

        self.start_time = time::now();
        self.times.reset();

        self.run_cartridge(filename, editor);
    }

    pub fn blit(&mut self) {
        self.renderer.blit(&self.px8.screen.lock().unwrap().back_buffer);
        self.times.limit();
    }

    pub fn run_cartridge(&mut self, filename: String, editor: bool) {
        let mut fps_counter = fps::FpsCounter::new();

        let players_input = Arc::new(Mutex::new(config::Players::new()));
        let players_clone = players_input.clone();

        self.px8.init();

        self.px8.load_cartridge(filename.clone(),
                                self.channels.tx_input.clone(),
                                self.channels.rx_output.clone(),
                                players_input,
                                self.info.clone(),
                                editor);

        info!("Init Game Controller");
        let game_controller_subsystem = self.sdl.game_controller().unwrap();

        info!("Loading the database of Game Controller");
        info!("-> {:?}", game_controller_subsystem.load_mappings(Path::new("./sys/config/gamecontrollerdb.txt")));

        let available =
        match game_controller_subsystem.num_joysticks() {
            Ok(n) => n,
            Err(e) => panic!("can't enumerate joysticks: {}", e),
        };

        info!("{} joysticks available", available);

        let mut joysticks = Vec::new();
        let mut controllers = Vec::new();

        for id in 0..available {
            if game_controller_subsystem.is_game_controller(id) {
                println!("Attempting to open controller {}", id);

                match game_controller_subsystem.open(id) {
                    Ok(c) => {
                        // We managed to find and open a game controller,
                        // exit the loop
                        info!("Success: opened \"{}\"", c.name());
                        info!("Success: opened \"{}\"", c.mapping());

                        controllers.push(Some(c));
                        break;
                    },
                    Err(e) => error!("failed: {:?}", e),
                }
            } else {
                info!("{} is not a game controller", id);
            }
        }

        let joystick_subsystem = self.sdl.joystick().unwrap();

        let available =
        match joystick_subsystem.num_joysticks() {
            Ok(n) => n,
            Err(e) => panic!("can't enumerate joysticks: {}", e),
        };

        println!("{} joysticks available", available);

        // Iterate over all available joysticks and stop once we manage to
        // open one.
        for id in 0..available {
            match joystick_subsystem.open(id) {
                Ok(c) => {
                    info!("Success: opened \"{}\"", c.name());

                    joysticks.push(Some(c));
                },
                Err(e) => error!("failed: {:?}", e),
            }
        }


        // Call the init of the cartridge
        self.px8.init_time = self.px8.call_init() * 1000.0;

        'main: loop {
            let delta = self.times.update();

            fps_counter.update(self.times.last_time);

            self.px8.fps = fps_counter.get_fps();

            let mouse_state = self.event_pump.mouse_state();
            let (width, height) = self.renderer.get_dimensions();

            players_clone.lock().unwrap().set_mouse_x(mouse_state.x() / (width as i32 / px8::SCREEN_WIDTH as i32));
            players_clone.lock().unwrap().set_mouse_y(mouse_state.y() / (height as i32 / px8::SCREEN_HEIGHT as i32));
            players_clone.lock().unwrap().set_mouse_state(mouse_state);

            if mouse_state.left() {
                debug!("MOUSE X {:?} Y {:?}",
                      mouse_state.x() / (width as i32 / px8::SCREEN_WIDTH as i32),
                      mouse_state.y() / (height as i32 / px8::SCREEN_HEIGHT as i32));
            }

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
                    Event::KeyDown { keycode: Some(keycode), .. } if keycode == Keycode::Escape => break 'main,
                    Event::Window { win_event: WindowEvent::SizeChanged(_, _), .. } => {
                        self.renderer.update_dimensions();
                    },
                    Event::KeyDown { keycode: Some(keycode), repeat, .. } => {
                        if let (Some(key), player) = map_keycode(keycode) {
                            players_clone.lock().unwrap().key_down(player, key, repeat, self.elapsed_time);
                        }

                        if keycode == Keycode::F2 {
                            self.px8.toggle_info_overlay();
                        } else if keycode == Keycode::F3 {
                            let dt = Local::now();
                            self.px8.screenshot("screenshot-".to_string() + &dt.format("%Y-%m-%d-%H-%M-%S.png").to_string());
                        } else if keycode == Keycode::F4 {
                            let record_screen = self.px8.is_recording();
                            if ! record_screen {
                                let dt = Local::now();
                                self.px8.start_record("record-".to_string() + &dt.format("%Y-%m-%d-%H-%M-%S.gif").to_string());
                            } else {
                                self.px8.stop_record(self.scale.factor());
                            }
                        } else if keycode == Keycode::F5 {
                            if editor {
                                let dt = Local::now();
                                self.px8.save_current_cartridge(dt.format("%Y-%m-%d-%H-%M-%S").to_string());
                            }
                        } else if keycode == Keycode::F6 && editor {
                            self.px8.switch_code(filename.clone());
                            // Call the init of the new code
                            self.px8.init_time = self.px8.call_init() * 1000.0;
                        }

                        let pause = players_clone.lock().unwrap().get_value_quick(0, 7) == 1;
                        if pause {
                            self.px8.pause();
                        }
                    },
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        if let (Some(key), player) = map_keycode(keycode) { players_clone.lock().unwrap().key_up(player, key) }
                    },

                    Event::ControllerDeviceAdded { which: id, .. } => {
                        info!("New Controller detected {:?}", id);
                    },

                    Event::ControllerButtonDown { which: id, button, .. } => {
                        info!("Controller button Down {:?} {:?}", id, button);
                        if let Some(key) = map_button(button) { players_clone.lock().unwrap().key_down(0, key, false, self.elapsed_time) }
                    },

                    Event::ControllerButtonUp { which: id, button, .. } => {
                        info!("Controller button UP {:?} {:?}", id, button);
                        if let Some(key) = map_button(button) { players_clone.lock().unwrap().key_up(0, key) }
                    },

                    Event::ControllerAxisMotion { which: id, axis, value, .. } => {
                        info!("Controller Axis Motion {:?} {:?} {:?}", id, axis, value);

                        if let Some((key, state)) = map_axis(axis, value) {
                            info!("Key {:?} State {:?}", key, state);


                            if axis == Axis::LeftX && value == 128 {
                                players_clone.lock().unwrap().key_direc_hor_up(0);
                            } else if axis == Axis::LeftY && value == -129 {
                                players_clone.lock().unwrap().key_direc_ver_up(0);
                            } else {
                                if state {
                                    players_clone.lock().unwrap().key_down(0, key, false, self.elapsed_time)
                                } else {
                                    players_clone.lock().unwrap().key_up(0, key)
                                }
                            }
                        }
                    },

                    Event::JoyAxisMotion { which: id, axis_idx, value: val, .. } => {
                        info!("Joystick Axis Motion {:?} {:?} {:?}", id, axis_idx, val);
                    },

                    Event::JoyButtonDown { which: id, button_idx, .. } => {
                        info!("Joystick button DOWN {:?} {:?}", id, button_idx);
                        // if let Some(key) = map_button(button) { players_clone.lock().unwrap().key_up(0, key) }
                    },

                    Event::JoyButtonUp { which: id, button_idx, .. } => {
                        info!("Joystick Button {:?} {:?} up", id, button_idx);
                    },

                    Event::JoyHatMotion { which: id, hat_idx, state, .. } => {
                        info!("Joystick Hat {:?} {:?} moved to {:?}", id, hat_idx, state);
                    },

                    _ => (),
                }
            }

            match self.px8.state {
                px8::PX8State::PAUSE => {
                    let up = players_clone.lock().unwrap().get_value_quick(0, 2) == 1;
                    let down = players_clone.lock().unwrap().get_value_quick(0, 3) == 1;
                    let enter = players_clone.lock().unwrap().get_value_quick(0, 6) == 1;

                    self.px8.update_pause(enter, up, down);

                    self.px8.update();
                },
                px8::PX8State::RUN => {
                    if self.px8.is_end() {
                        break 'main;
                    }

                    self.px8.update_time = self.px8.call_update() * 1000.0;
                    self.px8.draw_time = self.px8.call_draw() * 1000.0;

                    self.px8.update();

                    if self.px8.is_recording() {
                        self.px8.record();
                    }
                }
            }

            self.update_time(players_clone.clone());
            self.blit();
        }
    }

    pub fn update_time(&mut self, players: Arc<Mutex<config::Players>>) {
        let new_time = time::now();
        let diff_time = new_time - self.start_time;
        let nanoseconds = (diff_time.num_nanoseconds().unwrap() as f64) - (diff_time.num_seconds() * 1000000000) as f64;

        let elapsed_time = diff_time.num_seconds() as f64 + nanoseconds / 1000000000.0;

        self.info.lock().unwrap().elapsed_time = elapsed_time;

        players.lock().unwrap().update(elapsed_time);
    }

    #[cfg(target_os = "emscripten")]
    pub fn demo(&mut self) {
        let mut fps_counter = fps::FpsCounter::new();

        self.start_time = time::now();
        self.times.reset();

        self.px8.init();

        emscripten_loop::set_main_loop_callback(|| {
            let delta = self.times.update();

            fps_counter.update(self.times.last_time);

            self.px8.fps = fps_counter.get_fps();

            for event in self.event_pump.poll_iter() {
                // info!("EVENT {:?}", event);

                match event {
                   // Event::Quit { .. } => break 'main,
                    _ => (),
                }
            }

            self.px8.update();

            self.blit();
        });
    }

    #[cfg(not(target_os = "emscripten"))]
    pub fn demo(&mut self) {
        let mut fps_counter = fps::FpsCounter::new();

        self.start_time = time::now();
        self.times.reset();

        self.px8.init();

        'main: loop {
            let delta = self.times.update();

            fps_counter.update(self.times.last_time);

            self.px8.fps = fps_counter.get_fps();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
                    _ => (),
                }
            }

            self.px8.update();

            self.blit();
        }
    }
}

#[cfg(target_os = "emscripten")]
pub mod emscripten_loop {
    use std::cell::RefCell;
    use std::ptr::null_mut;
    use std::os::raw::{c_int, c_void};

    #[allow(non_camel_case_types)]
    type em_callback_func = unsafe extern fn();

    extern {
        fn emscripten_set_main_loop(func: em_callback_func, fps: c_int, simulate_infinite_loop: c_int);
    }

    thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

    pub fn set_main_loop_callback<F>(callback: F) where F: FnMut() {
        MAIN_LOOP_CALLBACK.with(|log| {
            *log.borrow_mut() = &callback as *const _ as *mut c_void;
        });

        unsafe { emscripten_set_main_loop(wrapper::<F>, 0, 1); }

        unsafe extern "C" fn wrapper<F>() where F: FnMut() {
            MAIN_LOOP_CALLBACK.with(|z| {
                let closure = *z.borrow_mut() as *mut F;
                (*closure)();
            });
        }
    }
}

fn map_button(button: Button) -> Option<PX8Key> {
    match button {
        Button::DPadRight => Some(PX8Key::Right),
        Button::DPadLeft => Some(PX8Key::Left),
        Button::DPadUp => Some(PX8Key::Up),
        Button::DPadDown => Some(PX8Key::Down),
        Button::A => Some(PX8Key::O),
        Button::B => Some(PX8Key::X),
        _ => None
    }
}

fn map_keycode(key: Keycode) -> (Option<PX8Key>, u8) {
    match key {
        Keycode::Right => (Some(PX8Key::Right), 0),
        Keycode::Left => (Some(PX8Key::Left), 0),
        Keycode::Up => (Some(PX8Key::Up), 0),
        Keycode::Down => (Some(PX8Key::Down), 0),
        Keycode::Z => (Some(PX8Key::O), 0),
        Keycode::C => (Some(PX8Key::O), 0),
        Keycode::N => (Some(PX8Key::O), 0),
        Keycode::X => (Some(PX8Key::X), 0),
        Keycode::V => (Some(PX8Key::X), 0),
        Keycode::M => (Some(PX8Key::X), 0),

        Keycode::F => (Some(PX8Key::Right), 1),
        Keycode::S => (Some(PX8Key::Left), 1),
        Keycode::E => (Some(PX8Key::Up), 1),
        Keycode::D => (Some(PX8Key::Down), 1),

        Keycode::LShift => (Some(PX8Key::O), 1),
        Keycode::Tab => (Some(PX8Key::O), 1),

        Keycode::A => (Some(PX8Key::X), 1),
        Keycode::Q => (Some(PX8Key::X), 1),

        Keycode::P => (Some(PX8Key::Pause), 0),
        Keycode::KpEnter => (Some(PX8Key::Enter), 0),

        _ => (None, 0)
    }
}

fn map_axis(axis: Axis, value: i16) -> Option<(PX8Key, bool)> {
match axis {
Axis::LeftX => match value {
    -32768...-16384 => Some((PX8Key::Left, true)),
    -16383...-1 => Some((PX8Key::Left, false)),
    0...16383 => Some((PX8Key::Right, false)),
    16384...32767 => Some((PX8Key::Right, true)),
    _ => None
},

Axis::LeftY => match value {
    -32768...-16384 => Some((PX8Key::Up, true)),
    -16383...-1 => Some((PX8Key::Up, false)),
    0...16383 => Some((PX8Key::Down, false)),
    16384...32767 => Some((PX8Key::Down, true)),
    _ => None
},
_ => None
}
}