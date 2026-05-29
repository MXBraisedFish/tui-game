// 引入官方标准双端队列库和间隔库
use std::collections::VecDeque;
use std::time::Duration;

// 引入crossterm事件库
use crossterm::event::{self, Event, KeyCode, KeyEvent, poll};
use rdev::Key;

// 按键输入
#[derive(Clone, Debug)]
pub struct  KeyInput {
  pub code: KeyCode, // 按键码
  pub kind: KeyEventKind // 按键类型
}

// 输入事件
#[derive(Clone, Debug)]
pub enum InputEvent {
  Key(KeyInput),
  Resize {
    width: u16,
    height: u16
  }
}

// 按键输入状态
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
  queue: VecDeque<InputEvent> // 使用双端队列做按键列表缓冲
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
      // 从系统读取事件
      match event::read() {
        // 按键事件
        Ok(Event::Key(key_event)) => {
          self.queue.push_back(InputEvent::Key(KeyInput::from(key_event)));
        }
        // 尺寸变化事件
        Ok(Event::Resize(width, height)) => {
          self.queue.push_back(InputEvent::Resize { width, height });
        }
        // 其它事件一律忽略
        _ => {}
      }
    }
  }

  // 获取下一个按键（头部出队并返回）
  pub fn next_key(&mut self) -> Option<KeyInput> {
    match self.queue.pop_front() {
      Some(InputEvent::Key(key)) => Some(key),
      _ => None
    }
  }

  // 下个事件
  pub fn next_event(&mut self) -> Option<InputEvent> {
    self.queue.pop_front()
  }

  // 消费按键事件
  pub fn consume_key(&mut self, code: KeyCode) -> bool {
    // 检查队头是否满足匹配
    // 先获取队头元素，然后为Some类型进入闭包
    let matched = self.queue.front().is_some_and(|event| {
      match event {
        // 判断键码和状态类型
        InputEvent::Key(key) => {
          key.code == code && matches!(
            key.kind, KeyEventKind::Press | KeyEventKind::Repeat
          )
        }
        _ => false
      }
    });

    // 消费匹配到的按键
    if matched {
      // 移除头部
      self.queue.pop_front();
      true
    } else {
      false
    }
  }

  // 消费尺寸变化事件
  pub fn consume_resize(&mut self) -> Option<(u16, u16)> {
    // 检查队头是否满足匹配
    // 先获取队头元素，然后为Some类型进入闭包
    let matched = self.queue.front().and_then(|event| {
      match event {
        // 这里有个解引用
        InputEvent::Resize { width, height } => Some((*width, *height)),
        _ => None
      }
    });

    // 消费事件
    if matched.is_some() {
      self.queue.pop_front();
    }

    // 返回提取到的数据
    matched
  }
}
