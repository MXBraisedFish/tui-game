//! Frame scene collection, content measurement and hierarchical drawing replay.

use super::*;
use crate::{LuaDrawCommand, LuaDrawTarget};
use tg_service_canvas::{CanvasService, SliceId};
use tg_service_layout::Size;
use tg_service_render::RenderService;
use tg_service_widget::{SliceLength, SliceService, clip_rect};

type Commands = HashMap<LuaDrawTarget, Vec<LuaDrawCommand>>;

pub(crate) enum ReplayStep {
  Draw(LuaDrawCommand),
  Begin(SliceId),
  End(SliceId),
  Effect(crate::LuaEffectRequest),
}

impl LuaObjectPool {
  /// Replay one frame with ancestor clipping and container-local drawing order.
  ///
  /// Prepare the canvas from this pool before calling this method. Content submissions may
  /// precede their container's draw call; the last placement of each container wins.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used for text and geometry.
  /// * `canvas` - The current frame's prepared canvas.
  /// * `commands` - The ordered script submissions for this frame.
  ///
  /// # Errors
  ///
  /// Reject shader submissions, which require [`crate::LuaSession::replay_draw_commands`]
  /// to supervise Lua callbacks and release pool borrows before calling script code.
  pub fn replay_draw_commands(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    commands: Vec<LuaDrawCommand>,
  ) -> mlua::Result<()> {
    if commands
      .iter()
      .any(|command| matches!(command, LuaDrawCommand::Effect(_)))
    {
      return Err(mlua::Error::RuntimeError(
        "cell shaders require LuaSession::replay_draw_commands".into(),
      ));
    }
    let steps = self.prepare_draw_commands(canvas, commands);
    let mut skipped = 0;
    for step in steps {
      self.replay_step(render, canvas, step, &mut skipped);
    }
    Ok(())
  }

  pub(crate) fn prepare_draw_commands(
    &mut self,
    canvas: &mut CanvasService,
    commands: Vec<LuaDrawCommand>,
  ) -> Vec<ReplayStep> {
    let mut grouped = Commands::new();
    let mut canvas_effects = Vec::new();
    for command in commands {
      if let LuaDrawCommand::Effect(request) = &command
        && request.effect.scope == crate::LuaEffectScope::Canvas
      {
        canvas_effects.push(ReplayStep::Effect(request.clone()));
        continue;
      }
      grouped
        .entry(command_target(&command))
        .or_default()
        .push(command);
    }
    for binding in self.scrollboxes.values_mut() {
      binding.widget.begin_frame();
    }
    let size = Size {
      width: canvas.viewport().width,
      height: canvas.viewport().height,
    };
    let mut measured = std::collections::HashSet::new();
    self.measure_surface(LuaDrawTarget::Base, size, &grouped, &mut measured, 0);
    // Undrawn containers still retain the bounds of content submitted to them this frame.
    for surface in self
      .scrollboxes
      .values()
      .map(|binding| binding.surface)
      .collect::<Vec<_>>()
    {
      self.measure_surface(
        LuaDrawTarget::Slice(surface),
        size,
        &grouped,
        &mut measured,
        0,
      );
    }
    for (id, placement) in &self.surface_placements {
      if placement.parent != LuaDrawTarget::Base {
        canvas.detach_slice(*id);
      }
    }
    self.ui.refresh_pointer_surfaces(canvas);
    let mut steps = Vec::new();
    self.plan_surface(LuaDrawTarget::Base, &grouped, &mut steps, 0);
    for id in SliceService::new().ids_by_layer(&self.ui) {
      if self
        .surface_placements
        .get(&id)
        .is_none_or(|placement| placement.parent == LuaDrawTarget::Base)
      {
        self.plan_surface(LuaDrawTarget::Slice(id), &grouped, &mut steps, 0);
      }
    }
    steps.extend(canvas_effects);
    steps
  }

