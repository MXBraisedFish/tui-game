# Security Details

Lua scripts run with a restricted set of project APIs. Their access depends on the session type and the package's Debug setting.

## Game sessions

- File APIs operate inside the current package's `assets/` directory. They cannot address arbitrary host paths or another package's files.
- `event.skip_action` and `event.clear_action` are available only to game sessions.
- Game control APIs are not exposed to screensaver sessions.

## Screensaver sessions

- Screensavers can read their own package assets. File writes, directory creation, deletion, and game action controls are unavailable.
- User input events are filtered by the host before delivery.

## Debug output

The package Debug setting enables `debug.print`, `debug.info`, `debug.warn`, `debug.error`, and slow-callback warnings. Assertion and protected-call helpers remain available when Debug is off.

Lua scripts do not receive the native `io`, `os`, or `package` libraries. The host enforces per-session memory and execution limits and keeps terminal control in the host application.
