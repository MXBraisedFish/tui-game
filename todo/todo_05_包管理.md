# 包管理层

## 当前阶段目标

在现有包扫描骨架的基础上，补全包的生命周期管理：启用/禁用、状态持久化、热重载、显示排序，以及包的校验和兼容性检查。

---

## 服务项目清单

### 5-1 PackageService 包管理核心

- **职责**：管理系统中的所有包（游戏、屏保、Boss），包括扫描发现、元数据管理、启用/禁用状态、热重载检测、显示顺序管理。
- **当前状态**：骨架存在。已实现目录扫描、`package.json` 解析、`PackageInfo` 结构体和按类型分类存储。已支持 `schema_version`、`api` 范围解析、`display` 信息解析。三个列表（games/screensavers/bosses）和错误列表已就位。
- **待完善**：

  **启用/禁用管理**：
  - `set_enabled(uid, enabled)`：修改包的启用状态。
  - 状态持久化到 ProfileStore（通过请求方式，不直接修改 StorageService 内部）。
  - 禁用的包在游戏列表中显示为灰色或隐藏（取决于用户设置）。

  **状态同步（reconcile）**：
  - 启动时从 ProfileStore 读取已保存的包状态，与当前扫描结果对齐。
  - 处理三种情况：新发现的包（设置默认状态）、已删除的包（从 profile 中清除记录）、状态变更的包（更新内存状态）。

  **显示排序**：
  - 从 ProfileStore 读取用户自定义的显示顺序。
  - 支持多种排序模式：按标题字母、按最近游玩时间、按安装时间、用户手动拖拽顺序。
  - 排序结果影响 UiService 中列表页的展示顺序。

  **热重载**：
  - `hot_reload()` 方法：重新扫描包目录，检测新增/移除/修改的包。
  - 文件变更检测：对比文件修改时间判断包是否被更新。
  - 重载后自动 reconcile 状态。

  **包校验**：
  - 校验 `package.json` 的必填字段（`type`、`package`、`namespace`、`version_code`、`api`、`entry`、`display.title`）。
  - 校验 API 版本兼容性：当前引擎 API 版本是否在包的 `api.min` 和 `api.max` 范围内。
  - 校验入口文件是否存在（`entry` 指向的 Lua 文件）。
  - 校验失败时记录到错误列表，不影响其他包的加载。

  **包信息扩展**：
  - 读取包的运行时需求（`runtime.min_width`、`runtime.min_height`、`runtime.target_fps`），供运行时使用。
  - 读取包的游戏专属信息（`game.name`、`game.save`、`game.score`、`game.actions`），供 GameService 使用。
  - 读取包的屏保/Boss 专属信息。

### 5-2 PackageRegistry 包注册表（可选）

- **职责**：如果包数量增长到需要频繁查询的程度，可在 PackageService 内部建立一个索引结构（如 HashMap<uid, &PackageInfo>），加速按 UID 查找。
- **当前状态**：不存在。当前通过遍历三个 Vec 进行查找。
- **待完善**：
  - 建立 UID 到包的索引映射。
  - 启用/禁用/卸载时更新索引。

---

## 旧架构参考

### Package 系统（`old_src/host_engine/package/`）
旧架构的包管理模块比当前骨架更为精细，包含以下子模块：
- **kind**：包类型定义（Game / Screensaver / Boss）。
- **manifest**：`package.json` 的结构化定义，包含所有字段的完整类型。
- **package_id**：包的全局唯一标识符（UID）的解析和比较逻辑。
- **registry**：包注册表，存储所有已发现的包及其状态。
- **validator**：包校验器，检查必填字段、API 兼容性、入口文件存在性等。
- **scanner**：目录扫描器，递归扫描指定目录，发现包含 `package.json` 的目录作为包。
- **manager**：包管理器，整合上述所有模块，对外提供统一接口。

旧架构中 PackageManager 持有对 ProfileStore 的直接引用，通过 `&mut` 修改 profile 中的包状态数据。新架构应改为：
- PackageService 需要读取包状态时，向 StorageService 发起请求。
- PackageService 需要保存包状态时，向 StorageService 发起保存请求。
- 任何服务不得直接修改其他服务的内部数据。

旧架构的 validator 独立为模块，新架构可以将校验逻辑内置于 PackageService 的扫描流程中，减少模块碎片化。

---

## 完成后可验证的可用项

1. 启动引擎后，`scripts/` 和 `data/mod/` 目录下的所有包被正确扫描并分类。
2. `package.json` 解析失败时，错误信息包含文件名和具体原因，不阻塞其他包的加载。
3. 在设置页面禁用一个游戏包后，游戏列表中该项显示为灰色或隐藏。
4. 重启引擎后，包的启用/禁用状态保持不变。
5. 新增一个包到目录后，调用热重载能检测到新包。
6. API 版本不兼容的包被标记为不兼容，在 UI 中显示警告。
7. 按不同排序方式切换，列表顺序正确更新。
