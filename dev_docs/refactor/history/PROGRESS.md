# Refactor progress (handoff log)

Read this first when resuming. PLAN.md = task, CLAUDE.md = workflow limits.
User decisions: see memory `refactor-plan-decisions` (workspace crates, rewrite test_package, local commits + periodic push to origin/dev, never touch main, no AI co-author lines).
NEVER read/produce images (incl. preview_screenshot) — session freezes.

## Step 1 — audit + visualization: DONE (2026-09-24)
- Tooling in temp/audit (extractor, resolve.py, units.py, build_site.py, check_desc.py).
- All 10 description batches validate OK; 4764/4764 items described, 272 findings.
- Site: architecture/ (index.html, app.js, style.css, data/before.js). Views verified via text-only preview tools.
- `data/after.js` is intentionally missing until the refactor ends (page requests it optionally -> 404 is expected).

## Step 2 — migration refactor: IN PROGRESS
- [x] Rewrite test_package (commit d92ef7c).
- [x] Baseline: cargo test 602 pass / 0 fail / 1 ignored.
- [ ] Workspace design (crates/ layout, core list, service list, cycle breaking) -> MIGRATION_MAP.md
- [ ] Cores migrated (tag refactor/core-<name>-done each) — done: package_id, fault, geometry, version, atomic_fs, arena, style, input, crash(->fault), log, unicode, style+cell/frame
- [ ] Services migrated (tag refactor/service-<name>-done each)
- [ ] Main loop / app layer split
- [ ] Full regression: build --release, test, clippy -D warnings, real launch + example script
- [ ] after.js snapshot

## Log
- 2026-09-24 18:xx: resumed from stuck session; finished B07 (153 items + 26 unit summaries), moved stray b07 drafts to temp/handoff, rebuilt before.js, verified site.

- 2026-09-25: test packages rewritten (host ipairs/pairs yield {index,value}); star_drift/pulse_grid kept light to fit Init budget under parallel tests.

- 2026-09-25: workspace created; cores migrated+tagged: package_id, fault, geometry (pushed with tags). Pattern: add behavior test at old path -> git mv -> crate Cargo.toml + examples/smoke.rs -> old path re-exports -> workspace test + release build -> MIGRATION_MAP row -> tag.
  Next cores: style (rich_text style/color/types), input types, version, fs (storage/atomic), arena (animation/pool). crash must invert TerminalService dependency before joining fault.

## After PLAN.md
- User (2026-09-25): when PLAN.md is fully done, start PLAN2.md on my own (no need to ask).
- PLAN2.md changes the manifest format (package/display/game/screensaver/actions json), removes safe mode,
  aligns sandboxed Lua built-ins (e.g. ipairs/pairs currently yield {index,value} records) -> test packages will be revised then.

