/// 全局键盘输入管理，融合 crossterm 终端事件与 rdev 全局热键监听
/// 业务逻辑：
/// 双源键盘输入
/// 事件去重与延迟
/// Shift 键长按检测
/// 语义键映射
/// 显式语义键集合
/// 显示转换

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use once_cell::sync::Lazy;
use rdev::{Event as REvent, EventType as REventType, Key as RKey, listen};

use crate::utils::host_log;

const RDEV_DELAY: Duration = Duration::from_millis(20);
const SUPPRESS_WINDOW: Duration = Duration::from_millis(120);

#[derive(Debug, Clone)]
struct DelayedRdev {
    key: RKey,
    timestamp: Instant,
}

#[derive(Default)]
struct SharedState {
    delayed_rdevs: VecDeque<DelayedRdev>,
    last_ct_output: Option<(String, Instant)>,
    shift_keys_down: HashSet<RKey>,
    shift_hold_started: Option<Instant>,
}

pub struct SemanticKeySource {
    shared: Arc<Mutex<SharedState>>,
}

static ALLOWED_CT_KEYCODES: Lazy<HashSet<KeyCode>> = Lazy::new(allowed_ct_keycodes);
static EXPLICIT_SEMANTIC_KEYS: Lazy<HashSet<String>> = Lazy::new(explicit_semantic_keys);
static GLOBAL_KEY_SOURCE: Lazy<SemanticKeySource> = Lazy::new(SemanticKeySource::new);

pub fn semantic_key_source() -> &'static SemanticKeySource {
    &GLOBAL_KEY_SOURCE
}

pub fn is_explicit_semantic_key(key: &str) -> bool {
    EXPLICIT_SEMANTIC_KEYS.contains(key.trim())
}

pub fn display_semantic_key(key: &str, case_sensitive: bool) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }

    if key.len() == 1 {
        let ch = key.chars().next().unwrap_or_default();
        if ch.is_ascii_lowercase() && !case_sensitive {
            return ch.to_ascii_uppercase().to_string();
        }
        return ch.to_string();
    }

    match key {
        "f1" => "F1",
        "f2" => "F2",
        "f3" => "F3",
        "f4" => "F4",
        "f5" => "F5",
        "f6" => "F6",
        "f7" => "F7",
        "f8" => "F8",
        "f9" => "F9",
        "f10" => "F10",
        "f11" => "F11",
        "f12" => "F12",
        "up" => "\u{2191}",
        "down" => "\u{2193}",
        "left" => "\u{2190}",
        "right" => "\u{2192}",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PgUp",
        "pagedown" => "PgDn",
        "enter" => "Enter",
        "backspace" => "Bksp",
        "del" => "Del",
        "ins" => "Ins",
        "tab" => "Tab",
        "back_tab" => "BTab",
        "space" => "Space",
        "left_ctrl" => "LCtrl",
        "right_ctrl" => "RCtrl",
        "left_shift" => "LShift",
        "right_shift" => "RShift",
        "shift" => "Shift",
        "left_alt" => "LAlt",
        "right_alt" => "RAlt",
        "left_meta" => "LMeta",
        "right_meta" => "RMeta",
        "capslock" => "Caps",
        "numlock" => "Num",
        "scrolllock" => "Scrl",
        "esc" => "Esc",
        "printscreen" => "Prtsc",
        "pause" => "Pause",
        "menu" => "Menu",
        other => other,
    }
    .to_string()
}

