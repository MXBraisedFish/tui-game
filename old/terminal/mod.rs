// terminal 模块入口，声明三个子模块，自身无业务逻辑

// 将游戏 Canvas 渲染到终端，支持增量渲染
pub mod renderer;

// 终端会话生命周期管理（raw mode、alternate screen）
pub mod session;

// 终端尺寸检测与尺寸不足警告
pub mod size_watcher;
