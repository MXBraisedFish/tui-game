use std::{path::Path, time::Duration};

use crate::host_engine::services::{
  ActionMapEntry, AudioError, AudioId, AudioService, AudioSource, AudioState, BorderStyle,
  CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService, KeyState,
  LayoutService, MouseButton, ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions,
  ProgressBarSegmentStyle, ProgressBarService, Rect, RenderService, RuntimeObjectPool,
  RuntimeObjectPoolOwner, StorageService, TerminalColor, TextAlign, TextColor, TextStyle, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

const TEST_AUDIO_PATH: &str = "audio/test.mp3";
const DEFAULT_VOLUME: f32 = 0.8;

const CONTROL_TOGGLE: usize = 0;
const CONTROL_STOP: usize = 1;
const CONTROL_RESTART: usize = 2;
const CONTROL_VOLUME_DOWN: usize = 3;
const CONTROL_VOLUME_UP: usize = 4;
const CONTROL_LOOP: usize = 5;

const PLAYBACK_GREEN: TextColor = TextColor::Rgb {
  r: 95,
  g: 215,
  b: 105,
};
const PLAYBACK_YELLOW: TextColor = TextColor::Rgb {
  r: 238,
  g: 205,
  b: 90,
};
const PLAYBACK_GRAY: TextColor = TextColor::Rgb {
  r: 85,
  g: 87,
  b: 83,
};
const PLAYER_BORDER: TextColor = TextColor::Rgb {
  r: 95,
  g: 215,
  b: 215,
};

/// Temporary audio player on the About page, used to verify the host audio service.
pub struct InputDemoUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  progress: ProgressBarId,
  controls: [HitAreaId; 6],
  audio_id: Option<AudioId>,
  state: Option<AudioState>,
  duration: Option<Duration>,
  position: Duration,
  volume: f32,
  looped: bool,
  last_error: Option<String>,
  last_position_tick: u64,
}

impl UiObjectPoolOwner for InputDemoUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for InputDemoUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDemoCommand {
  Back,
  TogglePlayback,
  Stop,
  Restart,
  VolumeDown,
  VolumeUp,
  ToggleLoop,
}

