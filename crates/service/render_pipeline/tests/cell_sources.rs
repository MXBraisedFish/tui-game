#[path = "../examples/render_pipeline_smoke.rs"]
mod scenario;

#[test]
fn final_cell_sources_preserve_hidden_layers_and_shader_results() {
  scenario::main();
}
