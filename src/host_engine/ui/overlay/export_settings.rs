use std::path::Path;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect,
  RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, TextInputCursorShape,
  TextInputEvent, TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams, UiEvent,
  UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "export_settings";

const HINT_GRAY: &str = "rgb(85,87,83)";

/// 导出文件格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
  Zip,
  Tar,
  TarGz,
}

impl ExportFormat {
  fn i18n_key(self) -> &'static str {
    match self {
      Self::Zip => "export_settings.set.type.zip",
      Self::Tar => "export_settings.set.type.tar",
      Self::TarGz => "export_settings.set.type.tar.gz",
    }
  }

  #[allow(dead_code)]
  fn extension(self) -> &'static str {
    match self {
      Self::Zip => "zip",
      Self::Tar => "tar",
      Self::TarGz => "tar.gz",
    }
  }

  fn next(self) -> Self {
    match self {
      Self::Zip => Self::Tar,
      Self::Tar => Self::TarGz,
      Self::TarGz => Self::Zip,
    }
  }

  fn prev(self) -> Self {
    match self {
      Self::Zip => Self::TarGz,
      Self::Tar => Self::Zip,
      Self::TarGz => Self::Tar,
    }
  }
}

/// 导出范围类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportType {
  Cache,
  Log,
  Mod,
  Profile,
  Screenshot,
  Recording,
  Data,
}

impl ExportType {
  #[allow(dead_code)]
  fn dynamic_type_str(self) -> &'static str {
    match self {
      Self::Cache => "cache",
      Self::Log => "log",
      Self::Mod => "mod",
      Self::Profile => "profile",
      Self::Screenshot => "screenshot",
      Self::Recording => "recording",
      Self::Data => "data",
    }
  }

  fn dynamic_type_text(self, i18n: &I18nService) -> String {
    let key = match self {
      Self::Cache => "export_settings.scope.cache",
      Self::Log => "export_settings.scope.log",
      Self::Mod => "export_settings.scope.mod",
      Self::Profile => "export_settings.scope.profile",
      Self::Screenshot => "export_settings.scope.screenshot",
      Self::Recording => "export_settings.scope.recording",
      Self::Data => "export_settings.scope.data",
    };
    i18n.get_runtime_text(NS, key)
  }
}

/// 聚焦目标
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExportSettingsFocus {
  Name,
  Path,
  Type,
}

impl ExportSettingsFocus {
  fn next(self) -> Self {
    match self {
      Self::Name => Self::Path,
      Self::Path => Self::Type,
      Self::Type => Self::Name,
    }
  }

  fn prev(self) -> Self {
    match self {
      Self::Name => Self::Type,
      Self::Path => Self::Name,
      Self::Type => Self::Path,
    }
  }
}

pub struct ExportSettingsUi {
  objects: UiObjectPool,
  #[allow(dead_code)]
  runtime_objects: RuntimeObjectPool,
  focus: ExportSettingsFocus,
  name_input_id: TextInputId,
  path_input_id: TextInputId,
  back_area: HitAreaId,
  name_area: HitAreaId,
  path_area: HitAreaId,
  type_area: HitAreaId,
  format: ExportFormat,
  export_type: Option<ExportType>,
  /// 是否有输入框正在活跃
  input_active: bool,
  /// TextInput 文字缓存，Changed 事件时同步
  name_text: String,
  path_text: String,
  /// 程序根目录，用于 {root} 解析
  root_dir: std::path::PathBuf,
  /// 当前校验状态，render 时更新，供 ConfirmExport 检查
  name_valid: bool,
  path_valid: bool,
}

