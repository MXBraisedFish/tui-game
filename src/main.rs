//! 测试入口调度器
//! 使用 `-old` 启动旧版入口，使用 `-new` 启动新版入口

#[path = "new/main.rs"]
mod new_entry;
pub use new_entry::host_engine;
#[path = "old/main.rs"]
mod old_entry;

const OLD_ENTRY_ARGUMENT: &str = "-old";
const NEW_ENTRY_ARGUMENT: &str = "-new";

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some(OLD_ENTRY_ARGUMENT) => old_entry::run_old_entry(),
        Some(NEW_ENTRY_ARGUMENT) => new_entry::run_new_entry(),
        _ => {
            eprintln!("Usage: tui-game -old | -new");
        }
    }
}
