// 引入官方标准双端队列库和间隔库
use std::collections::VecDeque;
use std::time::Duration;

// 引入crossterm事件库
use crossterm::event::{self, Event, KeyCode, KeyEvent, poll};

#[derive(Clone, Debug)]
pub struct  KeyInput {
  pub code: KeyCode, // 按键码
  pub kind: KeyEventKind // 按键类型
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
  Press, // 按下
  Release, // 松开
  Repeat // 持续按下
}

impl From<KeyEvent> for KeyInput {
  fn from(event: KeyEvent) -> Self {
    Self {
      code: event.code,
      kind: match event.kind {
        crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
        crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
        crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat
      }
    }
  }
}

pub struct InputService {
  queue: VecDeque<KeyInput> // 使用双端队列做按键列表缓冲
}

impl InputService {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new()
    }
  }

  // 收集所有待处理按键
  pub fn poll(&mut self) {
    // 只要还有按键事件就不断处理
    while poll(Duration::ZERO).unwrap_or(false) {
      // 读取第一个事件，然后做类型比较，获取最终按键事件
      if let Ok(Event::Key(key_event)) = event::read() {
        self.queue.push_back(KeyInput::from(key_event)); // 转换事件类型为自己的，入队列
      }
    }
  }

  // 获取下一个按键（头部出队并返回）
  pub fn next_key(&mut self) -> Option<KeyInput> {
    self.queue.pop_front()
  }

  // 只检查并删除队头的特定按键
  pub fn consume_key(&mut self, code: KeyCode) -> bool {
    let matched = self.queue.front().is_some_and(|key| {
      key.code == code && matches!(
        key.kind,
        KeyEventKind::Press | KeyEventKind::Repeat
      )
    });

    if matched {
      self.queue.pop_front();
      true
    } else {
      false
    }
  }
}
