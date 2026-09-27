# B0 契约整理与待裁决项

基点：P_PLAN.md（2026-09-27），代码以本工作区为准。A 阶段已由用户验收，本文件只交付 B0 契约台账，不重做 A。

## 已确认决定

| 项目 | 决定 | 执行边界 |
|---|---|---|
| Q1 | 用 `save_game`、`best_score.enable`；每层 keys 超过两个元素报错；默认 FPS 跟随玩家设置 | B4 按字段表实现，不提供旧字段别名 |
| Q2 | 清单完全升级，不兼容旧格式；`PACKAGE_MANIFEST_VERSION=2` | 不因此清除、重置或改写存档/profile |
| Q4 | 保留 vendored Lua 5.4 | 同版本原生语义为 B2 对照，Lua 5.5 只做差异分析 |
| Q5 / B5 左右 Meta 显示名（2026-09-27） | Linux 显示 Meta；macOS 显示 Cmd；Windows 显示 Win | canonical key token 保持左右 Meta 区分；只变更平台显示标签 |
| B2 字符串元表（2026-09-27） | 脚本字符串值继续保持无元表；不通过 `("x"):sub(...)` 暴露原生 string 方法 | 原生 string 方法不进入脚本 `_ENV`；项目 `string.*` 仍只通过注册 API 调用，避免绕过参数、返回值与大小限制 |
| B2.4 string 调用契约（2026-09-27） | `find/match/gmatch/gsub` 当前使用命名参数并返回项目结果表；Lua 5.4 原生签名和返回形状不同 | 保留项目调用格式、参数传递和结果表；核对业务行为及安全边界，不迁移成原生位置参数/多返回 |
| B2.4 math/utf8 调用契约（2026-09-27） | 宿主使用数组/命名参数及结果表/迭代记录；原生 Lua 5.4 签名和返回形状不同 | 保留项目参数及结果协议，对照 Lua 5.4 核对计算行为与安全边界；继续提供项目扩展 |
| B1.3 字体回退（2026-09-27） | 显式配置字体优先；无法使用时回退部署包内置字体，再回退系统字体；所有来源均无可用字体才报错 | 相对文件路径锚定上线根；绝对用户字体路径继续支持；最终错误列出尝试路径 |
| B6 图片 API（2026-09-27） | 图像默认尺寸、crop、透明背景、异步关联与 event output 曾存在实现/文档差异 | Lua 缺省尺寸按文档比例计算并最小为 1×1；负 crop 或越界报错、省略尺寸取剩余区域；增加 RGB 字符串背景色，默认黑色并混色/入 cache key；`image.load` 返回 request id、终态事件回传同一 id；事件 output 保留富文本字符串，可直接绘制 |
| 生命周期与分层 | 保留现有 boot/shutdown 编排；runtime 业务及宿主状态/服务组合归 app | 不再为目录纯度重复迁移 |

## 需在对应实现前裁决

| ID / 对应工作包 | 当前证据与未定行为 | 候选方案（未获批准） | 是否阻塞 |
|---|---|---|---|
| Q2a / B4.2 | 清单 `mod_id` 新规则只许字母/数字/下划线；`PackageId` 同时读取旧 profile，仓库 fixtures 目前有 `test.xxx` | 清单严格收紧，profile 身份反序列化保持兼容；测试 fixture 可显式改名；不自动重命名用户 ID | 阻塞 ID 收紧与旧 profile 回归设计，不阻塞 B1–B3 |
| Q3 键位 / B4.4 | `normalize_action_keys` 允许顶层 `[]` 表示未绑定，拒绝内层 `[]`，每层最多两个元素并规范化 token | 复用现有校验；重复 token、相同主/备用绑定和跨动作冲突需明确报错、去重还是允许 | 阻塞相关 keys 契约，不阻塞 B1 |
| Q3 CLI / B4.7 | 当前 `src/main.rs` 不解析命令行，package/game config 也无 command 消费者 | 启动参数只解析为单一 command；扫描结束后按唯一匹配包走现有 game/screensaver 启动路径；未知、重复、宿主保留参数报错，不调用 shell | 冲突/优先级规则待确认；禁止在 B4 前悄悄定行为 |
| Q6 / B10 | `color.rgb`、`draw.text`、`file.read` 分属纯值、多可选项和异步事件；参数、返回及旧名期限并不统一 | 参考 [LUA_API_MIGRATION.md](../zh_cn/LUA_API_MIGRATION.md) 候选形状，先确认逐 API 的输入/结果/错误和兼容期 | 阻塞批量扩展 API 改名，不阻塞 B1–B9 的独立功能 |
| Q7 / B7、B9 | “与终端一致”未给参考终端、字体、字号、行距、缩放和 IME 名称 | 同一基准配置比较几何/颜色/样式；跨终端逐像素一致不作为承诺 | 阻塞像素与 IME 的实机验收基准，不阻塞 B1 |

