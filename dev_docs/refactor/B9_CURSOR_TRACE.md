# B9.1 input cursor state trace

Date: 2026-09-28

## Frame path

1. `TextInputService::focus` stores one active `(UiObjectPool, TextInputId)`; a pool can have many fields, but only the active field reports a cursor.
2. `TextInputService::render_target` resolves the field against the base, slice, or host surface, clips its visible width/height, renders the existing input buffer, then adds the surface origin to the widget-local cursor coordinate.
3. Single-line layout counts grapheme display-cell widths and scrolls its leading text so the cursor remains in the input box. Multi-line layout wraps by display cells and scrolls the first visible line to keep the cursor line within the box.
4. `route_render` currently returns `Option<(u16, u16)>`. The runtime draws the popup, composes the frame, and passes that option unchanged to `FramePresenter::present`.
5. The presenter writes the frame, resets style, queues the final cursor move when present, then flushes. Cursor output therefore follows frame content and style reset; it does not provide a separate IME candidate-window API.

## Coordinate probes

The new widget tests use an 80×24 physical layout with a 40×12 developer viewport at physical origin `(4, 3)`.

| Input sample | Input rectangle (surface-local) | Returned `final_cursor` | Result |
|---|---:|---:|---|
| Base, `A界`, cursor at end | `(2, 1, 10, 1)` | `(9, 4)` | Wide grapheme uses two terminal cells; viewport origin is added once. |
| Base, multiline `a\nb\nc`, cursor at end, 2-row box | `(3, 4, 5, 2)` | `(8, 8)` | The visible window scrolls to show the final line. |
| Host overlay, `xy`, cursor at end | `(6, 2, 8, 1)` | `(8, 2)` | Host coordinates are already physical, so no developer viewport offset is added. |
| Focused input with zero-width rectangle | `(6, 2, 0, 1)` | `None` | No stale visible rectangle or final cursor is returned. |

These are widget-to-presenter coordinates. They do not claim that each screen currently forwards them or that the IME candidate window follows them.

## Screens that forward the cursor

`route_render` captures and returns cursors for `ToolbarCustom`, `ScreenshotSettings`, `RecordingSettings`, `ScreenshotList`, `RecordingList`, `GameKeyBindings`, `ScreensaverList`, `GameList`, `GamePackage`, `ScreensaverPackage`, and the `ExportSettings` overlay. The screenshot/recording settings wrappers propagate the cursor from their font/input subviews. List/detail pages return the active search or jump input cursor; the export overlay returns its active name or path input cursor.

The runtime filters that coordinate through `InputService::is_focused()` immediately before composing and presenting the frame. Terminal focus loss therefore keeps the text field and buffer active but does not write a background `final_cursor` move. The cursor position remains recomputed by the widget from the current layout each frame, so a resize uses the new geometry on the next render.

## Forwarding gaps closed in B9.3

Several rendered, focusable text fields do not reach `route_render`'s return value:

| Screen/component | Text fields | Current route result |
|---|---|---|
| Home / `GameList` | Search and page jump | `render` returns the active input coordinate. |
| `ScreensaverList` | Search | `draw_search` and `render` return the coordinate. |
| `GameKeyBindings` | Search | `draw_search` and `render` return the coordinate. |
| `GamePackage`, `ScreensaverPackage` | Search and page jump | Package list render returns the active input coordinate. |
| `ExportSettings` overlay | Export name and path | Overlay and route return the active host-surface coordinate. |

The B9.3 code gap is closed without changing input-buffer or clipboard semantics. The regression `terminal_focus_gates_final_input_cursor` covers focused, unfocused, and absent cursors. Component return forwarding is compile-checked across the workspace; actual IME composition remains unverified and is tracked separately as OQ-B9-02.

## Focus and input behavior

- `SystemEvent::FocusLost` calls `HitAreaService::focus_lost`, which clears hover/pressed mouse state. It does not call `TextInputService::blur`; logical text-input focus and its buffer survive window focus loss.
- `sync_input_method_policy` uses `Free` when the terminal window is unfocused or a text input is active, and `ForceAscii` otherwise.
- Clipboard copy/paste is routed through the existing `TextInputService` key handler and `ClipboardService`. No password-masking mode or password text input was found in the current UI/service API; B9 must not invent one as part of cursor work.
- The runtime currently keeps the TerminalService-hidden terminal cursor hidden. The presenter `final_cursor` is an address move for the OS/terminal IME anchor, separate from the widget's simulated block/underline cursor.

