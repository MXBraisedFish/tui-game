//! 键盘监听：融合 crossterm 与 rdev

use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use once_cell::sync::Lazy;
use rdev::{Event as RdevEvent, EventType as RdevEventType, Key as RdevKey, listen};

use super::ctrl_c_handler;
use super::input_event::HostInputEvent;
use super::resize_watcher::ResizeEvent;

const CROSSTERM_POLL_INTERVAL_MS: u64 = 16;
const RDEV_DELAY_MS: u64 = 20;
const SUPPRESS_WINDOW_MS: u64 = 120;
const RDEV_DRAIN_LIMIT: usize = 16;

static ALLOWED_CROSSTERM_KEYCODES: Lazy<HashSet<KeyCode>> = Lazy::new(allowed_crossterm_keycodes);

#[derive(Debug, Clone)]
struct DelayedRdevEvent {
    key: RdevKey,
    timestamp: Instant,
}

#[derive(Default)]
struct SharedKeyState {
    delayed_rdev_events: VecDeque<DelayedRdevEvent>,
    last_crossterm_output: Option<(String, Instant)>,
}

/// 键盘监听句柄
pub struct KeyListener {
    is_running: Arc<AtomicBool>,
    crossterm_thread: Option<JoinHandle<()>>,
}

/// 启动键盘监听
pub fn start(
    resize_sender: Sender<ResizeEvent>,
) -> Result<(KeyListener, Receiver<HostInputEvent>), Box<dyn std::error::Error>> {
    let (input_sender, input_receiver) = mpsc::channel();
    let is_running = Arc::new(AtomicBool::new(true));
    let shared_key_state = Arc::new(Mutex::new(SharedKeyState::default()));

    start_rdev_listener(Arc::clone(&shared_key_state));
    let crossterm_thread = start_crossterm_listener(
        Arc::clone(&is_running),
        Arc::clone(&shared_key_state),
        input_sender,
        resize_sender,
    );

    Ok((
        KeyListener {
            is_running,
            crossterm_thread: Some(crossterm_thread),
        },
        input_receiver,
    ))
}

impl KeyListener {
    /// 监听线程是否仍处于运行状态
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

impl Drop for KeyListener {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(crossterm_thread) = self.crossterm_thread.take() {
            let _ = crossterm_thread.join();
        }
    }
}

fn start_crossterm_listener(
    is_running: Arc<AtomicBool>,
    shared_key_state: Arc<Mutex<SharedKeyState>>,
    input_sender: Sender<HostInputEvent>,
    resize_sender: Sender<ResizeEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while is_running.load(Ordering::Relaxed) {
            if let Ok(true) = event::poll(Duration::from_millis(CROSSTERM_POLL_INTERVAL_MS)) {
                match event::read() {
                    Ok(Event::Key(key_event)) => {
                        handle_crossterm_key_event(key_event, &shared_key_state, &input_sender);
                    }
                    Ok(Event::Resize(width, height)) => {
                        let resize_event = ResizeEvent { width, height };
                        let _ = resize_sender.send(resize_event);
                        let _ = input_sender.send(HostInputEvent::Resize(resize_event));
                    }
                    Ok(_) | Err(_) => {}
                }
            }

            drain_ready_rdev_keys(&shared_key_state, &input_sender, RDEV_DRAIN_LIMIT);
        }
    })
}

fn start_rdev_listener(shared_key_state: Arc<Mutex<SharedKeyState>>) {
    thread::spawn(move || {
        let _ = listen(move |event: RdevEvent| {
            if let Ok(mut key_state) = shared_key_state.lock() {
                if let RdevEventType::KeyPress(key) = event.event_type {
                    key_state.delayed_rdev_events.push_back(DelayedRdevEvent {
                        key,
                        timestamp: Instant::now(),
                    });
                }
            }
        });
    });
}

fn handle_crossterm_key_event(
    key_event: KeyEvent,
    shared_key_state: &Arc<Mutex<SharedKeyState>>,
    input_sender: &Sender<HostInputEvent>,
) {
    if !matches!(key_event.kind, KeyEventKind::Press) {
        return;
    }

    if ctrl_c_handler::is_ctrl_c(&key_event) {
        let _ = input_sender.send(HostInputEvent::ExitRequested);
        return;
    }

    if !ALLOWED_CROSSTERM_KEYCODES.contains(&key_event.code) {
        return;
    }

    let Some(semantic_key) = map_crossterm_key_to_semantic(key_event.code) else {
        return;
    };

    if let Ok(mut key_state) = shared_key_state.lock() {
        key_state.last_crossterm_output = Some((semantic_key.clone(), Instant::now()));
    }
    let _ = input_sender.send(HostInputEvent::Key { key: semantic_key });
}

