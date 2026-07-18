use super::*;
use crate::host_engine::services::{HostObjectPool, LayoutService, Rect, Size};

pub(super) fn apply_host_viewport(services: &mut EngineServices, top_toolbar: bool) {
  refresh_host_areas(
    &mut services.host_objects,
    services.layout.physical_size(),
    top_toolbar,
  );
  apply_developer_viewport(&mut services.layout, &services.host_objects);
}

fn apply_developer_viewport(layout: &mut LayoutService, host_objects: &HostObjectPool) {
  if let Some(rect) = host_objects.area_rect(HostAreaKind::DeveloperViewport) {
    layout.set_developer_viewport(rect);
  }
}

fn refresh_host_areas(host_objects: &mut HostObjectPool, physical: Size, top_toolbar: bool) {
  let top = host_objects.ensure_area(HostAreaKind::TopBar);
  let separator = host_objects.ensure_area(HostAreaKind::Separator);
  let viewport = host_objects.ensure_area(HostAreaKind::DeveloperViewport);
  let top_height = u16::from(top_toolbar && physical.height > 0);
  let separator_height = u16::from(top_toolbar && physical.height > 1);
  host_objects.update_area(
    top,
    Rect {
      x: 0,
      y: 0,
      width: physical.width,
      height: top_height,
    },
    top_height > 0,
  );
  host_objects.update_area(
    separator,
    Rect {
      x: 0,
      y: top_height,
      width: physical.width,
      height: separator_height,
    },
    separator_height > 0,
  );
  let developer_y = top_height.saturating_add(separator_height);
  host_objects.update_area(
    viewport,
    Rect {
      x: 0,
      y: developer_y,
      width: physical.width,
      height: physical.height.saturating_sub(developer_y),
    },
    true,
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn host_viewport_uses_full_terminal_by_default() {
    let mut layout = LayoutService::new();
    let mut host_objects = HostObjectPool::new();
    layout.resize_physical(120, 40);

    refresh_host_areas(&mut host_objects, layout.physical_size(), false);
    apply_developer_viewport(&mut layout, &host_objects);

    assert_eq!(host_objects.area_rect(HostAreaKind::TopBar), None);
    assert_eq!(host_objects.area_rect(HostAreaKind::Separator), None);
    assert!(host_objects.is_visible(HostAreaKind::DeveloperViewport));
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 40
      }
    );
    assert_eq!(
      layout.developer_size(),
      Size {
        width: 120,
        height: 40
      }
    );
  }

  #[test]
  fn repeated_host_viewport_refresh_keeps_full_terminal_base() {
    let mut layout = LayoutService::new();
    let mut host_objects = HostObjectPool::new();
    layout.resize_physical(120, 40);
    refresh_host_areas(&mut host_objects, layout.physical_size(), true);
    apply_developer_viewport(&mut layout, &host_objects);

    refresh_host_areas(&mut host_objects, layout.physical_size(), true);
    apply_developer_viewport(&mut layout, &host_objects);

    assert!(host_objects.is_visible(HostAreaKind::TopBar));
    assert_eq!(host_objects.area_height(HostAreaKind::Separator), Some(1));
    assert_eq!(
      host_objects.area_width(HostAreaKind::DeveloperViewport),
      Some(120)
    );
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 0,
        y: 2,
        width: 120,
        height: 38
      }
    );
  }
}
