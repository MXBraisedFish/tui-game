//! Minimal entry: loads the embedded en_us fallback and resolves a runtime text.

use tg_service_i18n::I18nService;

fn main() {
  let mut i18n = I18nService::new();
  i18n.load_embedded_fallback();
  let text = i18n.get_runtime_text("exit_warning", "exit_warning.exception.countdown");
  assert_eq!(text, "Exiting in {value:second} seconds");
  println!("i18n ok: {text}");
}