impl ExportSettingsUi {
  pub fn init(
    hit_area: &HitAreaService,
    text_input: &crate::host_engine::services::TextInputService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let name_input_id = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text: String::new(),
        max_chars: Some(128),
        mode: TextInputMode::SingleLine,
        mouse: true,
      },
    );
    let path_input_id = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text: String::new(),
        max_chars: None,
        mode: TextInputMode::SingleLine,
        mouse: true,
      },
    );
    Self {
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      name_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      path_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      type_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      focus: ExportSettingsFocus::Name,
      name_input_id,
      path_input_id,
      format: ExportFormat::Zip,
      export_type: None,
      input_active: false,
      name_text: String::new(),
      path_text: String::new(),
      root_dir: std::path::PathBuf::new(),
      name_valid: false,
      path_valid: false,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "export_settings.focus_up".to_string(),
        description: "Focus previous setting".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.focus_down".to_string(),
        description: "Focus next setting".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.focus_name".to_string(),
        description: "Focus name input".to_string(),
        keys: vec![vec!["1".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.focus_path".to_string(),
        description: "Focus path input".to_string(),
        keys: vec![vec!["2".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.focus_type".to_string(),
        description: "Focus format selector".to_string(),
        keys: vec![vec!["3".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.confirm".to_string(),
        description: "Confirm / enter input / submit".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.back".to_string(),
        description: "Back / exit input".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.type_left".to_string(),
        description: "Previous format".to_string(),
        keys: vec![vec!["left".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.type_right".to_string(),
        description: "Next format".to_string(),
        keys: vec![vec!["right".to_string()]],
      },
      ActionMapEntry {
        action: "export_settings.confirm_export".to_string(),
        description: "Confirm export".to_string(),
        keys: vec![vec!["ctrl".to_string(), "s".to_string()]],
      },
    ]
  }

  pub fn start(&mut self, export_type: ExportType, root_dir: std::path::PathBuf) {
    self.export_type = Some(export_type);
    self.focus = ExportSettingsFocus::Name;
    self.format = ExportFormat::Zip;
    self.root_dir = root_dir;
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ExportSettingsCommand> {
    let name_id = self.name_input_id;
    let path_id = self.path_input_id;

    match event {
      // ── TextInput 组件事件 ──────────────────────────────
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == name_id => {
        self.focus = ExportSettingsFocus::Name;
        Some(ExportSettingsCommand::FocusInput)
      }
      UiEvent::TextInput(TextInputEvent::Pressed { id }) if *id == path_id => {
        self.focus = ExportSettingsFocus::Path;
        Some(ExportSettingsCommand::FocusInput)
      }
      UiEvent::TextInput(TextInputEvent::PressedOutside { .. }) => {
        Some(ExportSettingsCommand::BlurInput)
      }
      UiEvent::TextInput(TextInputEvent::Cancel { .. }) => Some(ExportSettingsCommand::CancelInput),
      UiEvent::TextInput(TextInputEvent::Submit { .. }) => Some(ExportSettingsCommand::BlurInput),
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == name_id => {
        self.name_text = value.clone();
        None
      }
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == path_id => {
        self.path_text = value.clone();
        None
      }
      // ── HitArea ─────────────────────────────────────────
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) if !self.input_active => {
        if *id == self.name_area {
          self.focus = ExportSettingsFocus::Name;
        } else if *id == self.path_area {
          self.focus = ExportSettingsFocus::Path;
        } else if *id == self.type_area {
          self.focus = ExportSettingsFocus::Type;
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if !self.input_active => {
        if *id == self.name_area || *id == self.path_area {
          self.focus = if *id == self.name_area {
            ExportSettingsFocus::Name
          } else {
            ExportSettingsFocus::Path
          };
          return Some(ExportSettingsCommand::FocusInput);
        } else if *id == self.type_area {
          self.focus = ExportSettingsFocus::Type;
          self.format = self.format.next();
        }
        None
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        id,
        button: MouseButton::Right,
        ..
      }) if *id == self.back_area
        || *id == self.name_area
        || *id == self.path_area
        || *id == self.type_area =>
      {
        if self.input_active {
          Some(ExportSettingsCommand::CancelInput)
        } else {
          Some(ExportSettingsCommand::Cancel)
        }
      }
      // ── Action ──────────────────────────────────────────
      UiEvent::Action(event) if event.state == KeyState::Pressed && !self.input_active => {
        match event.action.as_str() {
          "export_settings.focus_up" => {
            self.focus = self.focus.prev();
            None
          }
          "export_settings.focus_down" => {
            self.focus = self.focus.next();
            None
          }
          "export_settings.focus_name" => {
            self.focus = ExportSettingsFocus::Name;
            None
          }
          "export_settings.focus_path" => {
            self.focus = ExportSettingsFocus::Path;
            None
          }
          "export_settings.focus_type" => {
            self.focus = ExportSettingsFocus::Type;
            None
          }
          "export_settings.confirm" => {
            if self.focus != ExportSettingsFocus::Type {
              Some(ExportSettingsCommand::FocusInput)
            } else {
              None
            }
          }
          "export_settings.back" => Some(ExportSettingsCommand::Cancel),
          "export_settings.type_left" => {
            if self.focus == ExportSettingsFocus::Type {
              self.format = self.format.prev();
            }
            None
          }
          "export_settings.type_right" => {
            if self.focus == ExportSettingsFocus::Type {
              self.format = self.format.next();
            }
            None
          }
          "export_settings.confirm_export" => {
            if self.name_valid && self.path_valid {
              Some(ExportSettingsCommand::ConfirmExport)
            } else {
              None
            }
          }
          _ => None,
        }
      }
      _ => None,
    }
  }

  pub fn focus_input(&mut self, text_input: &mut crate::host_engine::services::TextInputService) {
    // 保存旧值，供 CancelInput 恢复
    self.name_text = text_input
      .get_text(&self.objects, self.name_input_id)
      .unwrap_or("")
      .to_string();
    self.path_text = text_input
      .get_text(&self.objects, self.path_input_id)
      .unwrap_or("")
      .to_string();
    self.input_active = true;
    match self.focus {
      ExportSettingsFocus::Name => {
        let _ = text_input.focus(&mut self.objects, self.name_input_id);
      }
      ExportSettingsFocus::Path => {
        let _ = text_input.focus(&mut self.objects, self.path_input_id);
      }
      ExportSettingsFocus::Type => {}
    }
  }

  pub fn blur_input(&mut self, text_input: &mut crate::host_engine::services::TextInputService) {
    self.input_active = false;
    let _ = text_input.blur(&mut self.objects);
  }

  /// Esc：退出输入并恢复旧值
  pub fn cancel_input(&mut self, text_input: &mut crate::host_engine::services::TextInputService) {
    let restore_name = self.name_text.clone();
    let restore_path = self.path_text.clone();
    self.input_active = false;
    let _ = text_input.blur(&mut self.objects);
    let _ = text_input.set_text(&mut self.objects, self.name_input_id, &restore_name);
    let _ = text_input.set_text(&mut self.objects, self.path_input_id, &restore_path);
    self.name_text = restore_name;
    self.path_text = restore_path;
  }

  pub fn name_text(&self) -> &str {
    &self.name_text
  }

  pub fn path_text(&self) -> &str {
    &self.path_text
  }

  pub fn resolved_name(&self) -> String {
    let trimmed = self.name_text.trim();
    if trimmed.is_empty() {
      let _export_type = self.export_type.unwrap_or(ExportType::Data);
      let type_str = _export_type.dynamic_type_str();
      let now = chrono::Local::now();
      let time_str = now.format("%Y-%m-%d %H-%M-%S").to_string();
      format!("TUI GAME {}_{}", type_str, time_str)
    } else {
      trimmed.to_string()
    }
  }

  pub fn resolved_path(&self) -> String {
    let trimmed = self.path_text.trim();
    if trimmed.is_empty() {
      self.root_dir.to_string_lossy().replace('\\', "/")
    } else {
      trimmed.replace('\\', "/")
    }
  }

  #[allow(dead_code)]
  pub fn format(&self) -> ExportFormat {
    self.format
  }

  pub fn export_scope(&self) -> Option<ExportType> {
    self.export_type
  }

  fn validate_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
      return false;
    }
    !trimmed.contains(&['<', '>', ':', '"', '|', '?', '*'][..])
      && trimmed.find(|c: char| c.is_ascii_control()).is_none()
  }

  fn validate_path(path_str: &str) -> bool {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
      return false;
    }
    let normalized = trimmed.replace('\\', "/");
    let path = Path::new(&normalized);
    path.exists() && path.is_dir()
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    text_input: &crate::host_engine::services::TextInputService,
  ) {
    let Some(_export_type) = self.export_type else {
      return;
    };

    let size = layout.physical_size();
    let params = Self::key_params();

    // ── Title ──────────────────────────────────────────────
    let title = i18n.get_runtime_text(NS, "export_settings.title");
    let title_w = layout.get_text_width(&title, None);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, title_w, 0),
        y: 1,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );

    // ── Content layout — dynamic centering ──────────────────
    let name_label = i18n.get_runtime_text(NS, "export_settings.set.name");
    let path_label = i18n.get_runtime_text(NS, "export_settings.set.path");
    let type_label = i18n.get_runtime_text(NS, "export_settings.set.type");
    let format_label = i18n.get_runtime_text(NS, self.format.i18n_key());

    // Resolved defaults for validation
    let _export_type = self.export_type.unwrap_or(ExportType::Data);
    let type_str = _export_type.dynamic_type_text(i18n);
    let now = chrono::Local::now();
    let time_str = now.format("%Y-%m-%d %H-%M-%S").to_string();
    let root_str = self.root_dir.to_string_lossy().replace('\\', "/");

    let default_name_resolved = i18n
      .get_runtime_text(NS, "export_settings.set.name.default")
      .replace("{type}", &type_str)
      .replace("{time}", &time_str);
    let default_path_resolved = i18n
      .get_runtime_text(NS, "export_settings.set.path.default")
      .replace("{root}", &root_str);

    // Raw hints
    let name_hint_raw = i18n.get_runtime_text(NS, "export_settings.set.name.tip");
    let path_hint_raw = i18n.get_runtime_text(NS, "export_settings.set.path.tip");

    // Validation
    let name_to_validate = {
      let raw = self.name_text();
      if raw.trim().is_empty() {
        default_name_resolved.clone()
      } else {
        raw.to_string()
      }
    };
    let path_to_validate = {
      let raw = self.path_text();
      if raw.trim().is_empty() {
        default_path_resolved.clone()
      } else {
        raw.to_string()
      }
    };
    let valid_name = Self::validate_name(&name_to_validate);
    let valid_path = Self::validate_path(&path_to_validate);
    let name_valid_label = if valid_name {
      i18n.get_runtime_text(NS, "export_settings.set.name.right")
    } else {
      i18n.get_runtime_text(NS, "export_settings.set.name.error")
    };
    let path_valid_label = if valid_path {
      i18n.get_runtime_text(NS, "export_settings.set.path.right")
    } else {
      i18n.get_runtime_text(NS, "export_settings.set.path.error")
    };
    self.name_valid = valid_name;
    self.path_valid = valid_path;
    let name_valid_color = if valid_name {
      "bright_green"
    } else {
      "bright_red"
    };
    let path_valid_color = if valid_path {
      "bright_green"
    } else {
      "bright_red"
    };

    // Indicators
    let ind_on = "<fg:bright_cyan>❯</fg>";
    let ind_off = " ";
    let name_ind = if self.focus == ExportSettingsFocus::Name {
      ind_on
    } else {
      ind_off
    };
    let path_ind = if self.focus == ExportSettingsFocus::Path {
      ind_on
    } else {
      ind_off
    };
    let type_ind = if self.focus == ExportSettingsFocus::Type {
      ind_on
    } else {
      ind_off
    };

    // Border color
    let name_border_fg: Option<crate::host_engine::services::TextColor> =
      if self.focus == ExportSettingsFocus::Name && self.input_active {
        Some(crate::host_engine::services::TextColor::Terminal(
          crate::host_engine::services::TerminalColor::BrightCyan,
        ))
      } else {
        None
      };
    let path_border_fg: Option<crate::host_engine::services::TextColor> =
      if self.focus == ExportSettingsFocus::Path && self.input_active {
        Some(crate::host_engine::services::TextColor::Terminal(
          crate::host_engine::services::TerminalColor::BrightCyan,
        ))
      } else {
        None
      };

    let name_placeholder = i18n.get_runtime_text(NS, "export_settings.set.name.default");
    let path_placeholder = i18n.get_runtime_text(NS, "export_settings.set.path.default");

    // ── Measure content widths ──────────────────────────────
    let name_label_w = layout.get_text_width(&name_label, None);
    let name_valid_w = layout.get_text_width(&name_valid_label, Some(&params));
    let name_line_w = name_label_w + 1 + name_valid_w; // label + space + status

    let path_label_w = layout.get_text_width(&path_label, None);
    let path_valid_w = layout.get_text_width(&path_valid_label, Some(&params));
    let path_line_w = path_label_w + 1 + path_valid_w;

    let name_hint_w = layout.get_text_width(&name_hint_raw, Some(&params));
    let path_hint_w = layout.get_text_width(&path_hint_raw, Some(&params));

    // Type line: "❯ 格式选择 [ZIP]"
    let type_line_plain = format!("❯ {} [{}]", type_label, format_label);
    let type_line_w = layout.get_text_width(&type_line_plain, Some(&params));

    // Border dimensions (capped at 52, min 6)
    let max_avail = size.width.saturating_sub(32).max(14);
    let border_w = (52u16).min(max_avail.saturating_sub(2)).max(6);
    let inner_w = border_w.saturating_sub(2);
    const BORDER_H: u16 = 3;
    let indented_border_w = 2 + border_w; // ❯ + space + border

    // Section widths = max of (label_line, indented_border, hint_indented)
    let name_section_w = [name_line_w, indented_border_w, 2 + name_hint_w]
      .into_iter()
      .max()
      .unwrap_or(1);
    let path_section_w = [path_line_w, indented_border_w, 2 + path_hint_w]
      .into_iter()
      .max()
      .unwrap_or(1);
    let type_section_w = type_line_w + 2; // ❯ indicator offset
    let block_w = [name_section_w, path_section_w, type_section_w]
      .into_iter()
      .max()
      .unwrap_or(1)
      .min(max_avail)
      .max(1);
    let content_x = size.width.saturating_sub(block_w) / 2;
    let border_x = content_x + 2;

    // ── Vertical centering ──────────────────────────────────
    const SECTION_H: u16 = 5;
    let total_rows = SECTION_H + 1 + SECTION_H + 1 + 1u16; // name + gap + path + gap + type = 13
    let start_y = (size
      .height
      .saturating_sub(1 /* hint */ + 1)
      .saturating_sub(total_rows)
      / 2)
      .max(3u16);
    let name_y = start_y;
    let path_y = name_y + SECTION_H + 1;
    let type_y = path_y + SECTION_H + 1;

    // Input rects
    let name_input_rect = Rect {
      x: border_x + 1,
      y: name_y + 2,
      width: inner_w,
      height: 1,
    };
    let path_input_rect = Rect {
      x: border_x + 1,
      y: path_y + 2,
      width: inner_w,
      height: 1,
    };

    let name_focused = text_input.is_focused(&self.objects, self.name_input_id);
    let path_focused = text_input.is_focused(&self.objects, self.path_input_id);

    // ── Name section ────────────────────────────────────────
    // Row 0: label + validation
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: name_y,
        text: format!(
          "f%{} <fg:{}>{}</fg>",
          name_label, name_valid_color, name_valid_label
        ),
        params: Some(params.clone()),
        max_width: Some(block_w),
        ..Default::default()
      },
    );

    // Rows 1-3: border box
    render.draw_host_border_rect(
      canvas,
      border_x,
      name_y + 1,
      border_w,
      BORDER_H,
      &BorderStyle::Line,
      name_border_fg,
      None,
      None,
      None,
    );

    // Row 2: ❯ indicator
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: name_y + 2,
        text: format!("f%{}", name_ind),
        ..Default::default()
      },
    );

    // Row 2: text input widget (handles text + placeholder + cursor)
    text_input.render_host(
      &mut self.objects,
      self.name_input_id,
      &TextInputRenderParams {
        rect: name_input_rect,
        placeholder: name_placeholder,
        cursor_shape: if name_focused {
          None
        } else {
          Some(TextInputCursorShape::None)
        },
        ..Default::default()
      },
      canvas,
    );

    // Row 4: raw hint
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x + 2,
        y: name_y + 4,
        text: format!("f%<fg:{}>{}</fg>", HINT_GRAY, name_hint_raw),
        params: Some(params.clone()),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(block_w.saturating_sub(2)),
        ..Default::default()
      },
    );

    // ── Path section ───────────────────────────────────────
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: path_y,
        text: format!(
          "f%{} <fg:{}>{}</fg>",
          path_label, path_valid_color, path_valid_label
        ),
        params: Some(params.clone()),
        max_width: Some(block_w),
        ..Default::default()
      },
    );

    render.draw_host_border_rect(
      canvas,
      border_x,
      path_y + 1,
      border_w,
      BORDER_H,
      &BorderStyle::Line,
      path_border_fg,
      None,
      None,
      None,
    );

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: path_y + 2,
        text: format!("f%{}", path_ind),
        ..Default::default()
      },
    );

    text_input.render_host(
      &mut self.objects,
      self.path_input_id,
      &TextInputRenderParams {
        rect: path_input_rect,
        placeholder: path_placeholder,
        cursor_shape: if path_focused {
          None
        } else {
          Some(TextInputCursorShape::None)
        },
        ..Default::default()
      },
      canvas,
    );

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x + 2,
        y: path_y + 4,
        text: format!("f%<fg:{}>{}</fg>", HINT_GRAY, path_hint_raw),
        params: Some(params.clone()),
        wrap_mode: TextWrapMode::Auto,
        max_width: Some(block_w.saturating_sub(2)),
        ..Default::default()
      },
    );

    // ── Type section (no border) ────────────────────────────
    let type_color = if self.focus == ExportSettingsFocus::Type {
      "bright_cyan"
    } else {
      "white"
    };
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: content_x,
        y: type_y,
        text: format!(
          "f%{} {} <fg:{}>[{}]</fg>",
          type_ind, type_label, type_color, format_label
        ),
        params: Some(params.clone()),
        max_width: Some(block_w),
        ..Default::default()
      },
    );

    // Hit areas — content-width, skip when text input active
    if !self.input_active {
      hit_area.render_host(
        &mut self.objects,
        self.back_area,
        Rect {
          x: 0,
          y: 0,
          width: size.width,
          height: size.height,
        },
        canvas,
      );
      hit_area.render_host(
        &mut self.objects,
        self.name_area,
        Rect {
          x: content_x,
          y: name_y,
          width: name_section_w,
          height: SECTION_H,
        },
        canvas,
      );
      hit_area.render_host(
        &mut self.objects,
        self.path_area,
        Rect {
          x: content_x,
          y: path_y,
          width: path_section_w,
          height: SECTION_H,
        },
        canvas,
      );
      hit_area.render_host(
        &mut self.objects,
        self.type_area,
        Rect {
          x: content_x,
          y: type_y,
          width: type_section_w,
          height: 1,
        },
        canvas,
      );
    }

    // ── Bottom hint ─────────────────────────────────────────
    let hint = self.bottom_hint(i18n);
    let hint_w = layout.get_text_width(&hint, Some(&params));
    let hint_x = size.width.saturating_sub(hint_w) / 2;
    let hint_y = size.height.saturating_sub(1);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: hint_x,
        y: hint_y,
        text: hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }

  fn bottom_hint(&self, i18n: &I18nService) -> String {
    if self.input_active {
      format!(
        "f%<fg:{}>{}  {}</fg>",
        HINT_GRAY,
        i18n.get_runtime_text(NS, "export_settings.action.input.back"),
        i18n.get_runtime_text(NS, "export_settings.action.input.confirm"),
      )
    } else if self.focus == ExportSettingsFocus::Type {
      format!(
        "f%<fg:{}>{}  {}  {}  {}  {}</fg>",
        HINT_GRAY,
        i18n.get_runtime_text(NS, "export_settings.action.select"),
        i18n.get_runtime_text(NS, "export_settings.action.focus"),
        i18n.get_runtime_text(NS, "export_settings.action.type.select"),
        i18n.get_runtime_text(NS, "export_settings.action.back"),
        i18n.get_runtime_text(NS, "export_settings.action.confirm_export"),
      )
    } else {
      format!(
        "f%<fg:{}>{}  {}  {}  {}  {}</fg>",
        HINT_GRAY,
        i18n.get_runtime_text(NS, "export_settings.action.select"),
        i18n.get_runtime_text(NS, "export_settings.action.focus"),
        i18n.get_runtime_text(NS, "export_settings.action.confirm"),
        i18n.get_runtime_text(NS, "export_settings.action.back"),
        i18n.get_runtime_text(NS, "export_settings.action.confirm_export"),
      )
    }
  }

  fn key_params() -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "export_settings.")
  }
}

impl UiObjectPoolOwner for ExportSettingsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ExportSettingsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportSettingsCommand {
  Cancel,
  FocusInput,
  BlurInput,
  CancelInput,
  ConfirmExport,
}