## B0 交付索引

- [Lua 兼容性与当前注册面](../zh_cn/LUA_COMPATIBILITY.md)：按实际 `install` 清点标准兼容层、宿主扩展与未注册占位。
- [Lua 扩展 API 迁移表](../zh_cn/LUA_API_MIGRATION.md)：登记 API 族、Q6 样例和候选规则。
- 本台账记录已确认、待确认和各项阻塞边界。未确认候选不得当作实现授权。

## B6 `image.load` 契约（已确认）

| 字段 | 当前页面 | 代码现状 / 草案 |
|---|---|---|
| `path` | 必填，相对 `assets/` | 实现时通过 session assets root + sandbox path；禁止任意绝对路径 |
| `block_width/height` | 省略时按源图宽高公式取整，可能为 0 | 用户确认省略值按文档比例计算、每轴最小 1；显式 0/负值报字段错误。Rust `ImageConvertParams::default` 仍按 80×24 保持 |
| `crop_x/y/width/height` | 像素单位 | 用户确认负偏移或矩形越界报错；省略 width/height 时从偏移位置取剩余区域；显式 0 仍报错 |
| `scale` | 默认 1.0 | 规范化后参与缓存键；非正数及非有限值拒绝 |
| `cache` | 默认 true | false 不查/不写磁盘或内存缓存 |
| 转换模式 | 现有只描述 half-block | B6 增 `half_block` / `mix_block`，缺省 half-block；缓存键包含 mode/version |
| 透明背景 | 当前转换忽略 alpha | 用户确认新增可选 RGB 字符串背景色（`#rrggbb` / `rgb(r,g,b)`，可传 `color.hex/rgb` 返回值），默认黑色；先将 RGBA 合成为该背景上的 RGB，再进行两种模式转换，背景值纳入缓存键；拒绝主题命名色以保证确定性 |
| 返回与事件 ID | 页面只写事件返回 | 用户确认 `image.load` 立即返回 Session 内 request id，完成/失败事件回传同一 id |
| 事件 `output` | `EVENT.md` 示例写虚拟标识/路径，但 Rust 当前输出为富文本 | 用户确认保留可直接绘制的富文本字符串；修正文档示例和字段说明 |

上表作为 B6 实施契约。图片 worker 必须支持请求取消/退出屏障策略并限制到包 assets；不能让 Lua 指定缓存文件名。

## 检查来源

- Lua 安装：`crates/service/lua/src/api/libraries.rs::install` 与 `api/libraries/*.rs`。
- 图片差异：`dev_docs/zh_cn/api/image.md`、`crates/service/image/src/lib.rs`、`crates/service/lua/src/events/broker.rs`。
- keys：`crates/service/package/src/lib.rs::normalize_action_keys` 和 `crates/core/input/src/action_map.rs`。
- 命令入口：`src/main.rs`、app 启动/包扫描路径与现有 `launch_game` / screensaver 路径。
