fn main() {
    // 1. CLI 指令处理
    if host_engine::boot::cli::handle_command()? {
        return;
    }

    // 2. 安装崩溃钩子
    host_engine::boot::panic_hook::install();

    // 3. 环境检查与初始化（此时还没有加载画面）
    host_engine::boot::environment::prepare()?;

    // 4. 打开加载画面（宿主硬编码的纯文本进度条）
    let loading = host_engine::loading_screen::start()?;
    loading.set_message( /* i18n: "正在加载资源..." */ );

    // 5. 资源载入（带进度回调）
    host_engine::boot::resources::load_all(|progress| {
        loading.update(progress);
    })?;

    // 6. 启动 Lua 虚拟机
    loading.set_message( /* i18n: "正在启动 Lua 虚拟机..." */ );
    let lua_vm = host_engine::boot::lua_rt::start()?;

    // 7. 关闭加载画面
    loading.close();

    // 8. 运行循环
    let exit_code = host_engine::runtime::run(lua_vm)?;

    // 9. 退出
    host_engine::shutdown::execute(exit_code)?;
}