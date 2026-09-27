# 当前架构

打开 `index.html` 即可浏览，无需启动网站服务。页面只保留当前工作区数据，包含依赖总览、生命周期、模块树和架构检查。

## 更新数据

在仓库根目录运行（需要 Python 3 和 Rust/Cargo）：

```powershell
python architecture/build_current.py
```

生成器通过 Cargo metadata 枚举工作区生产 `lib` / `bin` 目标，提取 Rust 模块、条目、导入和限定路径，更新 `data/current.js`。中间数据写入被 Git 忽略的 `temp/p_plan_audit/architecture/`。保留整个 `architecture/` 目录即可重复生成。

页面展示生成时的源码状态，底部标注生成时间；不会实时监听源码。其他 Agent 完成代码修改后，重新运行命令并刷新页面。

## 阅读方式与范围

- 依赖总览：卡片对应工作区 crate，连线从使用方指向被依赖方。点击卡片或搜索进入模块详情，支持拖动、缩放与收起详情栏。
- 模块树：按 Rust `mod` 声明展示模块层级，可展开、收起并查看源码位置。
- 生命周期：按当前生命周期入口整理，节点附源码位置；执行路径说明由生成脚本维护，变更控制流程时需同步核对。
- 架构检查：根据提取到的跨 crate 引用检查分层与服务环路。静态解析不等同于编译器完整名称解析，不能覆盖全部动态行为；无检查结果不代表通过完整代码审计。人工审计为空仅表示未附带记录。

主程序 crate 内包含阶段编排、应用层及终端 UI，因此总览合并标记为“主程序 / 应用”；内部结构在模块树中展开。没有独立 UI crate 时不显示空的 UI 分组。

## 浏览器核对

用 Chromium / Chrome / Edge 打开页面并启用本机 DevTools 端口后：

```powershell
node architecture/tools/dom_smoke.cjs <port>
# 可选：同时保存桌面和窄屏截图到指定目录
node architecture/tools/dom_smoke.cjs <port> temp/architecture_ui_review
```

检查单一当前数据源、四个视图、搜索定位、详情栏切换，以及桌面和窄屏布局。截图通过浏览器 DevTools 生成，不依赖页面外部资源。
