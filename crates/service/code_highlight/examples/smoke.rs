//! Minimal entry: highlights a Rust snippet and checks a keyword token is produced.

use tg_service_code_highlight::{CodeHighlightService, CodeTokenKind};

fn main() {
  let service = CodeHighlightService::new();
  let language = service.language_from_name("rust").expect("rust is supported");
  let source = "fn main() { let x = 1; }";
  let tokens = service.highlight(source, language);
  let keyword = tokens
    .iter()
    .find(|token| token.kind == CodeTokenKind::Keyword)
    .expect("keyword token");
  assert_eq!(&source[keyword.start_byte..keyword.end_byte], "fn");
  println!("code_highlight ok: {} tokens", tokens.len());
}