fn drain_ready_rdev_keys(
    shared_key_state: &Arc<Mutex<SharedKeyState>>,
    input_sender: &Sender<HostInputEvent>,
    limit: usize,
) {
    let now = Instant::now();
    let mut output_keys = Vec::new();
    let mut key_state = match shared_key_state.lock() {
        Ok(key_state) => key_state,
        Err(_) => return,
    };

    while output_keys.len() < limit {
        let Some(delayed_event) = key_state.delayed_rdev_events.front() else {
            break;
        };
        if now.duration_since(delayed_event.timestamp) < Duration::from_millis(RDEV_DELAY_MS) {
            break;
        }

        let key = delayed_event.key;
        key_state.delayed_rdev_events.pop_front();

        if let Some(semantic_key) = map_rdev_key_to_semantic(key) {
            let should_suppress = key_state
                .last_crossterm_output
                .as_ref()
                .map(|(last_key, timestamp)| {
                    last_key == &semantic_key
                        && now.duration_since(*timestamp)
                            < Duration::from_millis(SUPPRESS_WINDOW_MS)
                })
                .unwrap_or(false);
            if !should_suppress {
                output_keys.push(semantic_key);
            }
        }
    }

    drop(key_state);

    for semantic_key in output_keys {
        let _ = input_sender.send(HostInputEvent::Key { key: semantic_key });
    }
}

fn allowed_crossterm_keycodes() -> HashSet<KeyCode> {
    use KeyCode::*;

    let mut keycodes = HashSet::new();
    for character in 'a'..='z' {
        keycodes.insert(Char(character));
        keycodes.insert(Char(character.to_ascii_uppercase()));
    }
    for character in '0'..='9' {
        keycodes.insert(Char(character));
    }
    for symbol in [
        ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';',
        '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
    ] {
        keycodes.insert(Char(symbol));
    }
    for function_key in 1..=12 {
        keycodes.insert(F(function_key));
    }

    for keycode in [
        Enter,
        Tab,
        BackTab,
        Backspace,
        Esc,
        Delete,
        Insert,
        Home,
        End,
        PageUp,
        PageDown,
        Up,
        Down,
        Left,
        Right,
        CapsLock,
        NumLock,
        ScrollLock,
        PrintScreen,
        Pause,
        Menu,
    ] {
        keycodes.insert(keycode);
    }

    keycodes
}

fn map_crossterm_key_to_semantic(code: KeyCode) -> Option<String> {
    use KeyCode::*;
    match code {
        Char(' ') => Some("space".to_string()),
        Char(character) => Some(character.to_string()),
        F(index) if (1..=12).contains(&index) => Some(format!("f{index}")),
        Up => Some("up".to_string()),
        Down => Some("down".to_string()),
        Left => Some("left".to_string()),
        Right => Some("right".to_string()),
        Home => Some("home".to_string()),
        End => Some("end".to_string()),
        PageUp => Some("pageup".to_string()),
        PageDown => Some("pagedown".to_string()),
        Enter => Some("enter".to_string()),
        Backspace => Some("backspace".to_string()),
        Delete => Some("del".to_string()),
        Insert => Some("ins".to_string()),
        Tab => Some("tab".to_string()),
        BackTab => Some("back_tab".to_string()),
        Esc => Some("esc".to_string()),
        CapsLock => Some("capslock".to_string()),
        NumLock => Some("numlock".to_string()),
        ScrollLock => Some("scrolllock".to_string()),
        PrintScreen => Some("printscreen".to_string()),
        Pause => Some("pause".to_string()),
        Menu => Some("menu".to_string()),
        _ => None,
    }
}