impl InputDemoUi {
  pub fn init(hit_area: &HitAreaService, progress_bar: &ProgressBarService) -> Self {
    let mut objects = UiObjectPool::new();
    let progress = progress_bar
      .create(&mut objects, playback_progress_options())
      .expect("audio test progress bar options must be valid");
    let controls = std::array::from_fn(|_| {
      hit_area.create(
        &mut objects,
        HitAreaOptions {
          hover_move: false,
          drag: false,
        },
      )
    });
    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      progress,
      controls,
      audio_id: None,
      state: None,
      duration: None,
      position: Duration::ZERO,
      volume: DEFAULT_VOLUME,
      looped: false,
      last_error: None,
      last_position_tick: 0,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "input_demo.back".into(),
        description: "Back to home".into(),
        keys: vec![vec!["esc".into()]],
      },
      ActionMapEntry {
        action: "input_demo.toggle".into(),
        description: "Play or pause test audio".into(),
        keys: vec![vec!["space".into()]],
      },
      ActionMapEntry {
        action: "input_demo.stop".into(),
        description: "Stop test audio".into(),
        keys: vec![vec!["s".into()]],
      },
      ActionMapEntry {
        action: "input_demo.restart".into(),
        description: "Restart test audio".into(),
        keys: vec![vec!["r".into()]],
      },
      ActionMapEntry {
        action: "input_demo.volume_down".into(),
        description: "Decrease test audio volume".into(),
        keys: vec![vec!["left".into()]],
      },
      ActionMapEntry {
        action: "input_demo.volume_up".into(),
        description: "Increase test audio volume".into(),
        keys: vec![vec!["right".into()]],
      },
      ActionMapEntry {
        action: "input_demo.loop".into(),
        description: "Toggle test audio loop".into(),
        keys: vec![vec!["l".into()]],
      },
    ]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    match event {
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "input_demo.back" => Some(InputDemoCommand::Back),
        "input_demo.toggle" => Some(InputDemoCommand::TogglePlayback),
        "input_demo.stop" => Some(InputDemoCommand::Stop),
        "input_demo.restart" => Some(InputDemoCommand::Restart),
        "input_demo.volume_down" => Some(InputDemoCommand::VolumeDown),
        "input_demo.volume_up" => Some(InputDemoCommand::VolumeUp),
        "input_demo.loop" => Some(InputDemoCommand::ToggleLoop),
        _ => None,
      },
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => self.command_for_control(*id),
      _ => None,
    }
  }

  pub fn update(
    &mut self,
    audio: &mut AudioService,
    storage: &StorageService,
    progress_bar: &ProgressBarService,
  ) -> bool {
    if self.audio_id.is_none() && self.last_error.is_none() {
      self.load_test_audio(audio, storage);
    }

    let Some(audio_id) = self.audio_id else {
      return false;
    };
    let state = audio.state(self.objects.audio(), audio_id);
    let duration = audio.duration(self.objects.audio(), audio_id);
    let position = audio
      .position(self.objects.audio(), audio_id)
      .unwrap_or(Duration::ZERO);
    let ratio = duration
      .filter(|duration| !duration.is_zero())
      .map(|duration| position.as_secs_f64() / duration.as_secs_f64())
      .unwrap_or(0.0)
      .clamp(0.0, 1.0) as f32;
    progress_bar.set_completed(&mut self.objects, self.progress, ratio);

    let position_tick = position.as_millis().min(u64::MAX as u128) as u64 / 100;
    let changed =
      self.state != state || self.duration != duration || self.last_position_tick != position_tick;
    self.state = state;
    self.duration = duration;
    self.position = position;
    self.last_position_tick = position_tick;
    changed
  }

  pub fn toggle_playback(&mut self, audio: &mut AudioService) {
    let Some(audio_id) = self.audio_id else {
      return;
    };
    let result = if audio.state(self.objects.audio(), audio_id) == Some(AudioState::Playing) {
      audio.pause(self.objects.audio_mut(), audio_id)
    } else {
      audio.play(self.objects.audio_mut(), audio_id)
    };
    self.set_operation_result(result);
  }

  pub fn stop(&mut self, audio: &mut AudioService) {
    let Some(audio_id) = self.audio_id else {
      return;
    };
    let result = audio.stop(self.objects.audio_mut(), audio_id);
    self.set_operation_result(result);
  }

  pub fn restart(&mut self, audio: &mut AudioService) {
    let Some(audio_id) = self.audio_id else {
      return;
    };
    let result = audio.restart(self.objects.audio_mut(), audio_id);
    self.set_operation_result(result);
  }

  pub fn adjust_volume(&mut self, audio: &mut AudioService, delta: f32) {
    let Some(audio_id) = self.audio_id else {
      return;
    };
    let next = (self.volume + delta).clamp(0.0, 1.0);
    match audio.set_volume(self.objects.audio_mut(), audio_id, next) {
      Ok(true) => {
        self.volume = next;
        self.last_error = None;
      }
      Ok(false) => self.last_error = Some("音频对象已失效".into()),
      Err(error) => self.last_error = Some(error.to_string()),
    }
  }

  pub fn toggle_loop(&mut self, audio: &mut AudioService) {
    let Some(audio_id) = self.audio_id else {
      return;
    };
    let next = !self.looped;
    match audio.set_loop(self.objects.audio_mut(), audio_id, next) {
      Ok(true) => {
        self.looped = next;
        self.last_error = None;
      }
      Ok(false) => self.last_error = Some("音频对象已失效".into()),
      Err(error) => self.last_error = Some(error.to_string()),
    }
  }

  pub fn leave(&mut self, audio: &mut AudioService) {
    if let Some(audio_id) = self.audio_id.take() {
      let _ = audio.stop(self.objects.audio_mut(), audio_id);
      let _ = audio.remove(self.objects.audio_mut(), audio_id);
    }
    self.state = None;
    self.duration = None;
    self.position = Duration::ZERO;
  }

  #[allow(clippy::too_many_arguments)]
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    hit_area: &HitAreaService,
    progress_bar: &ProgressBarService,
  ) {
    let screen_width = layout.developer_width();
    let screen_height = layout.developer_height();
    if screen_width < 24 || screen_height < 10 {
      return;
    }

    let width = screen_width.saturating_sub(4).min(78);
    let height = screen_height.saturating_sub(2).min(18);
    let x = screen_width.saturating_sub(width) / 2;
    let y = screen_height.saturating_sub(height) / 2;
    render.draw_border_rect(
      canvas,
      x,
      y,
      width,
      height,
      &BorderStyle::Line,
      Some(PLAYER_BORDER),
      None,
      None,
      None,
    );

    let inner_x = x.saturating_add(2);
    let inner_width = width.saturating_sub(4);
    let title_y = y.saturating_add(1);
    draw_centered(
      render,
      canvas,
      inner_x,
      title_y,
      inner_width,
      "音频系统测试播放器",
      Some(TextColor::Terminal(TerminalColor::BrightMagenta)),
      true,
    );
    draw_centered(
      render,
      canvas,
      inner_x,
      title_y.saturating_add(2),
      inner_width,
      "资源：assets/audio/test.mp3",
      Some(PLAYBACK_GRAY),
      false,
    );

    let (state_text, state_color) = state_display(self.state);
    draw_centered(
      render,
      canvas,
      inner_x,
      title_y.saturating_add(4),
      inner_width,
      &format!("状态：{state_text}"),
      Some(state_color),
      true,
    );
    draw_centered(
      render,
      canvas,
      inner_x,
      title_y.saturating_add(5),
      inner_width,
      &format!(
        "{} / {}",
        format_duration(self.position),
        self
          .duration
          .map(format_duration)
          .unwrap_or_else(|| "--:--".into())
      ),
      None,
      false,
    );

    let progress_rect = Rect {
      x: inner_x,
      y: title_y.saturating_add(7),
      width: inner_width,
      height: 1,
    };
    progress_bar.render(&self.objects, self.progress, progress_rect, canvas);
    draw_centered(
      render,
      canvas,
      inner_x,
      title_y.saturating_add(8),
      inner_width,
      &format!(
        "音量：{:>3}%    循环：{}",
        (self.volume * 100.0).round() as u8,
        if self.looped { "开启" } else { "关闭" }
      ),
      None,
      false,
    );

    let control_y = title_y.saturating_add(10);
    let toggle_label = if self.state == Some(AudioState::Playing) {
      "[ 暂停 ]"
    } else {
      "[ 播放 ]"
    };
    self.draw_control_row(
      render,
      canvas,
      layout,
      hit_area,
      inner_x,
      inner_width,
      control_y,
      &[
        (CONTROL_TOGGLE, toggle_label),
        (CONTROL_STOP, "[ 停止 ]"),
        (CONTROL_RESTART, "[ 重播 ]"),
      ],
    );
    self.draw_control_row(
      render,
      canvas,
      layout,
      hit_area,
      inner_x,
      inner_width,
      control_y.saturating_add(2),
      &[
        (CONTROL_VOLUME_DOWN, "[ 音量 - ]"),
        (CONTROL_VOLUME_UP, "[ 音量 + ]"),
        (CONTROL_LOOP, "[ 循环 ]"),
      ],
    );

    if let Some(error) = self.last_error.as_deref() {
      draw_centered(
        render,
        canvas,
        inner_x,
        y.saturating_add(height.saturating_sub(2)),
        inner_width,
        error,
        Some(TextColor::Terminal(TerminalColor::BrightRed)),
        false,
      );
    }

    if screen_height >= 2 {
      draw_centered(
        render,
        canvas,
        0,
        screen_height.saturating_sub(1),
        screen_width,
        "[Space] 播放/暂停  [S] 停止  [R] 重播  [←]/[→] 音量  [L] 循环  [Esc] 返回",
        Some(PLAYBACK_GRAY),
        false,
      );
    }
  }

  fn load_test_audio(&mut self, audio: &mut AudioService, storage: &StorageService) {
    let source = match storage.resolve_audio_asset(Path::new(TEST_AUDIO_PATH)) {
      Ok(file) => AudioSource::File(file),
      Err(error) => {
        self.last_error = Some(error.to_string());
        return;
      }
    };
    let audio_id = match audio.create(self.objects.audio_mut(), source, None) {
      Ok(id) => id,
      Err(error) => {
        self.last_error = Some(error.to_string());
        return;
      }
    };
    self.audio_id = Some(audio_id);
    if let Err(error) = audio
      .set_volume(self.objects.audio_mut(), audio_id, self.volume)
      .and_then(|_| audio.play(self.objects.audio_mut(), audio_id).map(|_| true))
    {
      self.last_error = Some(error.to_string());
    }
  }

  fn command_for_control(&self, id: HitAreaId) -> Option<InputDemoCommand> {
    let index = self.controls.iter().position(|control| *control == id)?;
    Some(match index {
      CONTROL_TOGGLE => InputDemoCommand::TogglePlayback,
      CONTROL_STOP => InputDemoCommand::Stop,
      CONTROL_RESTART => InputDemoCommand::Restart,
      CONTROL_VOLUME_DOWN => InputDemoCommand::VolumeDown,
      CONTROL_VOLUME_UP => InputDemoCommand::VolumeUp,
      CONTROL_LOOP => InputDemoCommand::ToggleLoop,
      _ => return None,
    })
  }

  fn set_operation_result(&mut self, result: Result<(), AudioError>) {
    self.last_error = result.err().map(|error| error.to_string());
  }

  #[allow(clippy::too_many_arguments)]
  fn draw_control_row(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    hit_area: &HitAreaService,
    x: u16,
    width: u16,
    y: u16,
    controls: &[(usize, &str)],
  ) {
    let gap = 2;
    let total_width = controls
      .iter()
      .map(|(_, label)| layout.get_text_width(label, None))
      .sum::<u16>()
      .saturating_add(gap * controls.len().saturating_sub(1) as u16);
    let mut cursor = x.saturating_add(width.saturating_sub(total_width) / 2);
    for (index, label) in controls {
      let label_width = layout.get_text_width(label, None);
      let control = self.controls[*index];
      let hovered = hit_area.is_hovered(&self.objects, control);
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: cursor,
          y,
          text: (*label).to_string(),
          fg: Some(if hovered {
            TextColor::Terminal(TerminalColor::Black)
          } else {
            TextColor::Terminal(TerminalColor::BrightCyan)
          }),
          bg: hovered.then_some(TextColor::Terminal(TerminalColor::BrightCyan)),
          max_width: Some(label_width),
          max_height: Some(1),
          ..Default::default()
        },
      );
      hit_area.render(
        &mut self.objects,
        control,
        Rect {
          x: cursor,
          y,
          width: label_width,
          height: 1,
        },
        canvas,
      );
      cursor = cursor.saturating_add(label_width).saturating_add(gap);
    }
  }
}

