# B2.1 Lua 5.4 行为基线

日期：2026-09-27。基准使用仓库锁定的 `mlua 0.12.0` 与 vendored Lua 5.4，不启动系统 Lua CLI，不向游戏环境注入原生库。

## 方法

`crates/service/lua/src/session.rs::tests::lua_compatibility_baseline_runs_the_same_sample_on_lua_54` 将同一可信脚本运行在两个独立 VM：

- 基准：`Lua::new_with(StdLib::ALL_SAFE, ...)`，只存在于 Rust 测试。
- 宿主：生产 session 使用最小所需的 `StdLib::TABLE`，再通过项目 sandbox/API 暴露受保护代理；真实 Lua globals 不作为脚本环境。
- 分别从 VM 内部全局读取 `_VERSION`。测试要求字符串相同且包含 `5.4`；宿主脚本环境仍不可见 `_VERSION`。
- 输出只收集排序/确定的结构化字段；不比较 `pairs` 哈希遍历顺序、地址、时间或随机值。

验证命令：

```powershell
cargo test -p tg-service-lua --locked lua_compatibility_baseline_runs_the_same_sample_on_lua_54
```

2026-09-27 Windows 结果：退出 0，1 项通过。`cargo fmt --all -- --check` 同样退出 0。

## B2.1 初始差异：`ipairs` / `pairs`

样例使用 `{7, 11}`，比较迭代器状态与调用结果：

| 字段 | Lua 5.4 原生 | B2.1 开始时的宿主 | B2.2 修复后 |
|---|---|---|
| `ipairs` 第二返回值是原表 | 是 | 否（nil） | 是 |
| `ipairs` 首次迭代返回 | 两个返回值 `1, 7` | 单个记录表 `{index=1, value=7}` | 两个返回值 `1, 7` |
| `ipairs` 第二迭代值是否 nil | 否 | 是 | 否 |
| `pairs` 第二返回值是原表 | 是 | 否（nil） | 是 |
| `pairs` 第二迭代值是否 nil | 否 | 是 | 否 |

`pairs` 顺序不进入基准断言。B2.2 将 iterator/state/control、多返回与普通参数调用改为 Lua 5.4 形状；readonly API proxy 仍只暴露空状态表和受控 iterator，不返回 backing table。

## 额外发现

一次探索性样例将旧 `ipairs` 记录表直接传给旧宿主 `type(record)`，入口参数层把记录的 `value`/`index` 字段当成命名参数，报 `base.type: unknown parameter 'index'`。B2.2 已移除标准 base 函数的 `args::one/named` 解包，回归现检查 `type({index=1,value=2}) == "table"`。

同版本 `base_library_matches_native_lua54_on_plain_table_operations` 还比较 `next`、`select` 的 nil 多返回、元表保护、自定义 `__pairs`、`rawlen`、`rawequal`、`tonumber` 十六进制浮点和基础 `tostring/type`。其中十六进制浮点差异已修复。循环 table 可交给 `ipairs` 与 `type`；序列化仍单独拒绝循环。

用户于 2026-09-27 确认字符串元表保持隔离。`string_primitive_metatable_stays_isolated_from_native_string_methods` 明确记录基准 Lua 5.4 有原生字符串方法，生产 VM 的 `getmetatable("text")` 返回 nil；不因此打开 native string library。

## B2.3 只读代理与环境隔离

普通表的 `rawget/rawset` 与独立 Lua 5.4 VM 使用相同原始槽位语义，包括忽略 `__index`、nil 删除和 `rawset` 返回原表。宿主库表由空代理加受保护元表实现，因此：

- `rawset(proxy, key, value)` 在写入前识别项目代理并报只读错误；不会因 raw 操作建立遮蔽字段。
- `rawget(proxy, key)` 只读代理本身，返回 raw 槽位结果，不追踪 `__index`，也不访问 backing table。
- `pairs/next` 仍通过受控迭代器枚举公开库字段；迭代状态是空表，不能作为 backing 引用使用。
- `getmetatable(proxy)` 暴露锁定标记而非内部元表；`setmetatable` 不能替换代理元表。普通 Lua 表允许原生 `setmetatable(table, nil)`。

