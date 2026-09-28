# keyboard 库

截至 2026-09-28，`keyboard` 尚未由 `crates/service/lua/src/api/libraries.rs::install` 注册，Lua 包脚本当前不可调用。以下说明仅记录未来方向，不定义可调用函数。

当前注册面见 [LUA_COMPATIBILITY.md](../LUA_COMPATIBILITY.md)；生产事件来源见 [B10_EVENT_AUDIT.md](../../refactor/B10_EVENT_AUDIT.md)。

## 基本库说明

`keybord` 用于控制键盘原生事件。