fn playback_progress_options() -> ProgressBarOptions {
  let segment = |color| ProgressBarSegmentStyle {
    ch: '─',
    style: TextStyle {
      foreground: Some(color),
      background: Some(TextColor::Transparent),
      ..Default::default()
    },
  };
  ProgressBarOptions {
    completed: segment(PLAYBACK_GREEN),
    preview: segment(PLAYBACK_GREEN),
    remaining: segment(PLAYBACK_GRAY),
    origin: ProgressBarFillOrigin::Left,
  }
}

#[allow(clippy::too_many_arguments)]
fn draw_centered(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  x: u16,
  y: u16,
  width: u16,
  text: &str,
  fg: Option<TextColor>,
  bold: bool,
) {
  render.draw_text(
    canvas,
    &DrawTextParams {
      x,
      y,
      text: text.to_string(),
      fg,
      line_align: TextAlign::Center,
      max_width: Some(width),
      max_height: Some(1),
      overflow_marker: Some("...".into()),
      bold,
      ..Default::default()
    },
  );
}

fn state_display(state: Option<AudioState>) -> (&'static str, TextColor) {
  match state {
    None => ("等待加载", PLAYBACK_GRAY),
    Some(AudioState::Created | AudioState::Loading) => ("正在加载", PLAYBACK_YELLOW),
    Some(AudioState::Ready) => ("准备完成", TextColor::Terminal(TerminalColor::BrightCyan)),
    Some(AudioState::Playing) => ("正在播放", PLAYBACK_GREEN),
    Some(AudioState::Paused) => ("已暂停", PLAYBACK_YELLOW),
    Some(AudioState::Stopped) => ("已停止", TextColor::Terminal(TerminalColor::White)),
    Some(AudioState::Finished) => ("播放完成", TextColor::Terminal(TerminalColor::BrightCyan)),
    Some(AudioState::Failed) => (
      "加载/播放失败",
      TextColor::Terminal(TerminalColor::BrightRed),
    ),
  }
}

fn format_duration(duration: Duration) -> String {
  let total_seconds = duration.as_secs();
  let minutes = total_seconds / 60;
  let seconds = total_seconds % 60;
  format!("{minutes}:{seconds:02}")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duration_display_uses_unbounded_minutes() {
    assert_eq!(format_duration(Duration::from_secs(5)), "0:05");
    assert_eq!(format_duration(Duration::from_secs(125)), "2:05");
    assert_eq!(format_duration(Duration::from_secs(3_605)), "60:05");
  }
}
