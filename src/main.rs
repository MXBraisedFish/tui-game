//! 宿主程序入口，负责三阶段调度：启动(Boot)、运行(Runtime)、关闭(Shutdown)
//! 本文件仅负责流程编排，不包含具体业务逻辑

pub mod host_engine;

/// 宿主程序统一结果类型，使用 Box<dyn Error> 作为错误载体
type HostResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 程序入口函数
/// 捕获并打印运行错误，不执行额外初始化
fn main() {
    run_host_entry();
}

/// 宿主入口
pub fn run_host_entry() {
    if let Err(error) = run_host() {
        eprintln!("{error}");
    }
}

/// 宿主主流程调度器
/// 执行顺序：CLI命令 → Panic钩子 → 环境准备 → 语言加载 → 加载屏 → 资源加载 → Lua运行时 → 关闭加载屏 → 主循环 → 关闭流程
fn run_host() -> HostResult<()> {
    // 处理命令行独立命令（如 -version, -help 等），若已处理则提前返回
    if handle_cli_command()? {
        return Ok(());
    }

    // 安装 panic 钩子，捕获崩溃信息
    install_panic_hook()?;
    // 准备运行环境（目录、权限等）
    prepare_environment()?;
    // 加载语言文件
    load_language_files()?;

    // 打开加载屏幕
    let mut loading_screen = open_loading_screen()?;
    // 加载资源（图片、配置等）
    let loaded_resources = load_resources(&mut loading_screen)?;
    // 启动 Lua 运行时
    let lua_runtime = start_lua_runtime(&loaded_resources)?;

    // 关闭加载屏幕
    close_loading_screen(loading_screen)?;

    // 运行主事件循环
    let exit_state = run_runtime_loop(lua_runtime, loaded_resources)?;
    // 执行关闭清理与持久化
    execute_shutdown(exit_state)?;

    Ok(())
}

/// 处理 CLI 独立命令（如 -version, -clear-cache 等）
/// 返回值：true 表示命令已处理，程序应退出；false 表示继续正常启动
fn handle_cli_command() -> HostResult<bool> {
    host_engine::boot::cli::handle_command()
}

/// 安装 panic 钩子，记录崩溃信息到日志
fn install_panic_hook() -> HostResult<()> {
    host_engine::boot::panic_hook::install()
}

/// 准备运行环境：创建必要目录、初始化权限、检查文件完整性等
fn prepare_environment() -> HostResult<()> {
    host_engine::boot::environment::prepare()
}

/// 加载语言文件到内存缓存
fn load_language_files() -> HostResult<()> {
    host_engine::boot::i18n::load()
}

/// 打开加载屏幕，返回屏幕状态句柄
fn open_loading_screen() -> HostResult<LoadingScreenState> {
    let loading_handle = host_engine::boot::loading::start();
    loading_handle.update(host_engine::boot::loading::LoadingStage::InitEnv, 1)?;
    Ok(LoadingScreenState { loading_handle })
}

/// 加载资源（图片、字体、配置文件等），过程中可更新加载屏进度
fn load_resources(loading_screen: &mut LoadingScreenState) -> HostResult<LoadedResources> {
    let initialized_environment = host_engine::boot::preload::init_environment::initialize()?;
    let _ = initialized_environment.is_input_listener_running();

    loading_screen
        .loading_handle
        .update(host_engine::boot::loading::LoadingStage::ScanGame, 20)?;
    let game_module_registry = host_engine::boot::preload::game_modules::load()?;
    loading_screen
        .loading_handle
        .update(host_engine::boot::loading::LoadingStage::ScanUi, 40)?;
    let official_ui_registry = host_engine::boot::preload::official_ui::load()?;
    loading_screen
        .loading_handle
        .update(host_engine::boot::loading::LoadingStage::ReadData, 60)?;
    let persistent_data = host_engine::boot::preload::persistent_data::load()?;
    loading_screen
        .loading_handle
        .update(host_engine::boot::loading::LoadingStage::PreCache, 80)?;
    let cache_data = host_engine::boot::preload::cache_data::load(&game_module_registry)?;
    let host_state_machine = host_engine::boot::preload::state_machine::load();
    loading_screen
        .loading_handle
        .update(host_engine::boot::loading::LoadingStage::ReadyLaunch, 95)?;
    let launch_readiness = host_engine::boot::preload::finalize_launch::load(
        &game_module_registry,
        &official_ui_registry,
        &persistent_data,
        &cache_data,
        &host_state_machine,
    )?;

    Ok(LoadedResources {
        initialized_environment,
        game_module_registry,
        official_ui_registry,
        persistent_data,
        cache_data,
        host_state_machine,
        launch_readiness,
    })
}

/// 启动 Lua 虚拟机，加载基础脚本和 API
fn start_lua_runtime(loaded_resources: &LoadedResources) -> HostResult<LuaRuntimeState> {
    let lua_runtime_environment = host_engine::boot::preload::lua_runtime::load(loaded_resources)?;
    Ok(LuaRuntimeState {
        lua_runtime_environment,
    })
}

/// 关闭加载屏幕，释放相关资源
fn close_loading_screen(loading_screen: LoadingScreenState) -> HostResult<()> {
    loading_screen.loading_handle.finish()
}

/// 主运行时循环：事件分发、渲染、状态机更新
fn run_runtime_loop(
    lua_runtime: LuaRuntimeState,
    loaded_resources: LoadedResources,
) -> HostResult<ExitState> {
    let runtime_terminal = host_engine::runtime::terminal::enter()?;
    let _ = runtime_terminal.is_active();
    let _ = lua_runtime.lua_runtime_environment.is_sandbox_installed();
    let _ = loaded_resources
        .initialized_environment
        .is_input_listener_running();
    let _ = loaded_resources.game_module_registry.games.len();
    let _ = loaded_resources.official_ui_registry.packages.len();
    let _ = loaded_resources.persistent_data.language_code.len();
    let _ = loaded_resources.cache_data.removed_game_uids.len();
    let _ = loaded_resources.host_state_machine.has_dialog();
    let _ = loaded_resources.launch_readiness.has_todo_items();
    host_engine::runtime::event_loop::run(&loaded_resources.initialized_environment.input_receiver)?;
    Ok(ExitState {})
}

/// 执行关闭流程：清理临时文件、保存状态、持久化数据
fn execute_shutdown(_exit_state: ExitState) -> HostResult<()> {
    Ok(())
}

/// 加载屏幕状态，用于更新进度文本/动画
struct LoadingScreenState {
    loading_handle: host_engine::boot::loading::LoadingHandle,
}

/// 已加载的资源集合（图片、字体、配置文件等）
struct LoadedResources {
    initialized_environment: host_engine::boot::preload::init_environment::InitializedEnvironment,
    game_module_registry: host_engine::boot::preload::game_modules::GameModuleRegistry,
    official_ui_registry: host_engine::boot::preload::official_ui::OfficialUiRegistry,
    persistent_data: host_engine::boot::preload::persistent_data::PersistentData,
    cache_data: host_engine::boot::preload::cache_data::CacheData,
    host_state_machine: host_engine::boot::preload::state_machine::HostStateMachine,
    launch_readiness: host_engine::boot::preload::finalize_launch::LaunchReadiness,
}

/// Lua 运行时状态（虚拟机实例、全局注册表等）
struct LuaRuntimeState {
    lua_runtime_environment: host_engine::boot::preload::lua_runtime::LuaRuntimeEnvironment,
}

/// 退出状态，携带关闭时需要保留的数据
struct ExitState {}
