// 引入官方哈希表
use std::collections::HashMap;

use crate::host_engine::services::{CanvasService, CanvasStyle};

// 页面枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PageKey {
  Home,
  Packages,
  Settings,
}

impl PageKey {
  // 页面名
  pub fn name(&self) -> &'static str {
    match self {
      PageKey::Home => "Home",
      PageKey::Packages => "Packages",
      PageKey::Settings => "Settings",
    }
  }

  // 下一页
  pub fn next(&self) -> Self {
    match self {
      PageKey::Home => PageKey::Packages,
      PageKey::Packages => PageKey::Settings,
      PageKey::Settings => PageKey::Home,
    }
  }

  // 上一页
  pub fn prev(&self) -> Self {
    match self {
      PageKey::Home => PageKey::Settings,
      PageKey::Packages => PageKey::Home,
      PageKey::Settings => PageKey::Packages,
    }
  }
}

// 页面特征：每个页面实现自身的绘制逻辑
//
// 注意：此 API 当前未被运行时调用，为后续 UI 系统预留。
// 绘制时使用 CanvasService 替代旧的 RenderService。
pub trait Page {
  fn key(&self) -> PageKey;
  fn render(&self, canvas: &mut CanvasService);
}

/// 居中绘制文本的便捷辅助函数
fn draw_centered(canvas: &mut CanvasService, y: u16, text: &str) {
  let (width, _) = canvas.size();
  let centered_x = width.saturating_sub(text.len() as u16).saturating_div(2);
  canvas.clear_row(y);
  canvas.write_text(centered_x, y, text, CanvasStyle::default());
}

pub struct UiService {
  pages: HashMap<PageKey, Box<dyn Page>>,
  active_page: PageKey,
}

impl UiService {
  pub fn new() -> Self {
    let mut service = Self {
      pages: HashMap::new(),
      active_page: PageKey::Home,
    };

    service.register(Box::new(HomePage));
    service.register(Box::new(PackagesPage));
    service.register(Box::new(SettingsPage));

    service
  }

  // 注册页
  pub fn register(&mut self, page: Box<dyn Page>) {
    self.pages.insert(page.key(), page);
  }

  // 当前活跃页
  pub fn active_page(&self) -> PageKey {
    self.active_page
  }

  // 切换下一页
  pub fn navigate_next(&mut self) {
    self.active_page = self.active_page.next();
  }

  // 切换上一页
  pub fn navigate_prev(&mut self) {
    self.active_page = self.active_page.prev();
  }

  // 绘制活跃页面
  pub fn render_active(&self, canvas: &mut CanvasService) {
    if let Some(page) = self.pages.get(&self.active_page) {
      page.render(canvas)
    }
  }

  // 尺寸大小重绘
  pub fn on_resize(&mut self, width: u16, height: u16) {
    // TODO(render): 当增量渲染器存在时，在窗口大小调整后强制执行完整的重新绘制。
  }
}

struct HomePage;

impl Page for HomePage {
  fn key(&self) -> PageKey {
    PageKey::Home
  }

  fn render(&self, canvas: &mut CanvasService) {
    draw_centered(canvas, 5, "Welcome to TUI Game Engine");
    draw_centered(canvas, 7, "<- / -> to navigate | ESC to exit");
  }
}

struct PackagesPage;

impl Page for PackagesPage {
  fn key(&self) -> PageKey {
    PageKey::Packages
  }

  fn render(&self, canvas: &mut CanvasService) {
    draw_centered(canvas, 5, "Package List");
    draw_centered(canvas, 7, "(package list will appear here)");
  }
}

struct SettingsPage;

impl Page for SettingsPage {
  fn key(&self) -> PageKey {
    PageKey::Settings
  }

  fn render(&self, canvas: &mut CanvasService) {
    draw_centered(canvas, 5, "Settings");
    draw_centered(canvas, 7, "(settings will appear here)");
  }
}
