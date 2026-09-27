//! Extracts the module tree, items, imports and qualified paths of a Rust
//! crate by following `mod` declarations from its root file.
//!
//! Usage: arch_extractor <crate-root-file> <repo-root> > out.json

use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use serde_json::{Value, json};
use syn::spanned::Spanned;
use syn::visit::Visit;

#[derive(Default)]
struct Output {
  files: Vec<Value>,
  mods: Vec<Value>,
  items: Vec<Value>,
  uses: Vec<Value>,
  paths: Vec<Value>,
  errors: Vec<Value>,
}

#[derive(Clone)]
struct Scope {
  file: String,
  module: String,
  test: bool,
}

fn vis_str(vis: &syn::Visibility) -> String {
  match vis {
    syn::Visibility::Public(_) => "pub".into(),
    syn::Visibility::Restricted(restricted) => format!(
      "pub({})",
      restricted.path.to_token_stream().to_string().replace(' ', "")
    ),
    syn::Visibility::Inherited => String::new(),
  }
}

fn doc_of(attrs: &[syn::Attribute]) -> String {
  let mut lines = Vec::new();
  for attr in attrs {
    if !attr.path().is_ident("doc") {
      continue;
    }
    if let syn::Meta::NameValue(name_value) = &attr.meta
      && let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(text),
        ..
      }) = &name_value.value
    {
      lines.push(text.value().trim().to_string());
    }
  }
  lines.join("\n")
}

fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
  attrs.iter().any(|attr| {
    attr.path().is_ident("cfg") && {
      let text = attr.meta.to_token_stream().to_string();
      text.contains("test") && !text.contains("not (test")
    }
  })
}

fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
  attrs.iter().any(|attr| {
    attr
      .path()
      .segments
      .last()
      .is_some_and(|segment| segment.ident == "test")
  })
}

fn path_attr(attrs: &[syn::Attribute]) -> Option<String> {
  for attr in attrs {
    if attr.path().is_ident("path")
      && let syn::Meta::NameValue(name_value) = &attr.meta
      && let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(text),
        ..
      }) = &name_value.value
    {
      return Some(text.value());
    }
  }
  None
}

fn flatten_use(tree: &syn::UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, String)>) {
  match tree {
    syn::UseTree::Path(path) => {
      prefix.push(path.ident.to_string());
      flatten_use(&path.tree, prefix, out);
      prefix.pop();
    }
    syn::UseTree::Name(name) => {
      let mut full = prefix.clone();
      full.push(name.ident.to_string());
      out.push((full, name.ident.to_string()));
    }
    syn::UseTree::Rename(rename) => {
      let mut full = prefix.clone();
      full.push(rename.ident.to_string());
      out.push((full, rename.rename.to_string()));
    }
    syn::UseTree::Glob(_) => {
      let mut full = prefix.clone();
      full.push("*".into());
      out.push((full, "*".into()));
    }
    syn::UseTree::Group(group) => {
      for item in &group.items {
        flatten_use(item, prefix, out);
      }
    }
  }
}

fn short_type(ty: &syn::Type) -> String {
  match ty {
    syn::Type::Path(type_path) => type_path
      .path
      .segments
      .last()
      .map(|segment| segment.ident.to_string())
      .unwrap_or_default(),
    other => other.to_token_stream().to_string(),
  }
}

fn fn_signature(sig: &syn::Signature) -> String {
  let text = sig.to_token_stream().to_string();
  text
    .replace(" (", "(")
    .replace("( ", "(")
    .replace(" )", ")")
    .replace(" ,", ",")
    .replace(" :", ":")
    .replace("& ", "&")
    .replace(" < ", "<")
    .replace(" <", "<")
    .replace("< ", "<")
    .replace(" >", ">")
    .replace(":: ", "::")
    .replace(" ::", "::")
}

struct PathCollector<'a> {
  output: &'a mut Output,
  scope: Scope,
}

impl<'ast> Visit<'ast> for PathCollector<'_> {
  fn visit_path(&mut self, path: &'ast syn::Path) {
    let segments: Vec<String> = path
      .segments
      .iter()
      .map(|segment| segment.ident.to_string())
      .collect();
    if segments.len() >= 2 || segments.first().is_some_and(|first| first.chars().next().is_some_and(char::is_uppercase)) {
      self.output.paths.push(json!({
        "segs": segments,
        "line": path.span().start().line,
        "module": self.scope.module,
        "file": self.scope.file,
        "test": self.scope.test,
      }));
    }
    syn::visit::visit_path(self, path);
  }

  fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
    record_use(self.output, &self.scope, item, true);
  }
}

fn record_use(output: &mut Output, scope: &Scope, item: &syn::ItemUse, local: bool) {
  let mut flattened = Vec::new();
  flatten_use(&item.tree, &mut Vec::new(), &mut flattened);
  for (full, alias) in flattened {
    output.uses.push(json!({
      "segs": full,
      "alias": alias,
      "vis": vis_str(&item.vis),
      "line": item.span().start().line,
      "module": scope.module,
      "file": scope.file,
      "test": scope.test,
      "local": local,
    }));
  }
}

