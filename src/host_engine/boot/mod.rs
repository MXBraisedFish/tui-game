use std::thread;
use std::time::Duration;

use crate::host_engine::core::{
  BootOutput, HostFault, HostFaultDomain, HostFaultPhase, RuntimeWorld, catch_host_fault,
  with_fault_domain,
};
use crate::host_engine::services::{
  EngineEvent, EngineServices, HOST_VERSION, HostLogMessage, LogSource, PackageEvent,
};
use crate::host_engine::ui::{BootLoadingUi, BootProgress, BootStage};

/// Prepares the engine and keeps ownership of partially initialized services
/// when a supervised, post-terminal Boot fault occurs. This lets the caller
/// show the normal exception page and perform an orderly shutdown.
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();
  let world = RuntimeWorld::new();

  let result = catch_host_fault(HostFaultPhase::Boot, HostFaultDomain::Other, || {
    let terminal_profile = with_fault_domain(HostFaultDomain::Storage, || {
      services
        .storage
        .read_terminal_profile_or_default(&mut services.log)
    });
    services.terminal.apply_capability_profile(
      terminal_profile.unicode,
      terminal_profile.color.as_deref(),
      terminal_profile.mouse,
    );
    with_fault_domain(HostFaultDomain::Terminal, || services.terminal.enter()).map_err(
      |error| {
        HostFault::error(
          HostFaultPhase::Boot,
          HostFaultDomain::Terminal,
          format!("failed to enter terminal mode: {error}"),
        )
      },
    )?;
    prepare_supervised(&mut services)
  });
  let fault = match result {
    Ok(Ok(())) => None,
    Ok(Err(fault)) | Err(fault) => Some(fault),
  };

  BootOutput {
    services,
    world,
    fault,
  }
}

fn prepare_supervised(services: &mut EngineServices) -> Result<(), HostFault> {
  present_progress(services, BootProgress::at(BootStage::Storage))?;
  present_progress(services, BootProgress::at(BootStage::Terminal))?;
  present_progress(services, BootProgress::at(BootStage::Language))?;

  with_fault_domain(HostFaultDomain::I18n, || {
    services
      .i18n
      .refresh_language_registry(&services.storage, &mut services.log);
    let default_code = services.storage.default_language_code().to_string();
    let preferred = services.storage.read_language_code(&mut services.log);
    let selected_language = match preferred {
      None => default_code,
      Some(ref code) => {
        let in_registry = services
          .i18n
          .language_registry()
          .iter()
          .any(|entry| entry.code == *code);
        let available =
          services
            .i18n
            .is_language_package_available(&services.storage, &mut services.log, code);
        if in_registry && available {
          code.clone()
        } else {
          services.log.warn_message(
            LogSource::Boot,
            HostLogMessage::new(
              "log_info.boot.language_fallback",
              "Saved language '{code}' is invalid; language selection will be shown again.",
            )
            .param("code", code)
            .param("in_reg", in_registry.to_string())
            .param("avail", available.to_string()),
          );
          let _ = services.storage.write_language_code("");
          default_code
        }
      }
    };

    services.i18n.load_language_package_info(
      &services.storage,
      &mut services.log,
      &selected_language,
    );
    services
      .i18n
      .load_runtime_language(&services.storage, &mut services.log, &selected_language);
    let _ = services.i18n.apply_log_translations(&mut services.log);
  });

  let run_id = services.log.run_id().to_string();
  services.log.info_message(
    LogSource::Boot,
    HostLogMessage::new(
      "log_info.run.started",
      "Run {run_id} started with TUI GAME {version}.",
    )
    .param("run_id", run_id)
    .param("version", HOST_VERSION),
  );
  services.log.info_message(
    LogSource::Boot,
    HostLogMessage::new(
      "log_info.lifecycle.enter",
      "Lifecycle entered the {phase} phase.",
    )
    .param("phase", "boot"),
  );
  log_stage_complete(services, BootStage::Storage);
  log_stage_complete(services, BootStage::Terminal);
  log_stage_complete(services, BootStage::Language);

  present_progress(services, BootProgress::at(BootStage::Services))?;
  log_stage_complete(services, BootStage::Services);

  with_fault_domain(HostFaultDomain::Package, || {
    let root_dir = services.storage.root_dir().to_path_buf();
    let package_language = services.i18n.current_language().to_string();
    let missing_template = services
      .i18n
      .get_runtime_text("language_warning", "language_warning.missing");
    services
      .package
      .configure_scan(&root_dir, &package_language, &missing_template);
    if !services.package.request_rescan(&services.async_runtime) {
      return Err(HostFault::error(
        HostFaultPhase::Boot,
        HostFaultDomain::Package,
        "initial package scan could not be submitted",
      ));
    }
    wait_for_initial_package_scan(services)
  })?;
  log_stage_complete(services, BootStage::Packages);

  present_progress(services, BootProgress::at(BootStage::Listeners))?;
  services
    .input
    .start_key_listener(&mut services.async_runtime);
  services
    .input
    .start_system_listener(&mut services.async_runtime);
  services.package.start_watcher(&mut services.async_runtime);
  log_stage_complete(services, BootStage::Listeners);

  present_progress(services, BootProgress::at(BootStage::Runtime))?;
  log_stage_complete(services, BootStage::Runtime);
  present_progress(services, BootProgress::ready())?;
  log_stage_complete(services, BootStage::Ready);
  thread::sleep(Duration::from_millis(80));
  Ok(())
}

