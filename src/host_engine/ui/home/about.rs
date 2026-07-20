use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaService, KeyState, LayoutService,
  RenderService, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService, SliceService,
  UiEvent, UiObjectPool, UiObjectPoolOwner,
};

/// Unicode 与终端单元格覆盖样本页。
pub struct InputDemoUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
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
}

impl InputDemoUi {
  pub fn init(
    _hit_area: &HitAreaService,
    _slices: &SliceService,
    _scroll_box: &ScrollBoxService,
  ) -> Self {
    Self {
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![ActionMapEntry {
      action: "input_demo.back".into(),
      description: "Back to home".into(),
      keys: vec![vec!["esc".into()]],
    }]
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<InputDemoCommand> {
    let UiEvent::Action(event) = event else {
      return None;
    };
    (event.state == KeyState::Pressed && event.action == "input_demo.back")
      .then_some(InputDemoCommand::Back)
  }

  pub fn update(&mut self) {}

  pub fn apply_layout(&mut self, _layout: &LayoutService, _scroll_box: &ScrollBoxService) {}

  pub fn leave(&mut self) {}

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _hit_area: &HitAreaService,
    _scroll_box: &ScrollBoxService,
  ) {
    let samples = unicode_samples();
    let width = layout.developer_width().saturating_sub(2);
    for (row, text) in samples.iter().enumerate() {
      render.draw_text(
        canvas,
        &DrawTextParams {
          x: 1,
          y: row as u16,
          text: (*text).to_string(),
          max_width: Some(width),
          max_height: Some(1),
          overflow_marker: Some("...".into()),
          ..Default::default()
        },
      );
    }
  }
}

fn unicode_samples() -> &'static [&'static str] {
  &[
    "ASCII: !\"#$%&'()*+,-./ 0123456789 :;<=>?@ ABC xyz [\\]^_` {|}~",
    "Latin: ÀÁÂÃÄÅ Æ Ç ÈÉÊË ÌÍÎÏ Ñ ÒÓÔÕÖ Ø Œ ÙÚÛÜ Ý ß ẞ",
    "Combining: e\u{301} a\u{308} n\u{303} A\u{30a}  ZWJ: 👩\u{200d}💻 👨\u{200d}👩\u{200d}👧\u{200d}👦",
    "Zero width: A\u{200b}B A\u{200c}B A\u{200d}B A\u{2060}B  VS: ✈︎ ✈️",
    "RTL: עברית العربية فارسی اردو  | controls: \u{202b}ABC العربية\u{202c}",
    "CJK: 中文繁體 日本語かなカナ 한글 漢字 〇々〆〄〓〈〉《》「」『』【】",
    "Kana/Bopomofo: あいうえお アイウエオ ｱｲｳｴｵ ㄅㄆㄇㄈ ㆠㆡㆢ",
    "Indic/SEA: हिन्दी বাংলা ਪੰਜਾਬੀ ગુજરાતી தமிழ் తెలుగు ಕನ್ನಡ മലയാളം ไทย ລາວ မြန်မာ",
    "Greek/Cyrillic: ΑΒΓΔ αβγδ  Ελληνικά  АБВГ абвг Русский Українська",
    "Semitic/African: אבגדה العربية ሀሁሂ ትግርኛ ꦗꦮ ꧋ ߒߞߏ",
    "Symbols: ←↑→↓ ↔↕ ⇐⇒ ∀∂∃∅∇∈∉∑√∞∧∨∩∪≈≠≤≥ ⌘⌥⌫⏎",
    "Box: ─│┌┐└┘├┤┬┴┼ ═║╔╗╚╝╠╣╦╩╬ ╭╮╰╯ ┏┓┗┛┣┫┳┻╋",
    "Blocks: ▀▁▂▃▄▅▆▇█ ▏▎▍▌▋▊▉ ░▒▓ ■□▪▫●○◆◇◢◣◤◥",
    "Braille: ⠀⠁⠃⠇⠏⠟⠿⡿⣿  Music: ♩♪♫♬♭♮♯  Cards: ♠♥♦♣",
    "Emoji: 😀🥹🫠🚀🌍🔥✨⚙️🧪🏳️‍🌈🇨🇳👍🏽  Keycap: 1️⃣ #️⃣ *️⃣",
    "Historic/rare: 𓀀𓂀 𐀀 𐎀 𐤀 ᚠᚢᚦᚨᚱᚲ ⰀⰁ ⸘ ※ ⁂ ‽",
    "Full/Half width: ＡＢＣ１２３！ ａｂｃ ﾊﾝｶｸ ｡｢｣､･  Tab→\t←Tab",
    "Space widths: [ ] [\u{a0}] [\u{2002}] [\u{2003}] [\u{2009}] [\u{3000}] end",
  ]
}
