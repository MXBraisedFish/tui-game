# B10.3 Lua event production audit

Date: 2026-09-28

This audit distinguishes event data structures and test adapters from events that a packaged Lua session can actually receive. The installed Lua environment is defined by `crates/service/lua/src/api/libraries.rs::install`; having a service or enum variant does not create a Lua producer.

## Production delivery

| Event family | Production source and route | Lua entry point |
|---|---|---|
| `action`, `mouse`, `resize`, `focus` | Host input/runtime observations enter `LuaEventBroker::push_system`; target sessions are selected by broker policy. | `HandleEvent(event)` |
| `overlay_started`, `overlay_stopped` | App runtime reports game overlay lifecycle to the event broker. | `HandleEvent(event)` |
| `file` | Installed `file.*` APIs enqueue package-assets tasks and register their task ownership before task submission. | `HandleEvent(event)`; calls return nil immediately, terminal events include a Session request id that the caller did not receive synchronously. |
| `i18n` | Installed `i18n.create/reload` enqueue a package language-file task; broker translates the terminal file task result and applies successful language data before dispatch. | `HandleEvent(event)` |
| `image` | Installed `image.load` submits `ImageConvert`, registering its session request id and ownership. | `HandleEvent(event)`; `request_id` matches the immediate return. |

These are the event paths with both a production producer and session routing. Tests in `events/mod.rs`, `events/broker.rs`, and `session.rs` cover payload construction, ownership, stale generations, terminal delivery, and image request flow.

## Schema or infrastructure without a production Lua producer

| Event family / feature | Evidence | Current status |
|---|---|---|
| Independent event callbacks | `LuaSession::register_event_callback` and environment discovery helper are `#[cfg(test)]`; no installed Lua library exposes registration. Production routed events use `LuaEventRoute::HandleEvent`. | Do not document independent callback registration as callable. The route and lifetime tests are infrastructure tests. |
| `timer` | `events/translate.rs` is itself `#[cfg(test)]`; `LuaTaskOperation::Sleep` has no production registration call, and no timer/sleep library is installed. | Not produced for Lua packages. |
| `animation` | `LuaRoutableEvent` has no animation variant; the translator is test-only and no app event bridge invokes it. | Not produced for Lua packages. |
| `hit_area`, `hyperlink`, `markdown`, `text_input`, `scroll_box` | Translator module is test-only. Widget components create internal Rust UI events, but the Lua library registry has no widget package and the app does not translate these component events into Lua deliveries. | Do not imply widget event APIs are currently usable by Lua. |
| `audio` | Broker has audio routing structures, but `audio` is not installed and there is no production Lua owner/registration path. | Infrastructure only for this API inventory; no Lua package producer. |
| `network` | Network event types and broker translation exist, but there is no installed `http`/`network` API or production Lua task registration. | Infrastructure only; HTTP/network API remains unimplemented. |

The event declarations and exhaustive payload tests are useful contract groundwork, but do not justify enabling an API by removing `cfg(test)`. New production sources need a registered Lua API, per-session ownership, cancellation/terminal semantics, and end-to-end tests before being documented as available.

## Documentation corrections

- `LUA_COMPATIBILITY.md` is the authority for installed library availability; `image.load` is installed, while `audio`, `animation`, `http`, `timer`, and `widget.*` are not.
- `EVENT.md` and `CALLBACK.md` now state that independent callbacks and schema-only event kinds are not current production capabilities.
- The legacy `RICH_TEXT.md` examples used unsupported `{tc:...}` / `{ts:...}` syntax; current examples use the `f%` prefix and supported angle-bracket tags. The current reference now distinguishes Auto/Rich parsing, namespaced placeholders and non-stacking close tags. Game/screensaver config examples now match.
- The file API described binary reads as `byte=false`; the implementation uses `byte=true`. Read/write notes now state the implemented mode.
- The serialization registry and migration table now use the implemented `binary_pack`, `binary_unpack`, and `binary_packsize` names. The serialization guide records shared JSON conversion limits, empty-table behavior, and current nested-null data loss from `serialization/value.rs`; the final null representation is paused under OQ-B10-02.
- Q6 remains open for the request/result style of future or migrated APIs. This audit did not change any Lua signature or event payload.

## Verification basis

- `libraries.rs::install` checked against every module assignment.
- `EngineEvent::lua_routable`, `drain_engine_events`, runtime system event routing, and the Lua command dispatcher checked for production sources.
- `events/translate.rs` module gate, session callback registration gate, API registry and broker ownership paths inspected directly.
- `cargo test --workspace --locked` passed earlier in this turn; 2 existing interactive terminal tests were ignored by design. This is code-path/documentation verification, not a claim that schema-only event types were exercised in production.
- During B10.4 follow-up, `cargo test -p tg-service-lua serialization_ --locked` passed 4/4 serializer integration tests, including new top-level null/empty-table/numeric-boundary cases; nested JSON `null` data preservation remains untested under OQ-B10-02.