fn collect_paths<T: ?Sized>(output: &mut Output, scope: &Scope, node: &T, visit: fn(&mut PathCollector, &T)) {
  let mut collector = PathCollector {
    output,
    scope: scope.clone(),
  };
  visit(&mut collector, node);
}

#[allow(clippy::too_many_arguments)]
fn push_item(
  output: &mut Output,
  scope: &Scope,
  kind: &str,
  name: String,
  vis: String,
  span: proc_macro2::Span,
  attrs: &[syn::Attribute],
  parent: Option<String>,
  trait_name: Option<String>,
  signature: Option<String>,
) {
  let test = scope.test || has_cfg_test(attrs) || has_test_attr(attrs);
  output.items.push(json!({
    "kind": kind,
    "name": name,
    "vis": vis,
    "line": span.start().line,
    "end": span.end().line,
    "module": scope.module,
    "file": scope.file,
    "test": test,
    "doc": doc_of(attrs),
    "parent": parent,
    "trait": trait_name,
    "sig": signature,
  }));
}

fn walk_items(output: &mut Output, items: &[syn::Item], scope: &Scope, file_path: &Path, child_dir: &Path, repo_root: &Path) {
  for item in items {
    match item {
      syn::Item::Mod(module) => {
        let name = module.ident.to_string();
        let test = scope.test || has_cfg_test(&module.attrs);
        let child_scope = Scope {
          file: scope.file.clone(),
          module: format!("{}::{}", scope.module, name),
          test,
        };
        output.mods.push(json!({
          "parent": scope.module,
          "name": name,
          "module": child_scope.module,
          "vis": vis_str(&module.vis),
          "line": module.span().start().line,
          "inline": module.content.is_some(),
          "test": test,
          "file": scope.file,
          "doc": doc_of(&module.attrs),
        }));
        if let Some((_, inner)) = &module.content {
          walk_items(output, inner, &child_scope, file_path, &child_dir.join(&name), repo_root);
          continue;
        }
        let candidates: Vec<PathBuf> = match path_attr(&module.attrs) {
          Some(relative) => vec![file_path.parent().unwrap().join(relative)],
          None => vec![
            child_dir.join(format!("{name}.rs")),
            child_dir.join(&name).join("mod.rs"),
          ],
        };
        match candidates.iter().find(|candidate| candidate.exists()) {
          Some(found) => walk_file(output, found, &child_scope.module, test, repo_root),
          None => output.errors.push(json!({
            "error": "module file not found",
            "module": child_scope.module,
            "file": scope.file,
          })),
        }
      }
      syn::Item::Use(item_use) => record_use(output, scope, item_use, false),
      syn::Item::Fn(function) => {
        push_item(
          output,
          scope,
          "fn",
          function.sig.ident.to_string(),
          vis_str(&function.vis),
          function.span(),
          &function.attrs,
          None,
          None,
          Some(fn_signature(&function.sig)),
        );
        let fn_scope = Scope {
          test: scope.test || has_cfg_test(&function.attrs) || has_test_attr(&function.attrs),
          ..scope.clone()
        };
        collect_paths(output, &fn_scope, function, |collector, node| {
          collector.visit_item_fn(node)
        });
      }
      syn::Item::Struct(item_struct) => {
        push_item(output, scope, "struct", item_struct.ident.to_string(), vis_str(&item_struct.vis), item_struct.span(), &item_struct.attrs, None, None, None);
        collect_paths(output, scope, item_struct, |collector, node| collector.visit_item_struct(node));
      }
      syn::Item::Enum(item_enum) => {
        push_item(output, scope, "enum", item_enum.ident.to_string(), vis_str(&item_enum.vis), item_enum.span(), &item_enum.attrs, None, None, None);
        collect_paths(output, scope, item_enum, |collector, node| collector.visit_item_enum(node));
      }
      syn::Item::Union(item_union) => {
        push_item(output, scope, "union", item_union.ident.to_string(), vis_str(&item_union.vis), item_union.span(), &item_union.attrs, None, None, None);
      }
      syn::Item::Trait(item_trait) => {
        let trait_name = item_trait.ident.to_string();
        push_item(output, scope, "trait", trait_name.clone(), vis_str(&item_trait.vis), item_trait.span(), &item_trait.attrs, None, None, None);
        for trait_item in &item_trait.items {
          if let syn::TraitItem::Fn(function) = trait_item {
            push_item(output, scope, "trait_fn", function.sig.ident.to_string(), String::new(), function.span(), &function.attrs, Some(trait_name.clone()), None, Some(fn_signature(&function.sig)));
          }
        }
        collect_paths(output, scope, item_trait, |collector, node| collector.visit_item_trait(node));
      }
      syn::Item::Const(item_const) => {
        push_item(output, scope, "const", item_const.ident.to_string(), vis_str(&item_const.vis), item_const.span(), &item_const.attrs, None, None, None);
        collect_paths(output, scope, item_const, |collector, node| collector.visit_item_const(node));
      }
      syn::Item::Static(item_static) => {
        push_item(output, scope, "static", item_static.ident.to_string(), vis_str(&item_static.vis), item_static.span(), &item_static.attrs, None, None, None);
        collect_paths(output, scope, item_static, |collector, node| collector.visit_item_static(node));
      }
      syn::Item::Type(item_type) => {
        push_item(output, scope, "type", item_type.ident.to_string(), vis_str(&item_type.vis), item_type.span(), &item_type.attrs, None, None, None);
        collect_paths(output, scope, item_type, |collector, node| collector.visit_item_type(node));
      }
      syn::Item::Macro(item_macro) => {
        if let Some(ident) = &item_macro.ident {
          push_item(output, scope, "macro", ident.to_string(), String::new(), item_macro.span(), &item_macro.attrs, None, None, None);
        }
      }
      syn::Item::Impl(item_impl) => {
        let self_type = short_type(&item_impl.self_ty);
        let trait_name = item_impl.trait_.as_ref().map(|(_, path, _)| {
          path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default()
        });
        let impl_test = scope.test || has_cfg_test(&item_impl.attrs);
        let impl_scope = Scope {
          test: impl_test,
          ..scope.clone()
        };
        output.items.push(json!({
          "kind": "impl",
          "name": self_type,
          "vis": "",
          "line": item_impl.span().start().line,
          "end": item_impl.span().end().line,
          "module": scope.module,
          "file": scope.file,
          "test": impl_test,
          "doc": doc_of(&item_impl.attrs),
          "parent": Value::Null,
          "trait": trait_name,
          "sig": Value::Null,
        }));
        for impl_item in &item_impl.items {
          match impl_item {
            syn::ImplItem::Fn(function) => push_item(
              output,
              &impl_scope,
              "method",
              function.sig.ident.to_string(),
              vis_str(&function.vis),
              function.span(),
              &function.attrs,
              Some(self_type.clone()),
              trait_name.clone(),
              Some(fn_signature(&function.sig)),
            ),
            syn::ImplItem::Const(item_const) => push_item(
              output,
              &impl_scope,
              "assoc_const",
              item_const.ident.to_string(),
              vis_str(&item_const.vis),
              item_const.span(),
              &item_const.attrs,
              Some(self_type.clone()),
              trait_name.clone(),
              None,
            ),
            _ => {}
          }
        }
        collect_paths(output, &impl_scope, item_impl, |collector, node| collector.visit_item_impl(node));
      }
      _ => {}
    }
  }
}

