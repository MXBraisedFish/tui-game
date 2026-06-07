// 引用结构体和枚举
use crate::host_engine::core::{
    ExitState,
    FrameScheduler,
    RuntimeWorld,
};
use crate::host_engine::services::EngineServices;

// 运行函数
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
    // 启用终端模式
    services.terminal.enter(&mut services.log);

    // 启动全局键盘监听器
    services.input.start_key_listener();

    // 构建一个帧循环
    let mut scheduler = FrameScheduler::new(60);

    // ── 调试：固定帧数测试循环（~5秒，300帧 @ 60fps）──
    // TODO(debug): 状态机接入后替换为正式循环
    for _ in 0..300 {
        let frame = scheduler.begin_frame();

        world.clock.tick();

        services.canvas.begin_frame();
        services.canvas.clear();

        // 调试渲染
        render_debug_frame(services, world, frame);

        // TODO(error): 错误处理之后统一处理
        let _ = services.canvas.present(&mut services.terminal);

        scheduler.wait_for_next_frame();
    }
    // ── 调试循环结束 ──

    // TODO(runtime): 正式循环（状态机接入后启用）
    // 开始循环
    // while world.session.is_running() {
    //     let frame = scheduler.begin_frame();
    //
    //     world.clock.tick();
    //
    //     services.canvas.begin_frame();
    //     services.canvas.clear();
    //
    //     // 1. 输入处理 -> 路由到状态机
    //
    //     // 2. 更新(dt) -> 路由到状态机
    //
    //     // 3. 绘制 -> 路由到状态机
    //
    //     // 4. 呈现
    //     let _ = services.canvas.present(&mut services.terminal);
    //
    //     // 5. 帧等待
    //     scheduler.wait_for_next_frame();
    // }

    // 返回退出信息块
    ExitState::new()
}

// ── 调试渲染函数（状态机接入后删除）──
// TODO(debug): 临时渲染，用于验证 FrameScheduler + EngineClock + Canvas + Render 联调
fn render_debug_frame(
    services: &mut EngineServices,
    world: &RuntimeWorld,
    frame: u64,
) {
    services.render.draw_text(
        &mut services.canvas,
        0,
        0,
        "Runtime heartbeat test",
    );

    services.render.draw_text(
        &mut services.canvas,
        0,
        1,
        &format!("Frame: {}", frame),
    );

    services.render.draw_text(
        &mut services.canvas,
        0,
        2,
        &format!(
            "dt: {:.3} ms",
            world.clock.delta_time().as_secs_f64() * 1000.0,
        ),
    );

    services.render.draw_text(
        &mut services.canvas,
        0,
        3,
        &format!(
            "elapsed: {:.2} s",
            world.clock.elapsed().as_secs_f64(),
        ),
    );
}