  fn measure_surface(
    &mut self,
    target: LuaDrawTarget,
    parent_size: Size,
    commands: &Commands,
    measured: &mut std::collections::HashSet<SliceId>,
    depth: usize,
  ) -> Size {
    if depth > 32 {
      return Size::default();
    }
    let scrollbox = match target {
      LuaDrawTarget::Slice(id) => self.scrollbox_id_for_surface(id),
      _ => None,
    };
    if let LuaDrawTarget::Slice(id) = target
      && !measured.insert(id)
    {
      return scrollbox.map_or(Size::default(), |id| {
        self.scrollboxes[&id].widget.layout().outer
      });
    }
    let available = if let Some(id) = scrollbox {
      // Relative child sizes use the configured viewport, independent of measured overflow.
      let content = self.scrollboxes[&id]
        .widget
        .layout_for(Size::default())
        .content;
      Size {
        width: content.width,
        height: content.height,
      }
    } else if let LuaDrawTarget::Slice(id) = target {
      self.static_size(id, parent_size)
    } else {
      parent_size
    };
    let mut extent = Size::default();
    for command in commands.get(&target).into_iter().flatten() {
      let (x, y, size) = match command {
        LuaDrawCommand::Surface {
          id, x, y, sequence, ..
        } => {
          if self
            .surface_placements
            .get(id)
            .is_none_or(|placement| placement.sequence != *sequence)
          {
            continue;
          }
          (
            *x,
            *y,
            self.measure_surface(
              LuaDrawTarget::Slice(*id),
              available,
              commands,
              measured,
              depth + 1,
            ),
          )
        }
        LuaDrawCommand::Text { x, y, params, .. } => {
          let (width, height) = tg_service_text_layout::measure_draw_text(params);
          (*x, *y, Size { width, height })
        }
        LuaDrawCommand::FillRect {
          x,
          y,
          width,
          height,
          ..
        }
        | LuaDrawCommand::StrokeRect {
          x,
          y,
          width,
          height,
          ..
        } => (
          *x,
          *y,
          Size {
            width: *width,
            height: *height,
          },
        ),
        LuaDrawCommand::Progressbar { id, x, y, .. } => {
          let Some(binding) = self.progressbars.get(id) else {
            continue;
          };
          (
            *x,
            *y,
            Size {
              width: binding.width,
              height: 1,
            },
          )
        }
        LuaDrawCommand::Hitbox { id, x, y, .. } => {
          let Some(binding) = self.hitboxes.get(id) else {
            continue;
          };
          (
            *x,
            *y,
            Size {
              width: binding.width,
              height: binding.height,
            },
          )
        }
        LuaDrawCommand::Input { id, x, y, .. } => {
          let Some(binding) = self.inputs.get(id) else {
            continue;
          };
          (
            *x,
            *y,
            Size {
              width: binding.params.rect.width,
              height: binding.params.rect.height,
            },
          )
        }
        LuaDrawCommand::Link { id, x, y, .. } => {
          let Some(binding) = self.links.get(id) else {
            continue;
          };
          let (width, height) = tg_service_text_layout::measure_draw_text(
            &tg_service_text_layout::DrawTextParams::new(0, 0, &binding.options.text),
          );
          (*x, *y, Size { width, height })
        }
        LuaDrawCommand::EraseRect { .. } | LuaDrawCommand::Effect(_) => continue,
      };
      if size.width > 0
        && size.height > 0
        && i64::from(x) + i64::from(size.width) > 0
        && i64::from(y) + i64::from(size.height) > 0
      {
        extent.width = extent
          .width
          .max((i64::from(x) + i64::from(size.width)).clamp(0, i64::from(u16::MAX)) as u16);
        extent.height = extent
          .height
          .max((i64::from(y) + i64::from(size.height)).clamp(0, i64::from(u16::MAX)) as u16);
      }
    }
    if let Some(markdown) = scrollbox.and_then(|id| self.scrollboxes[&id].markdown) {
      extent = self.prepare_markdown(markdown, extent);
    }
    if let Some(id) = scrollbox {
      let widget = &mut self.scrollboxes.get_mut(&id).unwrap().widget;
      widget.set_extent(extent);
      widget.layout().outer
    } else {
      available
    }
  }

  fn static_size(&self, id: SliceId, parent: Size) -> Size {
    let Some(rect) = SliceService::new().configured_rect(&self.ui, id) else {
      return Size::default();
    };
    let resolve = |length, available| match length {
      SliceLength::Fixed(value) => value,
      SliceLength::Auto => available,
      SliceLength::Percent(value) => (u32::from(available) * u32::from(value) / 100) as u16,
    };
    Size {
      width: resolve(rect.width, parent.width),
      height: resolve(rect.height, parent.height),
    }
  }