fn wait_for_initial_package_scan(services: &mut EngineServices) -> Result<(), HostFault> {
  let mut scanned = 0usize;
  let mut total = 0usize;
  let mut finished = false;
  while !finished {
    present_progress(services, BootProgress::package(scanned, total))?;
    for event in services.async_runtime.poll_events() {
      match event {
        EngineEvent::Package(event) => {
          let event = services
            .package
            .handle_async_event(event, &mut services.log);
          match event {
            PackageEvent::ScanStarted { total: value } => total = value,
            PackageEvent::ScanProgress {
              scanned: value,
              total: maximum,
            } => {
              scanned = value;
              total = maximum;
            }
            PackageEvent::ScanFinished { .. } => finished = true,
            _ => {}
          }
        }
        EngineEvent::TaskFinished { .. } => {}
        EngineEvent::TaskFailed { error, .. } => {
          return Err(HostFault::error(
            HostFaultPhase::Boot,
            HostFaultDomain::Package,
            format!("initial package scan failed: {error}"),
          ));
        }
        other => services.engine_events.push(other),
      }
    }
    if !finished {
      thread::sleep(Duration::from_millis(16));
    }
  }
  present_progress(services, BootProgress::package(total, total))
}

fn present_progress(
  services: &mut EngineServices,
  progress: BootProgress,
) -> Result<(), HostFault> {
  if let Ok((width, height)) = crossterm::terminal::size()
    && (width != services.layout.physical_width() || height != services.layout.physical_height())
  {
    services.layout.resize_physical(width, height);
    services.canvas.resize(width, height);
    services.presenter.request_render();
  }
  with_fault_domain(HostFaultDomain::Render, || {
    services.canvas.begin_frame(&services.layout);
    BootLoadingUi::render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      progress,
    );
    let force_redraw = services.canvas.take_render_requested();
    let composed = services.compositor.compose(&services.canvas);
    services
      .presenter
      .present(&composed, &mut services.terminal, force_redraw, None)
  })
  .map_err(|error| {
    HostFault::error(
      HostFaultPhase::Boot,
      HostFaultDomain::Render,
      format!("boot loading page presentation failed: {error}"),
    )
  })
}

fn log_stage_complete(services: &mut EngineServices, stage: BootStage) {
  services.log.info_message(
    LogSource::Boot,
    HostLogMessage::new(
      "log_info.boot.stage_complete",
      "Boot stage completed: {stage}.",
    )
    .param("stage", format!("{stage:?}")),
  );
}
