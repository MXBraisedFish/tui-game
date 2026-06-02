// 引入官方哈希表
use std::collections::HashMap;

use crate::host_engine::services::{RenderService, render};

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

pub trait Page {
  fn key(&self) -> PageKey;
  fn render(&self, renderer: &mut RenderService);
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
  pub fn render_active(&self, renderer: &mut RenderService) {
    if let Some(page) = self.pages.get(&self.active_page) {
      page.render(renderer)
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

  fn render(&self, renderer: &mut RenderService) {
    renderer.draw_centered(5, "Welcome to TUI Game Engine");
    renderer.draw_centered(7, "<- / -> to navigate | ESC to exit");
  }
}

struct PackagesPage;

impl Page for PackagesPage {
  fn key(&self) -> PageKey {
    PageKey::Packages
  }

  fn render(&self, renderer: &mut RenderService) {
    renderer.draw_centered(5, "Package List");
    renderer.draw_centered(7, "(package list will appear here)");
  }
}

struct SettingsPage;

impl Page for SettingsPage {
  fn key(&self) -> PageKey {
    PageKey::Settings
  }

  fn render(&self, renderer: &mut RenderService) {
    renderer.draw_centered(5, "Settings");
    renderer.draw_centered(7, "(settings will appear here)");
  }
}