- 2026-09-25: style core migrated+tagged (resumed from stuck session; files were already git-mv'd). Removed leftover unused Arena re-export in animation/mod.rs.
  Next cores: input types, fs (storage), then crash (invert TerminalService dep before joining fault).
- 2026-09-25: input core migrated+tagged (events/key_token/action_map + Key..InputActionEvent block from service.rs; InputService stays as service). NOTE: never `git stash` — user keeps uncommitted dev_docs/.gitignore/PLAN2.md edits in the tree.
  Next cores: fs (storage), then crash (invert TerminalService dep before joining fault).
- 2026-09-25: crash merged into fault core; install_panic_hook(restore_terminal: fn()) — main loop injects TerminalService::force_restore.
- 2026-09-25: log types core migrated+tagged. Candidate survey done; next: task ids (split out of async_runtime.rs), audio/types.rs, unicode measure (RichText link), canvas cell (needs style).
- 2026-09-25: unicode measure core migrated+tagged (dead RichText link removed first). Deferred to service phase: task ids (async_runtime split; TaskCancellation cfg(test) ctor), audio types (users depend on audio service anyway), UnicodeService dead methods. Next: canvas cell/frame (depends on style).
- 2026-09-25: CanvasCell + ComposedFrame merged into style core. Core candidate list from survey exhausted (remaining ones deferred to service phase). Next: start service migration (bottom-up).
- 2026-09-25: service phase started. Design + decision in temp/handoff/SERVICE_DESIGN.md (user: structure is theoretical, decide pragmatically). Order: leaf services -> cheap cycle breaks -> re-graph -> async_runtime.
- 2026-09-25: service terminal (+capabilities) migrated+tagged; crates/service/* added to workspace.
- 2026-09-25: service clipboard migrated+tagged. Helper temp/handoff/move_service.py moves single-file services.
- 2026-09-25: service ffmpeg migrated+tagged.
- 2026-09-25: service input_method migrated+tagged.
- 2026-09-25: service popup migrated+tagged.
- 2026-09-25: service host_object migrated+tagged.
- 2026-09-25: service code_highlight migrated+tagged. Local msys2 gcc (cc1.exe) is broken -> any NEW C compile fails (tree-sitter grammars); cached artifacts still work. Verify examples via `cargo build --workspace --examples` then run target/debug/examples/smoke-*.exe. Examples all named smoke -> filename collision warning (rename later).
- 2026-09-25: examples renamed to <crate>_smoke.rs (no more collision). Verify all: `cargo build --workspace --examples` then run target/debug/examples/<name>_smoke.exe (17/17 pass).
- 2026-09-25: cycle cut rich_text<->text_layout (TextMode moved into rich_text/service.rs). Cycle-cut simulation: needed cuts = async_runtime out-edges, widget ids (layout/canvas/render/animation->widget), log->i18n/file, rich_text<->text_layout (done). Audio types not needed.
- 2026-09-25: service rich_text migrated+tagged.
- 2026-09-25: service text_layout migrated+tagged.
- 2026-09-25: cycle cut log->i18n/file: LogService::refresh_labels(translate, templates); I18nService::apply_log_translations(&mut log) is the glue; append_text moved into log.
- 2026-09-25: service log migrated+tagged.
- 2026-09-26: audio data model core extracted+tagged (storage no longer needs the audio service).
- 2026-09-26: service storage migrated+tagged. Pattern for cfg(test) helpers used by other crates: `test-support` feature + root [dev-dependencies] entry.
- 2026-09-26: service i18n migrated+tagged (build.rs moved into the crate). WARNING: changing build-dependency features invalidates tree-sitter C builds -> fails on the broken local gcc; keep build-dep specs stable.
- 2026-09-26: i18n commit was split (f494d0a renames only, 7368560 rest) because `git add build.rs` failed after git mv; tag refactor/service-i18n-done moved to 7368560 and force-pushed (tag only). Lesson: never list old paths of git-mv-ed files in `git add`; use `git add -A <dirs>`.
- 2026-09-26: widget-cycle step 1 done: random owns RandomGeneratorObjects; RandomService takes &mut RandomGeneratorObjects. Helper temp/handoff/fix_arg.py rewrites mismatched pool args from compiler JSON.
- 2026-09-26: service random migrated+tagged.
- 2026-09-26: widget-cycle step 2 done: time owns TimeObjects (timers/delay/repeat/callback requests + helpers); RuntimeObjectPool.time; TimeService keeps param name `pool` typed &mut TimeObjects. time still depends on async_runtime (sleep) -> async phase.
- 2026-09-26: widget-cycle step 3 done: animation owns AnimationObjects (+remove_animations_targeting); RuntimeObjectPool = {time, random_generators, animation}. Noted: ~30 UI pages hold a never-read runtime_objects field (dead, pre-existing) -> cleanup later.
- 2026-09-26: service animation migrated+tagged. Dead-code list kept in temp/handoff/DEAD_CODE.md (capture before each move).
- 2026-09-26: widget-cycle step 4 done: LayoutService resolve_slice_*/resolve_scroll_box_* (no callers, undocumented, only own tests) deleted -> layout no longer depends on widget.
- 2026-09-26: service layout migrated+tagged (found undeclared crossterm dependency). Policy: widen only externally used pub(crate) items; dead ones stay crate-private so the crate warns.
- 2026-09-26: widget-cycle step 5 done: SliceId/ScrollBoxId/SurfaceId/ScrollbarStyle/ScrollbarSide/ResolvedScrollBoxLayout live in canvas/surface.rs (widget re-exports at old paths); CanvasService::prepare(pool_id, Vec<SurfaceFrame>, layout); UiObjectPool::prepare_canvas builds frames. canvas no longer depends on widget.
- 2026-09-26: service canvas migrated+tagged. widget-side cycle now fully broken (random/time/animation/layout/canvas no longer depend on widget).
- 2026-09-26: executor extracted to crates/service/async (generic AsyncRuntime<E>, AsyncJob<E>, TaskStatusEvent); main aliases AsyncRuntime<EngineEvent>. Next: per-service jobs (file/image in async_runtime.rs, network, time sleep, ...) to remove EngineTask variants.
- 2026-09-26: sleep job moved into time (SleepTask: AsyncJob<E>, TimeAsyncEvent owned by time; EngineTask::Sleep removed). RULE: run `cargo build --all-targets` + tests BEFORE committing; never chain `git commit` after `;`.
- 2026-09-26: service time migrated+tagged.
- 2026-09-26: core sandbox_path extracted+tagged (lua::path).
- 2026-09-26: (resumed from a session killed by gateway 524/ECONNREFUSED mid-edit; its half-done file-task move was intact in the tree + temp/handoff drafts, finished it and deleted the drafts.)
  file task moved into the file service (FileTask: AsyncJob<E>; EngineTask::File removed; EngineEvent: From<FileEvent>); temp path for the write barrier reuses atomic_fs::temporary_path (now pub). Rename committed separately first (dda6a32) so history follows.
- 2026-09-26: service file migrated+tagged. Workspace: 615 tests pass / 1 ignored, 31/31 smoke examples, release OK, main warnings 176 -> 164.
  NOTE: rustfmt drift exists in ~40 files from earlier steps (only touched files are formatted per step) -> run `cargo fmt --all` in the Step 2.4 regression pass, as its own commit.
  Remaining EngineTask variants: Package, Export, Screenshot, Recording, Video, Image, Network. Next: image (task lives in async_runtime.rs, smallest), then the others one by one.
- 2026-09-26: image task moved into the image service (ImageTask: AsyncJob<E>; EngineTask::Image removed; 2 async behavior tests added), then service image migrated+tagged. 617 pass / 1 ignored, 32/32 smokes, release OK, main warnings 163.
  NOTE: the async image path is dead (convert_async has no caller; broker still routes ImageConvert) -> DEAD_CODE.md.
- 2026-09-26: user asked to speed up (cargo-only checks per small migration, batch full verification at the end, use subagents). Done since: network crate; render + render_pipeline crates (9 widget-driven compositor tests moved to widget/ui_object/surfaces/pipeline_tests.rs); dead UnicodeService/UiService deleted; export/screenshot/recording/package/video tasks became service-owned AsyncJobs -> EngineTask deleted, async_runtime.rs = EngineEvent aggregate only. 617 pass. Helpers: temp/handoff/{mv_crate,fix_imports,jobify,jobify_step2,fix_runtime_imports}.py.
  Subagents launched (no commits, review then commit): crates-quality-A/B (comments -> English + clippy per crate), doc-drift (DOC_DRIFT.md), ui-quality (ui/** comments + clippy). When committing my own work, NEVER `git add -A src/host_engine` (would pick up ui agent edits) — add explicit paths.
- 2026-09-26: screenshot/recording/video/export/package migrated+tagged (one amended commit; package test reads test_package via CARGO_MANIFEST_DIR/../../../); 18 unused root deps dropped. 617 pass / 1 ignored, main warnings 134.

## Conventions for the STATUS agent (added 2026-09-26)
- temp/handoff/STATUS.md is now owned by a background status agent (rewritten every ~15 min). The main session no longer edits it; it appends facts here instead.
- `METRICS: <date time> | tests <pass>/<fail>/<ignored> | main-warnings <n> | smokes <n> | note` — latest line = current numbers (only the main session runs cargo).
- `AGENT <name>: <running|done|merged|failed> - <short note>` — latest line per name = that subagent's state.
- `PLAN1 DONE` on its own line stops the status agent (so does creating temp/handoff/STATUS_AGENT_STOP).
METRICS: 2026-09-26 17:10 | tests 617/0/1 | main-warnings 134 | smokes 40 | EngineTask removed; 28 service + 12 core crates
AGENT crates-quality-A: running - core crates + 12 early service crates: comments to English, clippy -D warnings
AGENT crates-quality-B: running - 11 later service crates: comments to English, clippy -D warnings
AGENT doc-drift: running - writing DOC_DRIFT.md (read-only on docs)
AGENT ui-quality: running - ui/** comments to English + non-dead-code clippy
- 2026-09-26: audio (EventSink<AudioAsyncEvent> instead of Sender<EngineEvent>) + input (generic listeners, InputListenerError) migrated+tagged; tg-service-async gained EventSink. 619 pass.
  LESSONS: `set -e` is NOT honored by this Bash tool (commands after a failure still run) -> chain with `&&`. Windows python cannot open Git-Bash `/tmp/...` paths -> use temp/handoff/ for scratch files. To commit only my lines of a file a subagent is also editing: build the blob from HEAD + my edit, `git hash-object -w`, `git update-index --cacheinfo`.
METRICS: 2026-09-26 17:40 | tests 619/0/1 | main-warnings ~134 | smokes 42 | services left in main: widget, lua (+ event/async_runtime glue)
- 2026-09-26/27: widget (widen_private.py widens exactly the externally used crate-private items from compiler "defined here" spans + private-field errors) and lua (broker takes LuaRoutableEvent<'a>; EngineEvent::lua_routable() in the app layer) migrated+tagged. crates-quality-B merged (ed4221a) + test fixes (b6cf777: test-support dev features, unique temp names). ALL services are now crates; src/host_engine/services = facade (mod.rs) + app-layer glue (async_runtime.rs = EngineEvent, event.rs = EngineEventQueue).
  NOTE: `cargo test -p tg-service-video` alone rebuilds openh264-sys2 (C) because cc features differ outside the workspace -> fails on the broken local gcc; test it via the workspace.
METRICS: 2026-09-27 00:40 | tests 619/0/1 | main-warnings ? | smokes 45 | Step 2.2 done (all services migrated); next: Step 2.3 app layer split
AGENT crates-quality-B: merged - 11 crates, commit ed4221a
AGENT crates-quality-A: merged - 24 crates (core + 12 services), commit after e31cf9e

## 2026-09-27 — A0 handoff and A1 application split
- Base: `dev`, HEAD `1cf1227`; all 28 inherited dirty paths matched `P_PLAN.md`. Kept the user-edited `.gitignore` and three `dev_docs/zh_cn/**` files unchanged.
- Reviewed the 24 inherited UI paths. The storage-management module is a move with the same function bodies; the window-size hint selection, overlay widths, and UI iterator/condition simplifications preserve their prior rules. Added `filtered_rows_keep_visible_hit_areas_after_count_and_scroll_changes` to pin filtered hit-area resizing, scroll row mapping and click selection.
- A1 moved EngineEvent/EngineEventQueue, EngineServices, BootOutput, RuntimeWorld and state_machine under `src/host_engine/app/`. Existing runtime business modules moved under `app/runtime/`; the lifecycle phase module now delegates to the application entry points. Initialization order and event conversion/queue code were preserved.
- Verification: `cargo build --workspace --all-targets --locked` passed after the moves; `cargo test --workspace --locked` passed (619 passed, 0 failed, 1 ignored) before the added UI regression; targeted UI regression passed (1 passed, 98 filtered out). A second `cargo build --workspace --all-targets --locked` and workspace test run also passed after moving runtime code.
- Current state: A1 remains in progress; app/runtime still needs a boundary/ownership review. Formatting and strict Clippy have not been rerun; A2/A3 are pending. No commits or pushes were made.
- A1 closeout: app/core/services/phase imports were checked; application modules no longer import boot, shutdown or the lifecycle runtime module, and core/services no longer own host state or EngineServices. Added BootOutput partial-fault retention and normal stop-flow tests; the fault-domain and Lua-generation regressions remain in their existing owners. Added an active-write wait test to `tg-service-async`.
- Final A1 verification: `cargo build --workspace --all-targets --locked` passed; `cargo test --workspace --locked` passed with 623 passed / 0 failed / 1 ignored. The FFmpeg H.264/AAC test remains ignored as documented. `git diff --check` only reports trailing whitespace in the inherited user-edited `dev_docs/zh_cn/package字段.md`; that document was left unchanged.
- A1 is complete. Next: A2 strict Clippy/dead-code review, formatting as a separate change, and `DOC_DRIFT.md`; then A3 full acceptance and architecture snapshot. No commits or pushes were made.

## 2026-09-27 — A2 quality and documentation audit
- Grouped runtime input/page routing and game-list/package/font rendering dependencies into local context structs; removed remaining too_many_arguments findings without crate-wide lint suppressions.
- Removed unused EngineServices.file, character_effect, and slice fields; kept test-covered service APIs and deferred the unregistered image path to B6; interactive event translators and callback registration are test-only.
- Ran cargo fmt --all as a separate formatting pass. cargo fmt --all -- --check passed; cargo clippy --workspace --all-targets --locked -- -D warnings passed.
- Added tracked DOC_DRIFT.md with the event/callback producer gap, absent image registration, legacy package fields, Q5 key display, rich-text grammar mismatch, file byte-note typo, and serialization inventory.
- git diff --check still reports two trailing-whitespace lines in inherited dev_docs/zh_cn/package字段.md; preserved that protected document.
- A2 is complete. A3 is in progress: next run workspace build/test/release, inventory and inspect all metadata examples before running, then update architecture tooling/snapshot. No commits or pushes were made.

## 2026-09-27 — A3 automated regression and architecture snapshot
- Re-ran `cargo fmt --all -- --check` and `cargo build --workspace --all-targets --locked`; both exited 0. Final `cargo test --workspace --locked` passed 623 / failed 0 / ignored 1; strict workspace Clippy exited 0; `cargo build --release --locked` exited 0.
- Read-only audited side effects for every metadata-listed example, then ran all 44 by package with `cargo run --quiet --locked --package <package> --example <example>`; 44/44 exited 0. Unique temporary roots are created with `create_dir`; clipboard is restored when readable; storage restores process CWD.
- Confirmed `E:\Code\tg-test\ffmpeg.exe` exposes `libx264` and `aac`; filter listing selected exactly one ignored test. `cargo test --workspace --locked recording_with_audio_exports_as_h264_aac_mp4 -- --ignored` passed 1 / failed 0.
- Added reproducible architecture generation under `architecture/`: `build_after.py`, `README.md`, and copied extractor/resolver/site-builder/unit tools. `python architecture/build_after.py` exited 0; `after.js` covers 45/45 workspace packages, 265 source files, 105,487 lines, 4,782 items and 127 cross-package edges. All edge endpoints resolve; dependency checks report 0 cycles/layer violations. The flow view references the current host_engine/app phase boundary. Item descriptions present in source/doc metadata: 576/4,782.
- Headless Edge text-DOM check passed without screenshots: loaded `after`, rendered flow/tree/graph, switched to the `before` 111-node snapshot and back, located `service/tg_service_video`, focused its graph node/panel, and observed no browser runtime or console errors.
- Isolated deployment copy is under ignored `temp/p_plan_audit/deployment_a3_20260927_120413`; it uses copied assets/scripts, a release binary and an empty `data/`. Boot reached runtime and package scan finished, but the three bundled official manifests each reference a missing nested `scripts/` directory. A ConPTY attempt did not establish a normal exit; do not count this as a clean TUI run. The original `E:\Code\tg-test\data` was never opened for writing.
- A3 remains open for a real terminal normal-exit/restore check, the deployment/package workflow, user visual and IME confirmation, and Linux/macOS verification. Windows `exec_command` cannot allocate its requested interactive PTY; ConPTY here was only an isolated protocol attempt, not a physical terminal. B work has not started, per the A-before-B gate. No commits or pushes were made.
