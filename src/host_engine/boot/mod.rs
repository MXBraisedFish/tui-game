// 引用结构体
use crate::host_engine::core::{BootOutput, RuntimeWorld};
use crate::host_engine::services::EngineServices;

// 日志
use super::services::LogSource;

// 启动函数
pub fn prepare() -> BootOutput {
  let mut services = EngineServices::new();

  services
    .log
    .info(LogSource::Boot, "[Boot] Preparing engine...");

  services
    .log
    .info(LogSource::Boot, "[Boot] Scanning packages...");

  // ── 语言加载（含多层保底） ──
  services
    .i18n
    .refresh_language_registry(&services.storage, &mut services.log);

  let default_code = services.storage.default_language_code().to_string();
  let preferred = services.storage.read_language_code();

  let selected_language = match preferred {
    None => {
      // 无已保存语言 → runtime 会进入 LanguageSelect
      default_code
    }
    Some(ref code) => {
      // 校验：注册表是否有该 code
      let in_registry = services
        .i18n
        .language_registry()
        .iter()
        .any(|e| e.code == *code);
      // 校验：语言目录是否存在
      let dir_ok =
        services
          .i18n
          .is_language_package_available(&services.storage, &mut services.log, code);
      if in_registry && dir_ok {
        code.clone()
      } else {
        // 非法 → 清空 profile 让 runtime 进入 LanguageSelect
        services.log.warn(
          LogSource::Boot,
          format!(
            "Saved language '{}' invalid (registry={}, dir={}), will re-select",
            code, in_registry, dir_ok
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

  let root_dir = services.storage.root_dir();
  services.package.scan_all(&root_dir, &mut services.log);

  services
    .log
    .info(LogSource::Boot, "[Boot] Packages scan completed.");

  let world = RuntimeWorld::new();

  services.log.info(
    LogSource::Boot,
    &format!("[Boot] Storage root: {}", root_dir.display()),
  );

  // 返回启动输出
  BootOutput { services, world }
}