Focus-loss cursor policy is implemented per OQ-B9-01: preserve the active field and text, but omit `final_cursor` while the terminal window is unfocused. The policy unit test and full workspace test cover this automatic rule; a desktop IME run is not implied.

## Writer counts

The presenter regression uses a two-cell `AB` frame with final cursor `(1, 0)`:

| Frame input | Bytes | Rust `Write::write` calls | Flushes |
|---|---:|---:|---:|
| Two changed ASCII cells plus frame-end cursor move | 26 | 1 | 1 |
| Static two-cell frame plus cursor move | 10 | 1 | 1 |
| One-cell update plus cursor move | 25 | 1 | 1 |
| Forced full two-cell frame plus cursor move | 26 | 1 | 1 |

The same initial frame without a final cursor is 20 bytes, 11 Rust writes, and 1 flush in the pre-buffer B8.1 implementation. After B8.4 it is still 20 bytes but takes 1 write through the full-accepting in-memory tracking writer. Static/local/full probe counts are asserted by `cursor_output_counts_cover_static_local_and_full_frames`. The live Windows Terminal writer can accept only part of a large buffer, so `write_all` may call its `Write::write` implementation multiple times; these interface counts are not kernel writes. B8.4 baseline counts are documented in `B8_BASELINE.md`.

## Verification

- `cargo test -p tg-service-widget -p tg-service-render-pipeline --locked`: widget 82 passed; render pipeline 16 passed, 1 ignored (interactive B8 benchmark).
- The new widget cases cover developer viewport translation, wide-cell layout, multiline scrolling, host-surface coordinates, and zero-size hiding.
- The new presenter case asserts the exact final-cursor ANSI suffix and 26-byte / 1-write / 1-flush count through the test tracking writer.
- No whole-app IME composition run is included in B9.1; these checks validate the coordinate path only.

## B9.2 live terminal cursor-output probe

The ignored release test `terminal_presenter_cursor_output_baseline` ran in the same Windows Terminal default profile, Maple Mono NF CN, and 179×40 viewport. It rendered an 80×24 frame and sent `final_cursor=(40,12)` each frame. Static, one-cell-change, and full-redraw modes ran for 10 seconds at 30/60/120 FPS (9/9 points, exit code 0). Raw CSV and rerun script are in `E:\Code\tg-test\b9-cursor-baseline\`.

At target 60 FPS:

| Mode | Achieved FPS | p95 present time | Test-process CPU | Bytes / 10 s | Rust writes / 10 s | Flushes |
|---|---:|---:|---:|---:|---:|---:|
| Static frame + cursor move | 58.79 | 0.404 ms | 0.31% | 7,056 | 588 | 588 |
| One-cell update + cursor move | 58.81 | 0.445 ms | 1.09% | 15,888 | 589 | 589 |
| Full frame + cursor move | 58.82 | 1.076 ms | 3.12% | 8,876,230 | 2,356 | 589 |

The final cursor move stays after the rendered frame, and its command does not add another presenter write for an ordinary frame accepted by the live Terminal writer. Larger full-frame data still goes through the same merged output buffer. This is a cursor-output cost comparison; no active composition string was entered, so candidate popup placement, Chinese/Japanese commit/cancel, and real preedit motion remain unverified. The Windows user language list includes Chinese and Japanese input methods, but the active IME mode was not inspected or changed.

B9.2 output-protocol comparison is complete. B9.2's IME composition observations remain pending; they are kept distinct from the automatic timing probe.

## B9.3 cursor forwarding

- Every active host text input identified in the trace now returns its `TextInputService::render_host` coordinate through its component and `route_render` to `FramePresenter`; export name/path remain host physical coordinates.
- The runtime drops the final cursor only when `InputService::is_focused()` is false. It does not blur the text field, clear its buffer, or show the terminal's native cursor. Layout is recalculated on each frame, so the next frame after resize carries the new widget coordinate.
- Added `terminal_focus_gates_final_input_cursor` for focused, unfocused, and absent coordinate cases. `cargo test -p tui-game --locked`: 110 passed. `cargo test --workspace --locked`: passed across the workspace; 2 existing interactive tests ignored by design. `cargo check --workspace --all-targets --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check`, and targeted `git diff --check` passed.
- The UI was not manually exercised with active Chinese Pinyin or Japanese Romaji preedit, candidate selection, or resize while composing. B9.3 automatic forwarding is complete; the real-terminal acceptance is paused as high-impact OQ-B9-02. Continue B10, then return to the B9.3 terminal matrix.