  fn configured_surface_size(&self, target: LuaDrawTarget, base: Size, depth: usize) -> Size {
    if depth > 32 {
      return Size::default();
    }
    let LuaDrawTarget::Slice(id) = target else {
      return base;
    };
    if let Some(lua_id) = self.scrollbox_id_for_surface(id) {
      let content = self.scrollboxes[&lua_id]
        .widget
        .layout_for(Size::default())
        .content;
      return Size {
        width: content.width,
        height: content.height,
      };
    }
    let parent = self
      .surface_placements
      .get(&id)
      .map_or(LuaDrawTarget::Base, |placement| placement.parent);
    self.static_size(id, self.configured_surface_size(parent, base, depth + 1))
  }

  fn plan_surface(
    &self,
    target: LuaDrawTarget,
    commands: &Commands,
    steps: &mut Vec<ReplayStep>,
    depth: usize,
  ) {
    if depth > 32 {
      return;
    }
    for command in commands.get(&target).into_iter().flatten() {
      if let LuaDrawCommand::Surface { id, sequence, .. } = command {
        if self.scrollbox_id_for_surface(*id).is_some()
          && self
            .surface_placements
            .get(id)
            .is_some_and(|placement| placement.sequence == *sequence)
        {
          self.plan_embedded(*id, commands, steps, depth + 1);
        }
      } else if !matches!(command, LuaDrawCommand::Effect(_)) {
        steps.push(ReplayStep::Draw(command.clone()));
      }
    }
    // User slices keep their explicit layer order above ordinary parent drawing.
    for id in SliceService::new().ids_by_layer(&self.ui) {
      if target != LuaDrawTarget::Base
        && self
          .surface_placements
          .get(&id)
          .is_some_and(|placement| placement.parent == target)
      {
        self.plan_embedded(id, commands, steps, depth + 1);
      }
    }
    for command in commands.get(&target).into_iter().flatten() {
      if let LuaDrawCommand::Effect(request) = command {
        steps.push(ReplayStep::Effect(request.clone()));
      }
    }
  }

  fn plan_embedded(
    &self,
    id: SliceId,
    commands: &Commands,
    steps: &mut Vec<ReplayStep>,
    depth: usize,
  ) {
    if depth > 32 {
      return;
    }
    steps.push(ReplayStep::Begin(id));
    self.plan_surface(LuaDrawTarget::Slice(id), commands, steps, depth);
    steps.push(ReplayStep::End(id));
  }

  pub(crate) fn replay_step(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    step: ReplayStep,
    skipped: &mut usize,
  ) {
    match step {
      ReplayStep::Begin(id) => {
        if *skipped > 0 || !self.begin_embedded(id, canvas) {
          *skipped += 1;
        }
      }
      ReplayStep::End(id) => {
        if *skipped > 0 {
          *skipped -= 1;
        } else {
          let parent =
            self
              .surface_placements
              .get(&id)
              .and_then(|placement| match placement.parent {
                LuaDrawTarget::Base => None,
                LuaDrawTarget::Slice(id) => Some(id),
              });
          canvas.composite_embedded_slice(id, parent);
          self.ui.end_pointer_scope();
        }
      }
      ReplayStep::Draw(command) if *skipped == 0 => self.replay_command(render, canvas, command),
      _ => {}
    }
  }

