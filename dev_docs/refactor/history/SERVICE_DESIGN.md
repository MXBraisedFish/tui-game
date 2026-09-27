# 服务迁移设计（草案，待确认）

数据来源：`temp/audit/now/units.json`（2026-09-25 用审计工具对当前代码重新生成）。

## 现状

服务按强连通分量自底向上排序：

- 无环叶子（可直接迁移）：code_highlight、clipboard、ffmpeg、host_object、input_method、popup、terminal_capabilities → terminal、unicode(UnicodeService)、ui
- 小环：rich_text ↔ text_layout（text_layout 用 RichTextService/RichTextParams，rich_text 用 TextMode）
- **巨型环：21 个服务、约 5.4 万行**：animation、async_runtime、audio、canvas、export、file、i18n、image、layout、log、lua、network、package、random、recording、render、screenshot、storage、video、widget、write_barrier
- 环外但依赖环：event、input、time、render_pipeline

## 巨型环的成因（按影响排序）

1. **async_runtime 聚合中心**：`EngineTask`/`EngineEvent` 两个枚举把所有服务的任务/事件类型收拢在一起，worker 再按枚举分派回各服务执行 → async_runtime 依赖 export/image/network/package/recording/screenshot/video/audio/lua 等；这些服务又调用 `AsyncRuntime::submit` 与 `EngineEvent` → 双向。
2. **widget 的 ID/池类型被下层使用**：canvas、layout、render 用 ScrollBoxId/SliceId/SurfaceId/UiObjectPool；animation 用 RuntimeObjectPool；random 用放在 widget 里的 Random* 配置类型；而 widget 又依赖 animation/canvas/layout/audio。
3. **log ↔ file / i18n**：LogService 用 FileService 写文件、用 I18nService 生成标签；i18n 又用 LogService 记日志。
4. **storage → audio**：storage 用 ResolvedAudioFile/AudioError 解析音频路径；audio 又依赖 async_runtime。

## 拆环方案

- (2) widget ID/池类型下沉：ScrollBoxId/SliceId/SurfaceId 等 ID（≥3 处使用、零依赖）提为核；Random* 配置类型移回 random 服务。
- (3) log 不再直接依赖 i18n/file：标签由应用层注入（LogLabels 作为数据传入），文件写入改用 atomic_fs 核或 std。
- (4) audio 的 ResolvedAudioFile/AudioError 下沉为核（使用者 storage、package、lua、ui/home）。
- (1) async_runtime：**需要确认方向**（见下）。

### async_runtime 方向（待定）

- **A（推荐）通用执行器 + 事件聚合上移应用层**：async_runtime 变成不认识任何服务的通用执行器（`submit(job)` 接收一个闭包/任务 trait 对象，提供取消与事件发送）；`EngineEvent` 聚合枚举移到应用层；各服务只定义自己的任务函数与事件类型，通过注入的发送端回传事件。改动面最大，但符合“核 ← 服务 ← 应用层”单向依赖。
- B 保留中心枚举，把 `EngineTask`/`EngineEvent` 与 worker 分派整体上移应用层：服务不再调用 AsyncRuntime，而是把任务描述返回给调用方，由应用层提交。改动面中等，但服务的异步调用点都要改为“返回任务描述”。
- C 暂不拆 async_runtime：先把可独立的服务迁出，巨型环保留在主 crate，最后再处理。改动最小，但 PLAN 的“每个服务可单独运行”无法对环内服务达成。

## 决定（2026-09-25）

用户答复：服务与核结构为理论结构，不满足实际需要时可按工程需要优化，自行决定继续。

执行顺序（增量、每步可验证）：
1. 先迁出环外叶子服务：terminal_capabilities+terminal、clipboard、ffmpeg、code_highlight、input_method、popup、host_object。
2. 做低成本拆环：widget ID 下沉、Random* 类型归位、log 与 i18n/file 解耦、audio 共享类型下沉、rich_text ↔ text_layout。
3. 拆环后重新生成依赖图，再按实际剩余结构决定 async_runtime 的处理（倾向 A 的渐进版：先让执行器不认识具体服务，事件聚合留到应用层拆分时一起上移）。

## widget 环的拆法（2026-09-26）

现状：widget::runtime_object 的 `RuntimeObjectPool` 是 time/random/animation 各自对象池的聚合容器，却定义在 widget 里；
这三个服务的方法都接收整个聚合池 → 服务反向依赖 widget。UI 侧同理：`UiObjectPool` 聚合各 UI 对象，
canvas.prepare 与 LayoutService 的 resolve_slice_*/resolve_scroll_box_* 直接读取它。

做法（逐步、每步可验证）：
1. random：widget/runtime_object/random 的对象类型并入 random 服务；RandomService 方法改收 `&mut RandomGeneratorObjects`。
2. time：计时器对象与回调请求并入 time 服务（TimeObjects），RuntimeObjectPool 上的 clear/take 辅助方法随之下沉。
3. animation：三个池 + remove_animations_targeting 合成 AnimationObjects；AnimationService 方法改收 `&mut AnimationObjects`。
4. layout：接收 UiObjectPool 的 resolve_* 方法移到 widget（slice/scroll_box）侧，LayoutService 只保留视口与锚点计算。
5. canvas：`prepare` 遍历 UiObjectPool 的部分移到 widget，canvas 只接收准备好的数据；ID/滚动条样式等数据类型下沉。
聚合容器（RuntimeObjectPool/UiObjectPool）留在 widget，widget 位于这些服务之上。

## async_runtime 的拆法（2026-09-26）

async_runtime.rs = 通用执行器（工作线程、取消、任务状态、写屏障、托管线程）+ 文件/图片任务实现 + EngineTask/EngineEvent 聚合。

1. 执行器抽成 `tg-service-async`，对事件类型泛型：`AsyncRuntime<E>`，任务实现 `AsyncJob<E>`
   （run / 写入目标 / 未开始即取消时的回调 / 是否自行处理取消）。执行器自身产生的完成、失败事件通过
   `E: From<TaskStatusEvent>` 回传。TaskId、ManagedThreadId、TaskState、TaskCancellation、WriteBarrier 一并下沉。
   主程序保留 `type AsyncRuntime = tg_service_async::AsyncRuntime<EngineEvent>`，EngineTask 实现 AsyncJob，
   现有 `submit(EngineTask::…)` 调用不变。
2. 逐个服务：服务定义自己的 Job 类型与事件类型，发送端为 `Sender<E>`（`E: From<服务事件>`），
   提交函数对 E 泛型（调用处可推断，无需改动）；删去对应的 EngineTask 变体。
3. EngineEvent 最终作为应用层的事件聚合（服务不再引用）。
