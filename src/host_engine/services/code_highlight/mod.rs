use std::{collections::HashSet, path::Path, sync::OnceLock};

use crate::host_engine::services::{RichTextSegment, TextColor, TextStyle};
use serde::Deserialize;
use tree_sitter::{Node, Parser, Tree};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeLanguage {
  Rust,
  Python,
  JavaScript,
  TypeScript,
  Tsx,
  Json,
  Toml,
  Yaml,
  Lua,
  Shell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeTokenKind {
  Keyword,
  String,
  Comment,
  Function,
  TypeName,
  Number,
  Operator,
  Punctuation,
  Variable,
  Property,
  Constant,
  Builtin,
  Attribute,
  Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeHighlightToken {
  pub start_byte: usize,
  pub end_byte: usize,
  pub kind: CodeTokenKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeHighlightTheme {
  pub keyword: TextStyle,
  pub string: TextStyle,
  pub comment: TextStyle,
  pub function: TextStyle,
  pub type_name: TextStyle,
  pub number: TextStyle,
  pub operator: TextStyle,
  pub punctuation: TextStyle,
  pub variable: TextStyle,
  pub property: TextStyle,
  pub constant: TextStyle,
  pub builtin: TextStyle,
  pub attribute: TextStyle,
  pub text: TextStyle,
}

pub struct CodeHighlightService;

#[derive(Deserialize)]
struct CodeLightWords {
  keywords: Vec<String>,
  builtins: Vec<String>,
  constants: Vec<String>,
  operators: Vec<String>,
}

struct CodeLightLexicon {
  keywords: HashSet<String>,
  builtins: HashSet<String>,
  constants: HashSet<String>,
  operators: HashSet<String>,
}

const SUPPORTED: &[CodeLanguage] = &[
  CodeLanguage::Rust,
  CodeLanguage::Python,
  CodeLanguage::JavaScript,
  CodeLanguage::TypeScript,
  CodeLanguage::Tsx,
  CodeLanguage::Json,
  CodeLanguage::Toml,
  CodeLanguage::Yaml,
  CodeLanguage::Lua,
  CodeLanguage::Shell,
];

impl CodeHighlightService {
  pub fn new() -> Self {
    Self
  }

  pub fn supported_languages(&self) -> &'static [CodeLanguage] {
    SUPPORTED
  }

  pub fn language_from_name(&self, name: &str) -> Option<CodeLanguage> {
    language_from_name(name)
  }

  pub fn detect_language_from_path(&self, path: &Path) -> Option<CodeLanguage> {
    detect_language_from_path(path)
  }

  pub fn highlight(&self, source: &str, language: CodeLanguage) -> Vec<CodeHighlightToken> {
    let Some(lang) = tree_sitter_language(language) else {
      return Vec::new();
    };
    let mut parser = Parser::new();
    if parser.set_language(&lang).is_err() {
      return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
      return Vec::new();
    };
    collect_tokens(&tree, source, language)
  }

  pub fn highlight_segments(
    &self,
    source: &str,
    language: CodeLanguage,
    theme: &CodeHighlightTheme,
  ) -> Vec<RichTextSegment> {
    let tokens = self.highlight(source, language);
    if tokens.is_empty() {
      return vec![RichTextSegment {
        text: source.to_string(),
        style: theme.text.clone(),
      }];
    }

    let mut segments = Vec::new();
    let mut cursor = 0usize;
    for token in tokens {
      if token.start_byte > cursor {
        push_segment(
          &mut segments,
          &source[cursor..token.start_byte],
          &theme.text,
        );
      }
      if token.end_byte > token.start_byte && token.end_byte <= source.len() {
        push_segment(
          &mut segments,
          &source[token.start_byte..token.end_byte],
          theme.style(token.kind),
        );
        cursor = token.end_byte;
      }
    }
    if cursor < source.len() {
      push_segment(&mut segments, &source[cursor..], &theme.text);
    }
    segments
  }
}

impl Default for CodeHighlightTheme {
  fn default() -> Self {
    Self {
      keyword: fg(210, 204, 255),
      string: fg(164, 223, 174),
      comment: fg(153, 153, 153),
      function: fg(143, 199, 255),
      type_name: TextStyle {
        bold: true,
        ..fg(240, 192, 168)
      },
      number: fg(213, 242, 136),
      operator: fg(184, 215, 249),
      punctuation: fg(184, 215, 249),
      variable: fg(238, 207, 160),
      property: fg(222, 214, 207),
      constant: fg(240, 192, 168),
      builtin: fg(213, 242, 136),
      attribute: fg(143, 199, 255),
      text: fg(203, 213, 225),
    }
  }
}

impl CodeHighlightTheme {
  fn style(&self, kind: CodeTokenKind) -> &TextStyle {
    match kind {
      CodeTokenKind::Keyword => &self.keyword,
      CodeTokenKind::String => &self.string,
      CodeTokenKind::Comment => &self.comment,
      CodeTokenKind::Function => &self.function,
      CodeTokenKind::TypeName => &self.type_name,
      CodeTokenKind::Number => &self.number,
      CodeTokenKind::Operator => &self.operator,
      CodeTokenKind::Punctuation => &self.punctuation,
      CodeTokenKind::Variable => &self.variable,
      CodeTokenKind::Property => &self.property,
      CodeTokenKind::Constant => &self.constant,
      CodeTokenKind::Builtin => &self.builtin,
      CodeTokenKind::Attribute => &self.attribute,
      CodeTokenKind::Text => &self.text,
    }
  }
}

pub fn language_from_name(name: &str) -> Option<CodeLanguage> {
  match name.trim().to_ascii_lowercase().as_str() {
    "rust" | "rs" => Some(CodeLanguage::Rust),
    "python" | "py" => Some(CodeLanguage::Python),
    "javascript" | "js" | "mjs" | "cjs" => Some(CodeLanguage::JavaScript),
    "typescript" | "ts" => Some(CodeLanguage::TypeScript),
    "tsx" => Some(CodeLanguage::Tsx),
    "json" => Some(CodeLanguage::Json),
    "toml" => Some(CodeLanguage::Toml),
    "yaml" | "yml" => Some(CodeLanguage::Yaml),
    "lua" => Some(CodeLanguage::Lua),
    "shell" | "sh" | "bash" => Some(CodeLanguage::Shell),
    _ => None,
  }
}

pub fn detect_language_from_path(path: &Path) -> Option<CodeLanguage> {
  let ext = path.extension()?.to_str()?;
  language_from_name(ext)
}

fn tree_sitter_language(language: CodeLanguage) -> Option<tree_sitter::Language> {
  Some(match language {
    CodeLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
    CodeLanguage::Python => tree_sitter_python::LANGUAGE.into(),
    CodeLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
    CodeLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    CodeLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
    CodeLanguage::Json => tree_sitter_json::LANGUAGE.into(),
    CodeLanguage::Toml => tree_sitter_toml::LANGUAGE.into(),
    CodeLanguage::Yaml => tree_sitter_yaml::LANGUAGE.into(),
    CodeLanguage::Lua => tree_sitter_lua::LANGUAGE.into(),
    CodeLanguage::Shell => tree_sitter_bash::LANGUAGE.into(),
  })
}

fn collect_tokens(tree: &Tree, source: &str, language: CodeLanguage) -> Vec<CodeHighlightToken> {
  let mut tokens = Vec::new();
  let root = tree.root_node();
  let root_kind = root.kind().to_string();
  walk_tree(root, source, &mut tokens, &root_kind, language);
  tokens.sort_by_key(|token| token.start_byte);
  tokens
}

fn walk_tree(
  node: Node,
  source: &str,
  tokens: &mut Vec<CodeHighlightToken>,
  root_kind: &str,
  language: CodeLanguage,
) {
  let kind = node.kind();
  if node.start_byte() >= node.end_byte() {
    return;
  }
  if node.child_count() == 0 {
    add_leaf_token(node, kind, source, tokens, root_kind, language);
    return;
  }
  let kind = categorize_node(kind);
  if kind != CodeTokenKind::Text {
    tokens.push(CodeHighlightToken {
      start_byte: node.start_byte(),
      end_byte: node.end_byte(),
      kind,
    });
    return;
  }
  for i in 0..node.child_count() {
    if let Some(child) = node.child(i as u32) {
      walk_tree(child, source, tokens, root_kind, language);
    }
  }
}

fn add_leaf_token(
  node: Node,
  kind: &str,
  source: &str,
  tokens: &mut Vec<CodeHighlightToken>,
  root_kind: &str,
  language: CodeLanguage,
) {
  let text = &source[node.start_byte()..node.end_byte()];
  if text.trim().is_empty() {
    return;
  }
  tokens.push(CodeHighlightToken {
    start_byte: node.start_byte(),
    end_byte: node.end_byte(),
    kind: categorize_leaf(kind, node, source, root_kind, language),
  });
}

fn categorize_node(kind: &str) -> CodeTokenKind {
  let kl = kind.to_ascii_lowercase();
  if kl.contains("comment") || kl == "docstring" {
    return CodeTokenKind::Comment;
  }
  if kl.contains("string") || kl.contains("char") || kl.contains("regex") {
    return CodeTokenKind::String;
  }
  if kl == "number" || kl == "integer" || kl == "float" || kl.contains("number") {
    return CodeTokenKind::Number;
  }
  CodeTokenKind::Text
}

fn categorize_leaf(
  kind: &str,
  node: Node,
  source: &str,
  root_kind: &str,
  language: CodeLanguage,
) -> CodeTokenKind {
  let kl = kind.to_ascii_lowercase();
  let parent = node.parent();
  let parent_kind = parent
    .as_ref()
    .map(|parent| parent.kind().to_ascii_lowercase())
    .unwrap_or_default();
  let text = &source[node.start_byte()..node.end_byte()];
  let trimmed = text.trim();

  if kl.contains("comment")
    || parent_kind.contains("comment")
    || (trimmed.starts_with('#') && matches!(root_kind, "module" | "source_file" | "document"))
    || trimmed.starts_with("--")
  {
    return CodeTokenKind::Comment;
  }
  if kl.contains("string")
    || kl == "string_content"
    || kl == "string_fragment"
    || kl == "escape_sequence"
    || parent_kind.contains("string")
    || parent_kind.contains("char")
    || text.starts_with('"')
    || text.starts_with('\'')
    || text.starts_with("r\"")
    || text.starts_with("r'")
    || text.starts_with("`")
  {
    return CodeTokenKind::String;
  }
  if kl == "number" || kl == "integer" || kl == "float" || trimmed.parse::<f64>().is_ok() {
    return CodeTokenKind::Number;
  }
  if is_keyword(language, kind) || is_keyword(language, trimmed) {
    return CodeTokenKind::Keyword;
  }
  if is_builtin(language, kind) || is_builtin(language, trimmed) {
    return CodeTokenKind::Builtin;
  }
  if is_constant(language, kind) || is_constant(language, trimmed) {
    return CodeTokenKind::Constant;
  }
  if kl.contains("type") || kl == "type_identifier" || kl == "primitive_type" {
    return CodeTokenKind::TypeName;
  }
  if is_function_like(&kl, &parent_kind) {
    return CodeTokenKind::Function;
  }
  if kl.contains("attribute")
    || kl.contains("decorator")
    || parent_kind.contains("attribute")
    || parent_kind.contains("decorator")
  {
    return CodeTokenKind::Attribute;
  }
  if kl.contains("operator") || is_operator_symbol(language, text) {
    return CodeTokenKind::Operator;
  }
  if kl.contains("punctuation")
    || kl.contains("delimiter")
    || kl.contains("bracket")
    || kl.contains("brace")
    || kl.contains("paren")
    || (text.len() == 1 && "()[]{},.;:@".contains(text.chars().next().unwrap_or_default()))
  {
    return CodeTokenKind::Punctuation;
  }
  if kl == "identifier" || kl == "name" || kl == "property_identifier" {
    if parent_kind.contains("property")
      || parent_kind.contains("field")
      || parent_kind == "field_expression"
    {
      return CodeTokenKind::Property;
    }
    return CodeTokenKind::Variable;
  }
  CodeTokenKind::Text
}

fn is_function_like(kind: &str, parent_kind: &str) -> bool {
  (kind == "identifier" || kind == "property_identifier" || kind == "name")
    && (parent_kind.contains("function")
      || parent_kind.contains("method")
      || parent_kind == "call"
      || parent_kind == "function_call"
      || parent_kind == "function_definition")
}

fn is_keyword(language: CodeLanguage, s: &str) -> bool {
  code_light(language).keywords.contains(s)
}

fn is_builtin(language: CodeLanguage, s: &str) -> bool {
  code_light(language).builtins.contains(s)
}

fn is_constant(language: CodeLanguage, s: &str) -> bool {
  code_light(language).constants.contains(s)
}

fn is_operator_symbol(language: CodeLanguage, text: &str) -> bool {
  code_light(language).operators.contains(text)
}

fn code_light(language: CodeLanguage) -> &'static CodeLightLexicon {
  static RUST: OnceLock<CodeLightLexicon> = OnceLock::new();
  static PYTHON: OnceLock<CodeLightLexicon> = OnceLock::new();
  static JAVASCRIPT: OnceLock<CodeLightLexicon> = OnceLock::new();
  static TYPESCRIPT: OnceLock<CodeLightLexicon> = OnceLock::new();
  static TSX: OnceLock<CodeLightLexicon> = OnceLock::new();
  static JSON: OnceLock<CodeLightLexicon> = OnceLock::new();
  static TOML: OnceLock<CodeLightLexicon> = OnceLock::new();
  static YAML: OnceLock<CodeLightLexicon> = OnceLock::new();
  static LUA: OnceLock<CodeLightLexicon> = OnceLock::new();
  static SHELL: OnceLock<CodeLightLexicon> = OnceLock::new();

  match language {
    CodeLanguage::Rust => {
      RUST.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/rust.json")))
    }
    CodeLanguage::Python => PYTHON
      .get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/python.json"))),
    CodeLanguage::JavaScript => JAVASCRIPT.get_or_init(|| {
      parse_code_light(include_str!(
        "../../../../assets/code_light/javascript.json"
      ))
    }),
    CodeLanguage::TypeScript => TYPESCRIPT.get_or_init(|| {
      parse_code_light(include_str!(
        "../../../../assets/code_light/typescript.json"
      ))
    }),
    CodeLanguage::Tsx => {
      TSX.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/tsx.json")))
    }
    CodeLanguage::Json => {
      JSON.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/json.json")))
    }
    CodeLanguage::Toml => {
      TOML.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/toml.json")))
    }
    CodeLanguage::Yaml => {
      YAML.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/yaml.json")))
    }
    CodeLanguage::Lua => {
      LUA.get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/lua.json")))
    }
    CodeLanguage::Shell => SHELL
      .get_or_init(|| parse_code_light(include_str!("../../../../assets/code_light/shell.json"))),
  }
}

fn parse_code_light(json: &str) -> CodeLightLexicon {
  let words: CodeLightWords = serde_json::from_str(json).expect("valid code_light json");
  CodeLightLexicon {
    keywords: words.keywords.into_iter().collect(),
    builtins: words.builtins.into_iter().collect(),
    constants: words.constants.into_iter().collect(),
    operators: words.operators.into_iter().collect(),
  }
}

fn push_segment(segments: &mut Vec<RichTextSegment>, text: &str, style: &TextStyle) {
  if !text.is_empty() {
    segments.push(RichTextSegment {
      text: text.to_string(),
      style: style.clone(),
    });
  }
}

fn fg(r: u8, g: u8, b: u8) -> TextStyle {
  TextStyle {
    foreground: Some(TextColor::Rgb { r, g, b }),
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_supported_languages_from_path() {
    let service = CodeHighlightService::new();
    assert_eq!(
      service.detect_language_from_path(Path::new("main.rs")),
      Some(CodeLanguage::Rust)
    );
    assert_eq!(
      service.detect_language_from_path(Path::new("config.yaml")),
      Some(CodeLanguage::Yaml)
    );
    assert_eq!(
      service.detect_language_from_path(Path::new("run.sh")),
      Some(CodeLanguage::Shell)
    );
  }

  #[test]
  fn highlights_rust_without_losing_text() {
    let service = CodeHighlightService::new();
    let source = "fn main() {\n  println!(\"hi\");\n}";
    let segments =
      service.highlight_segments(source, CodeLanguage::Rust, &CodeHighlightTheme::default());
    assert_eq!(
      segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<String>(),
      source
    );
    assert!(!service.highlight(source, CodeLanguage::Rust).is_empty());
  }

  #[test]
  fn highlights_all_declared_languages() {
    let service = CodeHighlightService::new();
    let samples = [
      (CodeLanguage::Rust, "fn main() { let x = 1; }"),
      (CodeLanguage::Python, "def main():\n    print('hi')"),
      (
        CodeLanguage::JavaScript,
        "function main() { console.log('hi'); }",
      ),
      (CodeLanguage::TypeScript, "const x: number = 1;"),
      (CodeLanguage::Tsx, "const x = <div>Hello</div>;"),
      (CodeLanguage::Json, "{\"a\": 1}"),
      (CodeLanguage::Toml, "a = 1"),
      (CodeLanguage::Yaml, "a: 1"),
      (CodeLanguage::Lua, "local x = 1"),
      (CodeLanguage::Shell, "echo hi"),
    ];
    for (language, source) in samples {
      assert!(
        !service.highlight(source, language).is_empty(),
        "{language:?}"
      );
    }
  }

  #[test]
  fn maple_dark_theme_uses_expected_palette() {
    let theme = CodeHighlightTheme::default();

    assert_rgb(&theme.text, 203, 213, 225);
    assert_rgb(&theme.comment, 153, 153, 153);
    assert_rgb(&theme.keyword, 210, 204, 255);
    assert_rgb(&theme.function, 143, 199, 255);
    assert_rgb(&theme.variable, 238, 207, 160);
    assert_rgb(&theme.property, 222, 214, 207);
    assert_rgb(&theme.type_name, 240, 192, 168);
    assert_rgb(&theme.string, 164, 223, 174);
    assert_rgb(&theme.number, 213, 242, 136);
    assert_rgb(&theme.operator, 184, 215, 249);
    assert!(theme.type_name.bold);
  }

  #[test]
  fn code_light_assets_drive_word_categories() {
    for language in SUPPORTED {
      let lexicon = code_light(*language);
      assert!(
        !lexicon.keywords.is_empty()
          || !lexicon.builtins.is_empty()
          || !lexicon.constants.is_empty(),
        "{language:?} words"
      );
      assert!(!lexicon.operators.is_empty(), "{language:?} operators");
    }

    assert!(is_keyword(CodeLanguage::Rust, "fn"));
    assert!(is_builtin(CodeLanguage::Python, "print"));
    assert!(is_constant(CodeLanguage::JavaScript, "undefined"));
    assert!(is_operator_symbol(CodeLanguage::Shell, "&&"));
  }

  fn assert_rgb(style: &TextStyle, r: u8, g: u8, b: u8) {
    assert_eq!(style.foreground, Some(TextColor::Rgb { r, g, b }));
  }
}