fn relative(path: &Path, repo_root: &Path) -> String {
  path
    .strip_prefix(repo_root)
    .unwrap_or(path)
    .to_string_lossy()
    .replace('\\', "/")
}

fn walk_file(output: &mut Output, file_path: &Path, module: &str, test: bool, repo_root: &Path) {
  let source = match fs::read_to_string(file_path) {
    Ok(source) => source,
    Err(error) => {
      output.errors.push(json!({"error": error.to_string(), "file": relative(file_path, repo_root)}));
      return;
    }
  };
  let parsed = match syn::parse_file(&source) {
    Ok(parsed) => parsed,
    Err(error) => {
      output.errors.push(json!({"error": error.to_string(), "file": relative(file_path, repo_root)}));
      return;
    }
  };
  let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
  let is_dir_owner = matches!(file_name.as_str(), "mod.rs" | "main.rs" | "lib.rs");
  let child_dir = if is_dir_owner {
    file_path.parent().unwrap().to_path_buf()
  } else {
    file_path
      .parent()
      .unwrap()
      .join(file_path.file_stem().unwrap())
  };
  let scope = Scope {
    file: relative(file_path, repo_root),
    module: module.to_string(),
    test,
  };
  output.files.push(json!({
    "file": scope.file,
    "module": module,
    "lines": source.lines().count(),
    "test": test,
    "doc": doc_of(&parsed.attrs),
  }));
  walk_items(output, &parsed.items, &scope, file_path, &child_dir, repo_root);
}

fn main() {
  let mut args = std::env::args().skip(1);
  let root_file = PathBuf::from(args.next().expect("crate root file"));
  let repo_root = PathBuf::from(args.next().expect("repo root"));
  let mut output = Output::default();
  walk_file(&mut output, &root_file, "crate", false, &repo_root);
  let result = json!({
    "files": output.files,
    "mods": output.mods,
    "items": output.items,
    "uses": output.uses,
    "paths": output.paths,
    "errors": output.errors,
  });
  println!("{}", serde_json::to_string(&result).unwrap());
}
