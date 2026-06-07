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