  fn begin_embedded(&mut self, id: SliceId, canvas: &mut CanvasService) -> bool {
    let Some(placement) = self.surface_placements.get(&id).copied() else {
      return false;
    };
    let parent = match placement.parent {
      LuaDrawTarget::Base => None,
      LuaDrawTarget::Slice(id) => Some(id),
    };
    if let Some(lua_id) = self.scrollbox_id_for_surface(id) {
      let binding = self.scrollboxes.get_mut(&lua_id).unwrap();
      let Some(view) = binding
        .widget
        .begin_draw(&mut self.ui, canvas, parent, placement.position)
      else {
        return false;
      };
      canvas.prepare_embedded_slice(id, view.clip, view.origin, view.rank, None, false);
    } else {
      let service = SliceService::new();
      if !service.is_visible(&self.ui, id) {
        return false;
      }
      let Some((parent_clip, parent_origin, rank)) = canvas.surface_geometry(parent) else {
        return false;
      };
      let size = self.configured_surface_size(
        LuaDrawTarget::Slice(id),
        Size {
          width: canvas.viewport().width,
          height: canvas.viewport().height,
        },
        0,
      );
      let origin = (
        i64::from(parent_origin.0) + i64::from(placement.position.0),
        i64::from(parent_origin.1) + i64::from(placement.position.1),
      );
      let clip = clip_rect(origin, size, parent_clip);
      if clip.width == 0 || clip.height == 0 {
        return false;
      }
      let scope = if let Some(scope) = self.slice_scopes.get(&id) {
        *scope
      } else {
        let scope = self.ui.allocate_pointer_scope();
        self.slice_scopes.insert(id, scope);
        scope
      };
      if !self.ui.begin_pointer_scope(scope, clip) {
        return false;
      }
      if !service.event_passthrough(&self.ui, id).unwrap_or(false) {
        self.ui.register_pointer_region(rank, clip);
      }
      canvas.prepare_embedded_slice(
        id,
        clip,
        (origin.0 as i32, origin.1 as i32),
        rank,
        service.background(&self.ui, id).flatten(),
        service.is_opaque(&self.ui, id),
      );
    }
    if let Some(markdown) = self
      .scrollbox_id_for_surface(id)
      .and_then(|id| self.scrollboxes[&id].markdown)
    {
      let binding = &self.markdowns[&markdown];
      tg_service_widget::MarkdownService::new().render_layout_on(
        &mut self.ui,
        binding.native,
        id,
        &binding.layout,
        canvas,
      );
    }
    true
  }

  fn replay_command(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    command: LuaDrawCommand,
  ) {
    match command {
      LuaDrawCommand::Surface { .. } | LuaDrawCommand::Effect(_) => {}
      LuaDrawCommand::Progressbar { id, target, x, y } => {
        self.draw_progressbar(id, target, (x, y), canvas)
      }
      LuaDrawCommand::Link { id, target, x, y } => self.draw_link(id, target, (x, y), canvas),
      LuaDrawCommand::Input { id, target, x, y } => self.draw_input(id, target, (x, y), canvas),
      LuaDrawCommand::Hitbox { id, target, x, y } => self.draw_hitbox(id, target, (x, y), canvas),
      LuaDrawCommand::Text {
        target,
        x,
        y,
        params,
      } => match target {
        LuaDrawTarget::Base => render.draw_text_at(canvas, x, y, &params),
        LuaDrawTarget::Slice(id) => {
          render.draw_text_at_on(canvas, id, x, y, &params);
        }
      },
      LuaDrawCommand::FillRect {
        target,
        x,
        y,
        width,
        height,
        fill_char,
        fg,
        bg,
      } => match target {
        LuaDrawTarget::Base => {
          render.draw_filled_rect(canvas, x, y, width, height, fill_char, fg, bg)
        }
        LuaDrawTarget::Slice(id) => {
          render.draw_filled_rect_on(canvas, id, x, y, width, height, fill_char, fg, bg);
        }
      },
      LuaDrawCommand::StrokeRect {
        target,
        x,
        y,
        width,
        height,
        border,
        fg,
        bg,
      } => match target {
        LuaDrawTarget::Base => {
          render.draw_border_rect(canvas, x, y, width, height, &border, fg, bg, None, None)
        }
        LuaDrawTarget::Slice(id) => {
          render.draw_border_rect_on(canvas, id, x, y, width, height, &border, fg, bg, None, None);
        }
      },
      LuaDrawCommand::EraseRect {
        target,
        x,
        y,
        width,
        height,
      } => match target {
        LuaDrawTarget::Base => canvas.erase_rect(x, y, width, height),
        LuaDrawTarget::Slice(id) => {
          canvas.erase_rect_on(id, x, y, width, height);
        }
      },
    }
  }
}

fn command_target(command: &LuaDrawCommand) -> LuaDrawTarget {
  match command {
    LuaDrawCommand::Effect(request) => request.target,
    LuaDrawCommand::Progressbar { target, .. }
    | LuaDrawCommand::Surface { target, .. }
    | LuaDrawCommand::Link { target, .. }
    | LuaDrawCommand::Input { target, .. }
    | LuaDrawCommand::Hitbox { target, .. }
    | LuaDrawCommand::Text { target, .. }
    | LuaDrawCommand::FillRect { target, .. }
    | LuaDrawCommand::StrokeRect { target, .. }
    | LuaDrawCommand::EraseRect { target, .. } => *target,
  }
}