fn map_rdev_key_to_semantic(code: RdevKey) -> Option<String> {
    use RdevKey::*;
    match code {
        Num0 | Kp0 => Some("0".to_string()),
        Num1 | Kp1 => Some("1".to_string()),
        Num2 | Kp2 => Some("2".to_string()),
        Num3 | Kp3 => Some("3".to_string()),
        Num4 | Kp4 => Some("4".to_string()),
        Num5 | Kp5 => Some("5".to_string()),
        Num6 | Kp6 => Some("6".to_string()),
        Num7 | Kp7 => Some("7".to_string()),
        Num8 | Kp8 => Some("8".to_string()),
        Num9 | Kp9 => Some("9".to_string()),
        KeyA => Some("a".to_string()),
        KeyB => Some("b".to_string()),
        KeyC => Some("c".to_string()),
        KeyD => Some("d".to_string()),
        KeyE => Some("e".to_string()),
        KeyF => Some("f".to_string()),
        KeyG => Some("g".to_string()),
        KeyH => Some("h".to_string()),
        KeyI => Some("i".to_string()),
        KeyJ => Some("j".to_string()),
        KeyK => Some("k".to_string()),
        KeyL => Some("l".to_string()),
        KeyM => Some("m".to_string()),
        KeyN => Some("n".to_string()),
        KeyO => Some("o".to_string()),
        KeyP => Some("p".to_string()),
        KeyQ => Some("q".to_string()),
        KeyR => Some("r".to_string()),
        KeyS => Some("s".to_string()),
        KeyT => Some("t".to_string()),
        KeyU => Some("u".to_string()),
        KeyV => Some("v".to_string()),
        KeyW => Some("w".to_string()),
        KeyX => Some("x".to_string()),
        KeyY => Some("y".to_string()),
        KeyZ => Some("z".to_string()),
        F1 => Some("f1".to_string()),
        F2 => Some("f2".to_string()),
        F3 => Some("f3".to_string()),
        F4 => Some("f4".to_string()),
        F5 => Some("f5".to_string()),
        F6 => Some("f6".to_string()),
        F7 => Some("f7".to_string()),
        F8 => Some("f8".to_string()),
        F9 => Some("f9".to_string()),
        F10 => Some("f10".to_string()),
        F11 => Some("f11".to_string()),
        F12 => Some("f12".to_string()),
        ControlLeft => Some("left_ctrl".to_string()),
        ControlRight => Some("right_ctrl".to_string()),
        ShiftLeft => Some("left_shift".to_string()),
        ShiftRight => Some("right_shift".to_string()),
        MetaLeft => Some("left_meta".to_string()),
        MetaRight => Some("right_meta".to_string()),
        Alt => Some("left_alt".to_string()),
        AltGr => Some("right_alt".to_string()),
        CapsLock => Some("capslock".to_string()),
        NumLock => Some("numlock".to_string()),
        ScrollLock => Some("scrolllock".to_string()),
        UpArrow => Some("up".to_string()),
        DownArrow => Some("down".to_string()),
        LeftArrow => Some("left".to_string()),
        RightArrow => Some("right".to_string()),
        Home => Some("home".to_string()),
        End => Some("end".to_string()),
        PageUp => Some("pageup".to_string()),
        PageDown => Some("pagedown".to_string()),
        Insert => Some("ins".to_string()),
        Delete => Some("del".to_string()),
        Backspace => Some("backspace".to_string()),
        Return => Some("enter".to_string()),
        Space => Some("space".to_string()),
        Tab => Some("tab".to_string()),
        Escape => Some("esc".to_string()),
        BackQuote => Some("`".to_string()),
        Minus => Some("-".to_string()),
        Equal => Some("=".to_string()),
        LeftBracket => Some("[".to_string()),
        RightBracket => Some("]".to_string()),
        BackSlash | IntlBackslash => Some("\\".to_string()),
        SemiColon => Some(";".to_string()),
        Quote => Some("'".to_string()),
        Comma => Some(",".to_string()),
        Dot => Some(".".to_string()),
        Slash => Some("/".to_string()),
        KpPlus => Some("+".to_string()),
        KpMinus => Some("-".to_string()),
        KpMultiply => Some("*".to_string()),
        KpDivide => Some("/".to_string()),
        KpDelete => Some("del".to_string()),
        KpReturn => Some("enter".to_string()),
        Function => Some("fn".to_string()),
        PrintScreen => Some("printscreen".to_string()),
        Pause => Some("pause".to_string()),
        Unknown(code) => Some(format!("key({code})")),
    }
}
