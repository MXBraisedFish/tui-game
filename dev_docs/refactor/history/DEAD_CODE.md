# Dead code seen before moving code into library crates

Library crates do not report unused `pub` items, so dead code visible in the binary build is
recorded here before each move. Clean up in a dedicated pass (verify each item is really unused).

## animation (captured 2026-09-26, before crates/service/animation)
- `AnimationTargetRouter` trait (service.rs) — never used
- AnimationService: multiple methods never used (service.rs impl at line ~87); `reset_playback_time` fn
- CharacterEffectService: `create`, `remove`, `exists`, `parameter`, `set_parameter`, `clear_override`
- pool.rs: AnimationPlayback.`owner` never read; AnimationPool `remove`, `ids_owned_by`; `resolved`;
  AnimationValuePool `remove`; CharacterEffectPool `insert`, `remove`; AnimationObjects `remove_animations_targeting`
- types.rs: consts MAX_CHARACTER_FRAMES / _WIDTH / _HEIGHT; `CharacterFrame` (+ from_text, display_size,
  normalize, normalized_text_lines, character_frames, character_frames_from_text); variants
  AnimationOwner::{UiPool, Game, Object}, AnimationValue::{Bool, Text, CharacterFrame}, AnimationSource::Clip,
  AnimationEndMode::Restore, event kind Cancelled; `UiObjectRef::new`-like associated fn at types.rs:72
- mod.rs re-exports never used: AnimationClip, AnimationColor, AnimationKeyframe, AnimationMarker,
  AnimationTrack, CharacterFrame, GameInstanceId, GameObjectRef, UiObjectRef, UiPoolId

## main binary (still visible, not moved yet)
- ~30 UI pages own a `runtime_objects` field that is never read
- RuntimeObjectPoolOwner trait never used
- TimeService: many methods unused (take_timer_events, CountDown mode, Paused/Stopped states, repeat modes...)
- EngineServices fields `file`, `character_effect`, `slice`, `ui`, `unicode` never read

## layout (captured 2026-09-26, before crates/service/layout)
- measure.rs: `get_text_height`, `get_draw_text_height`; position.rs `resolve_rect`
- LayoutService: `get_text_height`, `get_draw_text_height`, `reset_developer_viewport`, `resolve_base_y`,
  `resolve_rect`, `resolve_base_rect`, `resolve_host_y`, consts ALIGN_LEFT/CENTER/RIGHT (and more in the
  same "multiple associated items" group)
- Removed already (no callers, undocumented): resolve_slice_x/y/rect, resolve_scroll_box_x/y/rect

## canvas (captured 2026-09-26, before crates/service/canvas)
- buffer.rs `CanvasBuffer::row_text` (only tests)
- CanvasService: `base_width`, `base_height`, `base_size`, `clear`, `top_styled_text`, `cell_at`,
  `prepared_slices`, `prepared_slice_size/width/height`, `prepared_scroll_box_size`,
  `prepared_scroll_box_content_size`, `prepared_scroll_box_viewport_size`,
  `prepared_scroll_box_scroll_position` (several are used only by tests)

## time (captured 2026-09-26, before crates/service/time)
- TimerMode::CountDown, TimerState::{Paused, Stopped}, RepeatMode::{Forever, Count}; DelayTimerOptions,
  RepeatTimerOptions never constructed; event `id()` helpers, `delay_id`/`repeat_id`; `next_id` fields
- Timer `duration`/`remaining`/`progress`; TimeObjects take_*/clear_* event helpers
- SleepTask never constructed / TimeAsyncEvent::SleepFinished never constructed => TimeService::sleep unused
- TimeService: "multiple methods are never used"
- re-exports never used: DelayTimerId, DelayTimerOptions, RepeatMode, RepeatTimerOptions, TimeCallbackId,
  TimeCallbackRequest, TimerMode, TimerOptions

## file (captured 2026-09-26, before crates/service/file)
- FileService: `read_text`, `write_text`, `read_bytes`, `write_bytes` never used (the whole struct is a dead
  wrapper; EngineServices.file never read) => FileTask::{ReadText, WriteText, ReadBytes, WriteBytes} never
  constructed (only Lua* variants are submitted, by lua/api/libraries/{file,i18n}.rs)
- FileEvent: `path` field of every variant is never read (broker only uses task_id + payload)

## image (captured 2026-09-26, before crates/service/image)
- ImageService::convert_async never called => ImageTask / ImageEvent never constructed. The Lua event broker
  still routes LuaTaskOperation::ImageConvert -> LuaImageEvent (documented `image` event in EVENT.md), but no
  Lua API registers an ImageConvert task. Decide in the cleanup pass (or in PLAN2) whether to expose or drop it.

## Removed by crates-quality-B (2026-09-26, clippy -D warnings on library crates)
- layout: `resolve_host_y` removed; `reset_developer_viewport` now `#[cfg(test)]` (only a test uses it)
- canvas: `top_styled_text`, `prepared_slices` removed
- render: `draw_host_text_at`, `draw_top_text_at`, `draw_top_filled_rect` removed
(all were crate-private with no callers; the other items listed above for layout/canvas are still present)
