# 步骤 11：UiService — 页面注册与导航

## 目标

给 `UiService` 添加页面栈。注册占位页面，用方向键切换，
渲染活动页面。证明 UI 槽位能跑通。

## 期望行为

- 3 个页面：Home、Packages、Settings
- 左/右方向键切换页面
- 每个页面渲染不同的占位内容
- 状态栏显示当前页面名称

## 操作

### 11.1 实现 `src/app/services/ui.rs`

```rust
use std::collections::HashMap;

/// UI 页面唯一标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PageKey {
    Home,
    Packages,
    Settings,
}

impl PageKey {
    pub fn name(&self) -> &'static str {
        match self {
            PageKey::Home => "Home",
            PageKey::Packages => "Packages",
            PageKey::Settings => "Settings",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PageKey::Home => PageKey::Packages,
            PageKey::Packages => PageKey::Settings,
            PageKey::Settings => PageKey::Home,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            PageKey::Home => PageKey::Settings,
            PageKey::Packages => PageKey::Home,
            PageKey::Settings => PageKey::Packages,
        }
    }
}

/// 最小页面 trait。
pub trait Page {
    fn key(&self) -> PageKey;
    fn title(&self) -> &str;
    fn render(&self, renderer: &mut crate::app::services::render::RenderService);
}

pub struct UiService {
    pages: HashMap<PageKey, Box<dyn Page>>,
    active_page: PageKey,
    page_order: Vec<PageKey>,
}

impl UiService {
    pub fn new() -> Self {
        let mut service = Self {
            pages: HashMap::new(),
            active_page: PageKey::Home,
            page_order: vec![PageKey::Home, PageKey::Packages, PageKey::Settings],
        };
        // 注册三个占位页面
        service.register(Box::new(HomePage));
        service.register(Box::new(PackagesPage));
        service.register(Box::new(SettingsPage));
        service
    }

    pub fn register(&mut self, page: Box<dyn Page>) {
        self.pages.insert(page.key(), page);
    }

    pub fn active_page(&self) -> PageKey { self.active_page }

    pub fn navigate_next(&mut self) { self.active_page = self.active_page.next(); }
    pub fn navigate_prev(&mut self) { self.active_page = self.active_page.prev(); }
    pub fn navigate_to(&mut self, key: PageKey) { self.active_page = key; }

    pub fn render_active(&self, renderer: &mut crate::app::services::render::RenderService) {
        if let Some(page) = self.pages.get(&self.active_page) {
            page.render(renderer);
        }
    }
}

// ---- 占位页面 ----
struct HomePage;
impl Page for HomePage {
    fn key(&self) -> PageKey { PageKey::Home }
    fn title(&self) -> &str { "Home" }
    fn render(&self, r: &mut RenderService) {
        r.draw_centered(5, "Welcome to TUI Game Engine");
        r.draw_centered(7, "← → to navigate | ESC to exit");
    }
}

struct PackagesPage;
impl Page for PackagesPage {
    fn key(&self) -> PageKey { PageKey::Packages }
    fn title(&self) -> &str { "Packages" }
    fn render(&self, r: &mut RenderService) {
        r.draw_centered(5, "Package List");
        r.draw_centered(7, "(package list will appear here)");
    }
}

struct SettingsPage;
impl Page for SettingsPage {
    fn key(&self) -> PageKey { PageKey::Settings }
    fn title(&self) -> &str { "Settings" }
    fn render(&self, r: &mut RenderService) {
        r.draw_centered(5, "Settings");
        r.draw_centered(7, "(settings will appear here)");
    }
}
```

### 11.2 在 runtime 中接入导航

```rust
// 页面导航
if services.input.consume_key(KeyCode::Right) {
    services.ui.navigate_next();
}
if services.input.consume_key(KeyCode::Left) {
    services.ui.navigate_prev();
}
```

### 11.3 通过 UiService 渲染

render 中委托给 `services.ui.render_active()`，状态栏显示当前页面名。

## 验证

```bash
cargo build
cargo run
```

- 默认显示 Home 页面
- 左右方向键在 Home → Packages → Settings 之间切换
- 每页显示不同内容
- ESC 退出
