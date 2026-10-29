//! Run the independent Markdown scenario as an integration regression.
#[path = "../examples/lua_markdown_smoke.rs"]
mod scenario;
#[test]
fn markdown_documents_and_conversion() {
  scenario::main();
}
