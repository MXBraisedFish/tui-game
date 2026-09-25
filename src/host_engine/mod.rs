pub mod boot;

pub mod runtime;

pub mod shutdown;

pub mod core;

pub mod services;

pub mod ui;

use crate::host_engine::core::{
  CrashPhase, HostFaultDomain, HostFaultPhase, catch_host_fault, finalize_host_fault,
  install_panic_hook, set_crash_phase,
};
use crate::host_engine::services::{HostLogMessage, LogSource, TerminalService};

/// 启动并运行引擎主循环，依次执行引导、运行时、关闭三个阶段
pub fn run() {
  install_panic_hook(TerminalService::force_restore);

  set_crash_phase(CrashPhase::Init);
  let boot_output = boot::prepare();

  let mut services = boot_output.services;
  let mut world = boot_output.world;

  if let Some(fault) = boot_output.fault {
    let run_id = services.log.run_id().to_string();
    let _ = finalize_host_fault(&run_id, &fault);
    services.log.error_message(
      LogSource::Crash,
      HostLogMessage::new(
        "log_info.crash.summary",
        "A host fault occurred during {phase}; details were written to tui_crash.log.",
      )
      .param("phase", format!("{:?}", fault.phase)),
    );
    set_crash_phase(CrashPhase::Runtime);
    log_lifecycle(&mut services, "runtime");
    let exit_state = runtime::run_exception(&mut services, &mut world);
    set_crash_phase(CrashPhase::Shutdown);
    log_lifecycle(&mut services, "shutdown");
    shutdown::close(&mut services, world, exit_state);
    return;
  }

  set_crash_phase(CrashPhase::Runtime);
  log_lifecycle(&mut services, "runtime");
  let exit_state = match catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Other, || {
    runtime::run(&mut services, &mut world)
  }) {
    Ok(exit_state) => exit_state,
    Err(fault) => {
      let run_id = services.log.run_id().to_string();
      let _ = finalize_host_fault(&run_id, &fault);
      services.log.error_message(
        LogSource::Crash,
        HostLogMessage::new(
          "log_info.crash.summary",
          "A host fault occurred during {phase}; details were written to tui_crash.log.",
        )
        .param("phase", format!("{:?}", fault.phase)),
      );
      runtime::run_exception(&mut services, &mut world)
    }
  };

  set_crash_phase(CrashPhase::Shutdown);
  log_lifecycle(&mut services, "shutdown");
  shutdown::close(&mut services, world, exit_state);
}

fn log_lifecycle(services: &mut services::EngineServices, phase: &'static str) {
  services.log.info_message(
    LogSource::Engine,
    HostLogMessage::new(
      "log_info.lifecycle.enter",
      "Lifecycle entered the {phase} phase.",
    )
    .param("phase", phase),
  );
}
