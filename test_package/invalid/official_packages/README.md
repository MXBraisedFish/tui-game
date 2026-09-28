# Official packages without Lua entry scripts

These are explicit negative fixtures. They contain schema 2 manifests but intentionally have no `scripts/` directory, so package scanning must reject them. They are outside the production `scripts/game/` tree and must not be given empty or placeholder Lua entry points.
