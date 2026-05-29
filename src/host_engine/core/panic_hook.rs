use std::panic;

use crate::host_engine::services::TerminalService;

pub fn install_panic_hook() {
  let previous_hook = panic::take_hook();

  panic::set_hook(Box::new(move |panic_info| {
    TerminalService::force_restore();

    previous_hook(panic_info);
  }))
}