`session::tests::loader_matches_module_semantics_and_rejects_unsafe_sources` 还验证入口 `_ENV` 和 `loader.require` 执行的模块都看不到 `_G`、原生 `load/package` 与 `debug.getregistry`。生产 VM 只打开当前工作组需要的原生 `table` 库；它存在于 Rust 管理的真实 globals 中，不会通过脚本 `_ENV` 暴露。其他标准库继续关闭，真实 globals/registry 访问仅保留在 Rust 宿主与测试内。

Windows 验证：`cargo test -p tg-service-lua --locked`（99 通过）；`cargo clippy -p tg-service-lua --all-targets --locked -- -D warnings`；`cargo fmt --all -- --check`，均退出 0。

未开放 `io`、`os`、原生 `debug` 或 `package`。table 的 C 标准函数由宿主包装保留单次处理上限：sort 最多 4096 项，索引范围和 pack/unpack 最多 16384 项，concat 输出最多 1 MiB。string/pattern、math/utf8 与错误/GC/关闭的逐组结果见下文；预算和宿主资源限制在 B2.5 保持并验证。

## B2.4 string/pattern 原生基线审计

`session::tests::lua54_string_baseline_records_byte_indices_and_native_returns` 固定 Lua 5.4 标准行为：`string.sub` 与 `find` 索引按字节；`find` 返回起止位置和捕获作为多个值；`match/gmatch` 返回多捕获；`gsub` 返回替换文本与替换计数；`reverse` 反转字节；标准大小写只转换当前 C locale 可处理的字节。另一个格式探针覆盖 `%g/%e` 舍入与指数、正负无穷/NaN、`%q` 的控制字符与数字歧义转义，以及 `%c` 的单字节行为。Lua 5.4 此处不支持 `%a/%A`。

用户于 2026-09-27 确认保留项目调用格式、参数传递和结果表形状；字符串 primitive metatable 继续保持隔离。本组检查业务语义与安全边界，不迁移成原生位置参数/多返回。项目使用 Unicode 字符位置与 `lower/upper/reverse` 语义；格式宽度及 `%s` 精度也按 Unicode 字符计算，`%c` 接受 Unicode 码点。这些是记录在 API 文档中的项目语义，与原生 Lua 的字节行为有意不同。

修复及约束：

- 正则从非零 `init` 搜索时使用整段文本的锚点语义，`^` 不会错误地锚定到切片后的搜索位置。
- `find/match` 即使 `init` 越界也先检查必填参数和 `plain` 类型；不匹配时才跳过 pattern 编译。
- `%e/%E/%g/%G` 对照原生格式；`%q` 为控制字符使用 Lua 可往返的转义（包括后接数字时补足三位），`%f` 的无穷与 NaN 使用 Lua 5.4 基线形式。数值精度限制为 32，字符串精度不被误截断到 32。
- `string.format` 对最终输出再检查 1 MiB，覆盖结尾的普通文本；格式宽度最多 1 MiB。
- 空串 `string.rep` 在极大重复次数下直接返回空串，非空输出按预估大小一次分配，不建立重复引用数组。
- Lua pattern 限制为 8 KiB、512 个操作、32 个捕获组和 1,000,000 个匹配步骤；regex 限制为 8 KiB 与 32 个捕获组。单次输出/捕获总字节最多 1 MiB，遍历或替换最多 10,000 项。
- 批量 Lua pattern 搜索复用一份紧凑 UTF-8 索引；惰性 `gmatch` 持有原 Lua 字符串、首次迭代时建立索引并复用，匹配步数仍累计受限。结果达到迭代上限时不预先计算剩余匹配。

生产回归 `session::tests::string_api_matches_documented_tables_unicode_and_limits` 覆盖命名参数与表返回契约、Unicode 索引/迭代、锚点、参数错误、捕获/操作上限、`%g/%e/%q/%c` 格式、输出限额和空串极大重复次数。此处通过探针确认的实现差异不是一律消除；调用形状、安全边界以及文档承诺的项目行为优先。

## B2.4 math/utf8 原生基线审计

