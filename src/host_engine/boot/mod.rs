use crate::host_engine::core::{BootOutput, RuntimeWorld};
use crate::host_engine::services::EngineServices;

use super::services::LogSource;

/// 执行引擎启动准备：扫描语言包与资源包、初始化运行时世界
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();
  let terminal_profile = services
    .storage
    .read_terminal_profile_or_default(&mut services.log);
  services.terminal.apply_capability_profile(
    terminal_profile.unicode,
    terminal_profile.color.as_deref(),
    terminal_profile.mouse,
  );

  services
    .log
    .info(LogSource::Boot, "[Boot] Preparing engine...");

  services
    .log
    .info(LogSource::Boot, "[Boot] Preparing package scan...");

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
        .any(|e| e.code == *code);

      let registry_ok =
        services
          .i18n
          .is_language_package_available(&services.storage, &mut services.log, code);
      if in_registry && registry_ok {
        code.clone()
      } else {
        services.log.warn(
          LogSource::Boot,
          format!(
            "Saved language '{}' invalid (registry={}, available={}), will re-select",
            code, in_registry, registry_ok
          ),
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
  let _ = services.log.refresh_labels_from_i18n(&services.i18n);

  let root_dir = services.storage.root_dir().to_path_buf();
  let package_language = services.i18n.current_language().to_string();
  let missing_template = services
    .i18n
    .get_runtime_text("language_warning", "language_warning.missing");
  services
    .package
    .configure_scan(&root_dir, &package_language, &missing_template);
  let _ = services.package.request_rescan(&services.async_runtime);

  services
    .log
    .info(LogSource::Boot, "[Boot] Package scan scheduled.");

  let world = RuntimeWorld::new();

  services.log.info(
    LogSource::Boot,
    &format!("[Boot] Storage root: {}", root_dir.display()),
  );

  BootOutput { services, world }
}