impl SemanticKeySource {
    fn new() -> Self {
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let shared_rdev = Arc::clone(&shared);
        thread::spawn(move || {
            if let Err(err) = listen(move |event: REvent| {
                if let Ok(mut state) = shared_rdev.lock() {
                    match event.event_type {
                        REventType::KeyPress(key) => {
                            state.delayed_rdevs.push_back(DelayedRdev {
                                key,
                                timestamp: Instant::now(),
                            });
                            if is_shift_key(key) {
                                state.shift_keys_down.insert(key);
                                state.shift_hold_started.get_or_insert_with(Instant::now);
                            }
                        }
                        REventType::KeyRelease(key) => {
                            if is_shift_key(key) {
                                state.shift_keys_down.remove(&key);
                                if state.shift_keys_down.is_empty() {
                                    state.shift_hold_started = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }) {
                host_log::append_host_error(
                    "host.error.global_keyboard_listener_failed",
                    &[("err", &format!("{err:?}"))],
                );
            }
        });

        Self { shared }
    }

    pub fn record_crossterm_key(&self, key: KeyEvent) -> Vec<String> {
        if !matches!(key.kind, KeyEventKind::Press) {
            return Vec::new();
        }

        let now = Instant::now();
        let mut state = match self.shared.lock() {
            Ok(state) => state,
            Err(_) => return Vec::new(),
        };

        if ALLOWED_CT_KEYCODES.contains(&key.code) {
            if let Some(semantic) = map_keycode_to_semantic(key.code) {
                state.last_ct_output = Some((semantic.clone(), now));
                return vec![semantic];
            }
        }

        Vec::new()
    }

    pub fn drain_ready_rdev_keys(&self, limit: usize) -> Vec<String> {
        let now = Instant::now();
        let mut out = Vec::new();
        let mut state = match self.shared.lock() {
            Ok(state) => state,
            Err(_) => return out,
        };

        while out.len() < limit {
            let Some(delayed) = state.delayed_rdevs.front() else {
                break;
            };
            if now.duration_since(delayed.timestamp) < RDEV_DELAY {
                break;
            }

            let key = delayed.key;
            state.delayed_rdevs.pop_front();
            if let Some(semantic) = map_rkey_to_semantic(key) {
                let suppressed = state
                    .last_ct_output
                    .as_ref()
                    .map(|(last_key, timestamp)| {
                        last_key == &semantic && now.duration_since(*timestamp) < SUPPRESS_WINDOW
                    })
                    .unwrap_or(false);
                if suppressed {
                    continue;
                }
                out.push(semantic);
            }
        }

        out
    }

    pub fn clear_pending_keys(&self) {
        if let Ok(mut state) = self.shared.lock() {
            state.delayed_rdevs.clear();
            state.shift_keys_down.clear();
            state.shift_hold_started = None;
        }
    }

    pub fn is_shift_held_for(&self, duration: Duration) -> bool {
        let Ok(state) = self.shared.lock() else {
            return false;
        };
        !state.shift_keys_down.is_empty()
            && state
                .shift_hold_started
                .map(|started| started.elapsed() >= duration)
                .unwrap_or(false)
    }
}

fn is_shift_key(key: RKey) -> bool {
    matches!(key, RKey::ShiftLeft | RKey::ShiftRight)
}

fn allowed_ct_keycodes() -> HashSet<KeyCode> {
    use KeyCode::*;
    let mut set = HashSet::new();

    for c in 'a'..='z' {
        set.insert(Char(c));
        set.insert(Char(c.to_ascii_uppercase()));
    }
    for c in '0'..='9' {
        set.insert(Char(c));
    }
    let symbols = [
        ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';',
        '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
    ];
    for &c in &symbols {
        set.insert(Char(c));
    }
    for i in 1..=12 {
        set.insert(F(i));
    }
    set.insert(Enter);
    set.insert(Tab);
    set.insert(BackTab);
    set.insert(Backspace);
    set.insert(Esc);
    set.insert(Delete);
    set.insert(Insert);
    set.insert(Home);
    set.insert(End);
    set.insert(PageUp);
    set.insert(PageDown);
    set.insert(Up);
    set.insert(Down);
    set.insert(Left);
    set.insert(Right);
    set.insert(CapsLock);
    set.insert(NumLock);
    set.insert(ScrollLock);
    set.insert(PrintScreen);
    set.insert(Pause);
    set.insert(Menu);

    set
}

fn explicit_semantic_keys() -> HashSet<String> {
    let mut set = HashSet::new();

    for key in allowed_ct_keycodes()
        .into_iter()
        .filter_map(map_keycode_to_semantic)
    {
        set.insert(key);
    }

    for key in [
        "left_ctrl",
        "right_ctrl",
        "left_shift",
        "right_shift",
        "shift",
        "left_meta",
        "right_meta",
        "left_alt",
        "right_alt",
        "fn",
    ] {
        set.insert(key.to_string());
    }

    set
}

fn map_keycode_to_semantic(code: KeyCode) -> Option<String> {
    use KeyCode::*;
    match code {
        Char('1') => Some("1".to_string()),
        Char('2') => Some("2".to_string()),
        Char('3') => Some("3".to_string()),
        Char('4') => Some("4".to_string()),
        Char('5') => Some("5".to_string()),
        Char('6') => Some("6".to_string()),
        Char('7') => Some("7".to_string()),
        Char('8') => Some("8".to_string()),
        Char('9') => Some("9".to_string()),
        Char('0') => Some("0".to_string()),
        Char('!') => Some("!".to_string()),
        Char('@') => Some("@".to_string()),
        Char('#') => Some("#".to_string()),
        Char('$') => Some("$".to_string()),
        Char('%') => Some("%".to_string()),
        Char('^') => Some("^".to_string()),
        Char('&') => Some("&".to_string()),
        Char('*') => Some("*".to_string()),
        Char('(') => Some("(".to_string()),
        Char(')') => Some(")".to_string()),
        Char('a') => Some("a".to_string()),
        Char('b') => Some("b".to_string()),
        Char('c') => Some("c".to_string()),
        Char('d') => Some("d".to_string()),
        Char('e') => Some("e".to_string()),
        Char('f') => Some("f".to_string()),
        Char('g') => Some("g".to_string()),
        Char('h') => Some("h".to_string()),
        Char('i') => Some("i".to_string()),
        Char('j') => Some("j".to_string()),
        Char('k') => Some("k".to_string()),
        Char('l') => Some("l".to_string()),
        Char('m') => Some("m".to_string()),
        Char('n') => Some("n".to_string()),
        Char('o') => Some("o".to_string()),
        Char('p') => Some("p".to_string()),
        Char('q') => Some("q".to_string()),
        Char('r') => Some("r".to_string()),
        Char('s') => Some("s".to_string()),
        Char('t') => Some("t".to_string()),
        Char('u') => Some("u".to_string()),
        Char('v') => Some("v".to_string()),
        Char('w') => Some("w".to_string()),
        Char('x') => Some("x".to_string()),
        Char('y') => Some("y".to_string()),
        Char('z') => Some("z".to_string()),
        Char('A') => Some("A".to_string()),
        Char('B') => Some("B".to_string()),
        Char('C') => Some("C".to_string()),
        Char('D') => Some("D".to_string()),
        Char('E') => Some("E".to_string()),
        Char('F') => Some("F".to_string()),
        Char('G') => Some("G".to_string()),
        Char('H') => Some("H".to_string()),
        Char('I') => Some("I".to_string()),
        Char('J') => Some("J".to_string()),
        Char('K') => Some("K".to_string()),
        Char('L') => Some("L".to_string()),
        Char('M') => Some("M".to_string()),
        Char('N') => Some("N".to_string()),
        Char('O') => Some("O".to_string()),
        Char('P') => Some("P".to_string()),
        Char('Q') => Some("Q".to_string()),
        Char('R') => Some("R".to_string()),
        Char('S') => Some("S".to_string()),
        Char('T') => Some("T".to_string()),
        Char('U') => Some("U".to_string()),
        Char('V') => Some("V".to_string()),
        Char('W') => Some("W".to_string()),
        Char('X') => Some("X".to_string()),
        Char('Y') => Some("Y".to_string()),
        Char('Z') => Some("Z".to_string()),
        Char('`') => Some("`".to_string()),
        Char('~') => Some("~".to_string()),
        Char('_') => Some("_".to_string()),
        Char('+') => Some("+".to_string()),
        Char('[') => Some("[".to_string()),
        Char(']') => Some("]".to_string()),
        Char(';') => Some(";".to_string()),
        Char('\'') => Some("'".to_string()),
        Char(',') => Some(",".to_string()),
        Char('.') => Some(".".to_string()),
        Char('/') => Some("/".to_string()),
        Char('-') => Some("-".to_string()),
        Char('=') => Some("=".to_string()),
        Char('{') => Some("{".to_string()),
        Char('}') => Some("}".to_string()),
        Char(':') => Some(":".to_string()),
        Char('"') => Some("\"".to_string()),
        Char('<') => Some("<".to_string()),
        Char('>') => Some(">".to_string()),
        Char('?') => Some("?".to_string()),
        Char('|') => Some("|".to_string()),
        Char('\\') => Some("\\".to_string()),
        F(1) => Some("f1".to_string()),
        F(2) => Some("f2".to_string()),
        F(3) => Some("f3".to_string()),
        F(4) => Some("f4".to_string()),
        F(5) => Some("f5".to_string()),
        F(6) => Some("f6".to_string()),
        F(7) => Some("f7".to_string()),
        F(8) => Some("f8".to_string()),
        F(9) => Some("f9".to_string()),
        F(10) => Some("f10".to_string()),
        F(11) => Some("f11".to_string()),
        F(12) => Some("f12".to_string()),
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
        Char(' ') => Some("space".to_string()),
        _ => None,
    }
}

fn map_rkey_to_semantic(code: RKey) -> Option<String> {
    use RKey::*;
    match code {
        Num0 => Some("0".to_string()),
        Num1 => Some("1".to_string()),
        Num2 => Some("2".to_string()),
        Num3 => Some("3".to_string()),
        Num4 => Some("4".to_string()),
        Num5 => Some("5".to_string()),
        Num6 => Some("6".to_string()),
        Num7 => Some("7".to_string()),
        Num8 => Some("8".to_string()),
        Num9 => Some("9".to_string()),
        Kp0 => Some("0".to_string()),
        Kp1 => Some("1".to_string()),
        Kp2 => Some("2".to_string()),
        Kp3 => Some("3".to_string()),
        Kp4 => Some("4".to_string()),
        Kp5 => Some("5".to_string()),
        Kp6 => Some("6".to_string()),
        Kp7 => Some("7".to_string()),
        Kp8 => Some("8".to_string()),
        Kp9 => Some("9".to_string()),
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
        BackSlash => Some("\\".to_string()),
        SemiColon => Some(";".to_string()),
        Quote => Some("'".to_string()),
        Comma => Some(",".to_string()),
        Dot => Some(".".to_string()),
        Slash => Some("/".to_string()),
        IntlBackslash => Some("\\".to_string()),
        KpPlus => Some("+".to_string()),
        KpMinus => Some("-".to_string()),
        KpMultiply => Some("*".to_string()),
        KpDivide => Some("/".to_string()),
        KpDelete => Some("del".to_string()),
        KpReturn => Some("enter".to_string()),
        Function => Some("fn".to_string()),
        PrintScreen => Some("printscreen".to_string()),
        Pause => Some("pause".to_string()),
        Unknown(code) => Some(format!("key({})", code)),
        _ => None,
    }
}