`session::tests::lua54_math_utf8_baseline_records_native_returns_and_byte_offsets` 在同版本基准 VM 固定了下列行为：`math.modf/frexp` 分别返回两个值；`math.min/max` 接受变参；`math.ult` 按无符号整数比较；`math.tointeger` 对非整数返回 nil。原生 `utf8.codes` 返回迭代器/状态/控制三元组，迭代产生 UTF-8 字节位置和码点；`utf8.codepoint` 返回多个码点，`utf8.char` 接受位置参数；`utf8.len` 可限定字节范围，`utf8.offset` 返回字节位置。

宿主当前保留一套带输入校验的项目 API：`math.min/max` 接受稠密数组，`math.frexp/modf` 返回有命名字段的结果表，`fmod/pow/log/ldexp/atan2/ult` 使用命名参数表；扩展项包括 `round/round_to/normalize_angle/approx_equal/percent/factorial/combination` 和大写常量。`utf8.len` 只接收一个字符串；`codepoints` 是返回单项记录表的惰性迭代器；`codepoint_to_char`、`char_to_codepoint`、`char_to_ascii`、`char_position`、`next` 等是项目扩展，部分使用命名参数和结果表。尽管完整字符串的 `utf8.len` 码点计数与原生一致，范围参数、码点/字节位置、参数形状和返回数量不一致。

用户于 2026-09-27 确认 math/utf8 也保留项目参数与结果协议：数组参数、命名参数表、结果表和惰性记录迭代器不改成 Lua 5.4 的位置参数/多返回。Lua 5.4 基线用于核对运算、边界和安全行为，不替换项目调用协议；现有扩展继续保留。实现按 math 与 UTF-8 分块验证；如后续需要改变已确认协议之外的具体行为，先在实现前提问。

math/utf8 修复记录：`math.min/max` 按声明的 `n` 长度校验稠密数组，拒绝类型错误、长度不符和空洞；`utf8.next` 从给定字节位置开始检查字符边界，不每次重扫前缀；`utf8.char_position` 不再为整段文本建立字节位置向量；`codepoints`/`next` 和 string `gmatch` 复用 Lua 字符串句柄，避免额外复制完整文本。生产回归覆盖 `table.pack`、错误 `n` 和 UTF-8 多字节中间位置。

## B2.5 错误、预算、GC 与关闭

项目 `debug.pcall/xpcall` 继续使用命名参数与结果表。普通 Lua/API 错误按项目协议作为结果返回；内存错误、执行时间/指令预算耗尽和宿主标记的致命 API 资源错误继续向外传播，不能被脚本保护调用隐藏。完整 traceback 字符串不作为跨实现一致性契约。

`run_with_budget` 为每个受控执行建立独立线程 hook，统计 Lua 指令并同时检查包含 Rust API 执行时间的硬时限；执行失败后 reset Lua 5.4 线程，使待关闭 `<close>` 变量展开。回归 `instruction_budget_cannot_be_hidden_by_pcall`、`instruction_limit_still_closes_pending_variables` 与 `callback_error_closes_to_be_closed_variables` 验证预算错误不可吞、超限后关闭动作继续发生、普通错误关闭值非 nil。memory-limit 回归在受保护调用中触发 VM 内存限额，确认错误上报为 `MemoryLimit` 且独立会话可继续。

Lua 垃圾回收只通过 Rust 宿主控制；脚本没有原生 `collectgarbage`。`lua_gc_preserves_weak_tables_and_runs_finalizers` 用显式测试 GC 核对弱值表与 `__gc` 行为，不把自然回收时点固定成输出。`stop_releases_registered_callbacks_before_collecting` 验证会话停止先释放事件 callback 的 RegistryKey，再做一次完整 GC；捕获对象得以回收。宿主对象池在停止/故障时直接取走，Game、Screensaver 和会话级回归验证对象释放。Lua 状态销毁本身也由 mlua 执行最终关闭。

B2.5 还依赖既有 `persistent_callback_is_removed_after_its_terminal_event`、broker 的 stale-generation 清理、六类测试包入口生命周期与 loader 安全/缓存/输入大小测试。2026-09-27 Windows `cargo test -p tg-service-lua --locked` 为 106 通过、0 失败、0 忽略；严格 Clippy、fmt 与目标文件 diff 检查均通过。
