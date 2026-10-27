//! Exercise signed link geometry, hover styles, activation and asset resolution without the host.
//!
//! Run `cargo run -p tg-service-widget --example link_smoke --locked` for headless assertions.
//! To invoke actual default applications on the current desktop, append `-- --open <asset-root>`.
//! The optional explicit root retains its smoke file/directory so launched applications can read them.
//! Linux/macOS require a configured desktop opener; Windows requires default file/URL associations.

use std::{
  fs,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_core_style::{TerminalColor, TextColor, TextStyle};
use tg_service_canvas::CanvasService;
use tg_service_clipboard::ClipboardService;
use tg_service_layout::LayoutService;
use tg_service_widget::{
  HyperlinkEvent, HyperlinkOptions, HyperlinkService, TextInputService, UiEvent, UiObjectPool,
  is_valid_link_target, open_link_target, resolve_link_target,
};

struct TemporaryRoot(Option<PathBuf>);
impl Drop for TemporaryRoot {
  fn drop(&mut self) {
    if let Some(path) = &self.0 {
      let _ = fs::remove_dir_all(path);
    }
  }
}
pub(crate) fn main() {
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  let open_root = if args.first().is_some_and(|arg| arg == "--open") {
    Some(PathBuf::from(
      args.get(1).expect("supply an explicit asset root"),
    ))
  } else {
    None
  };
  run(open_root);
}
pub(crate) fn run(open_root: Option<PathBuf>) {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let temporary = TemporaryRoot(
    open_root
      .is_none()
      .then(|| std::env::temp_dir().join(format!("tg-link-{}-{nonce}", std::process::id()))),
  );
  let root = open_root
    .clone()
    .unwrap_or_else(|| temporary.0.as_ref().unwrap().join("assets"));
  fs::create_dir_all(root.join("folder")).unwrap();
  fs::write(
    root.join("manual & notes.txt"),
    "TUI GAME default-application link smoke.\n",
  )
  .unwrap();
  for target in [
    "manual & notes.txt",
    "./folder",
    "https://example.com/?a=1&b=2",
  ] {
    assert!(is_valid_link_target(target));
    assert!(resolve_link_target(&root, target).is_some());
  }
  if let Some(deployment) = &temporary.0 {
    let outside = deployment.join("outside");
    fs::create_dir_all(&outside).unwrap();
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_dir(&outside, root.join("escape"));
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(&outside, root.join("escape"));
    #[cfg(any(windows, unix))]
    if linked.is_ok() {
      assert!(resolve_link_target(&root, "escape").is_none());
    } else {
      println!("Directory symlink creation unavailable; escape fixture not exercised");
    }
  }
  let resolved = resolve_link_target(&root, "manual & notes.txt").unwrap();
  assert!(PathBuf::from(&resolved).is_absolute());
  assert!(PathBuf::from(&resolved).ends_with("manual & notes.txt"));
  for invalid in [
    "../outside",
    "/etc/passwd",
    "C:\\outside.txt",
    "file:///outside",
    "mailto:test@example.com",
    "javascript:alert(1)",
    "https://",
    "x\0y",
  ] {
    assert!(!is_valid_link_target(invalid), "{invalid}");
    assert!(resolve_link_target(&root, invalid).is_none());
  }
  assert!(resolve_link_target(&root, "missing").is_none());
  let mut clipboard = ClipboardService::new();
  assert!(!open_link_target(&root, "missing", false, &mut clipboard));
  let mut pool = UiObjectPool::new();
  let service = HyperlinkService::new();
  let input = TextInputService::new();
  let mut options = HyperlinkOptions::new("./folder", "abcd");
  options.hover_style = Some(TextStyle {
    foreground: Some(TextColor::Terminal(TerminalColor::Red)),
    underline: true,
    ..Default::default()
  });
  let id = service.create(&mut pool, options.clone()).unwrap();
  let layout = LayoutService::new();
  let mut canvas = CanvasService::new();
  pool.prepare_canvas(&mut canvas, &layout);
  assert!(service.render_at(&mut pool, id, None, (-1, 0), &mut canvas));
  assert_eq!(canvas.base_buffer().get(0, 0).unwrap().text, "b");
  let mouse = |kind, button| MouseEvent {
    kind,
    button,
    scroll: None,
    x: 0,
    y: 0,
  };
  service.route_mouse_event(&mut pool, &input, mouse(MouseEventKind::Move, None));
  assert!(service.is_hovered(&pool, id));
  pool.begin_render();
  pool.prepare_canvas(&mut canvas, &layout);
  service.render_at(&mut pool, id, None, (-1, 0), &mut canvas);
  assert_eq!(
    canvas.base_buffer().get(0, 0).unwrap().style.foreground,
    Some(TextColor::Terminal(TerminalColor::Red))
  );
  for kind in [MouseEventKind::Press, MouseEventKind::Release] {
    service.route_mouse_event(&mut pool, &input, mouse(kind, Some(MouseButton::Left)));
  }
  assert!(
    matches!(pool.pop_event(),Some(UiEvent::Hyperlink(HyperlinkEvent::Clicked{id:clicked,link})) if clicked==id&&link=="./folder")
  );
  service.route_mouse_event(
    &mut pool,
    &input,
    mouse(MouseEventKind::Press, Some(MouseButton::Left)),
  );
  options.mouse = false;
  assert!(service.set_options(&mut pool, id, options.clone()));
  service.render_at(&mut pool, id, None, (-1, 0), &mut canvas);
  assert!(!service.is_hovered(&pool, id));
  assert_eq!(
    canvas.base_buffer().get(0, 0).unwrap().style.foreground,
    options.style.foreground
  );
  for kind in [
    MouseEventKind::Release,
    MouseEventKind::Press,
    MouseEventKind::Release,
  ] {
    service.route_mouse_event(&mut pool, &input, mouse(kind, Some(MouseButton::Left)));
  }
  assert!(pool.pop_event().is_none());
  assert!(service.click(&mut pool, id));
  assert!(matches!(pool.pop_event(), Some(UiEvent::Hyperlink(_))));
  options.mouse = true;
  assert!(service.set_options(&mut pool, id, options.clone()));
  service.render_at(&mut pool, id, None, (-1, 0), &mut canvas);
  service.route_mouse_event(
    &mut pool,
    &input,
    mouse(MouseEventKind::Release, Some(MouseButton::Left)),
  );
  assert!(pool.pop_event().is_none());
  for kind in [MouseEventKind::Press, MouseEventKind::Release] {
    service.route_mouse_event(&mut pool, &input, mouse(kind, Some(MouseButton::Left)));
  }
  assert!(matches!(pool.pop_event(), Some(UiEvent::Hyperlink(_))));
  for position in [(i32::MIN, 0), (i32::MAX, i32::MAX)] {
    service.render_at(&mut pool, id, None, position, &mut canvas);
    service.route_mouse_event(
      &mut pool,
      &input,
      mouse(MouseEventKind::Press, Some(MouseButton::Left)),
    );
    service.route_mouse_event(
      &mut pool,
      &input,
      mouse(MouseEventKind::Release, Some(MouseButton::Left)),
    );
    assert!(pool.pop_event().is_none());
  }
  assert!(service.click(&mut pool, id));
  assert!(service.remove(&mut pool, id));
  assert!(pool.pop_event().is_none());
  assert!(!service.click(&mut pool, id));
  if open_root.is_some() {
    for target in ["manual & notes.txt", "./folder", "https://example.com"] {
      if target == "manual & notes.txt"
        && let Some(previous) = clipboard.read_text()
      {
        let opened = open_link_target(&root, target, true, &mut clipboard);
        let copied = clipboard.read_text();
        let restored = clipboard.write_text(&previous);
        assert!(restored, "restore the pre-smoke clipboard text");
        assert!(opened, "default launcher rejected {target}");
        assert_eq!(copied.as_deref(), resolved.to_str());
        println!("Resolved-path clipboard copy passed; previous clipboard text restored");
      } else {
        assert!(
          open_link_target(&root, target, false, &mut clipboard),
          "default launcher rejected {target}"
        );
      }
    }
    #[cfg(windows)]
    {
      fs::write(
        root.join("unknown.tg-link-smoke-no-association"),
        "unassociated link smoke",
      )
      .unwrap();
      assert!(!open_link_target(
        &root,
        "unknown.tg-link-smoke-no-association",
        false,
        &mut clipboard
      ));
    }
    println!(
      "Windows/desktop launch requests accepted; smoke assets retained at {}",
      root.display()
    );
  }
  println!("link ok: signed clipping, hover, clicks, target validation and cleanup");
}
