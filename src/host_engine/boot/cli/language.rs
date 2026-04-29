//! CLI 命令行的国际化语言模块
//! 负责加载命令行的多语言文本，提供文本获取和格式化功能

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

/// 默认语言代码（英语-美国）
const DEFAULT_LANGUAGE_CODE: &str = "en_us";
/// 语言配置文件的存储路径
const LANGUAGE_PROFILE_PATH: &str = "data/profiles/language.txt";
/// 命令行语言文件目录
const COMMAND_LANGUAGE_DIR: &str = "assets/bash_lang";

// ========== 静态语言文本存储（OnceCell 单例） ==========

/// 当前版本号显示文本
pub static CLI_VERSION_CURRENT: OnceCell<String> = OnceCell::new();
/// 最新版本号显示文本
pub static CLI_VERSION_LATEST: OnceCell<String> = OnceCell::new();
/// 版本可更新提示文本
pub static CLI_VERSION_IS_UPDATE: OnceCell<String> = OnceCell::new();
/// 版本已是最新提示文本
pub static CLI_VERSION_IS_NEW: OnceCell<String> = OnceCell::new();
/// 版本下载链接显示文本
pub static CLI_VERSION_URL: OnceCell<String> = OnceCell::new();
/// 版本检查失败提示文本
pub static CLI_VERSION_CHECK_FAILED: OnceCell<String> = OnceCell::new();
/// 未知参数错误提示文本
pub static CLI_ERROR_UNKNOWN_ARG: OnceCell<String> = OnceCell::new();
/// 通用帮助错误提示文本
pub static CLI_ERROR_HELP: OnceCell<String> = OnceCell::new();
/// 缺失语言键错误提示模板
pub static CLI_ERROR_MISSING_KEY: OnceCell<String> = OnceCell::new();
/// 路径显示标签文本
pub static CLI_PATH: OnceCell<String> = OnceCell::new();
/// 帮助信息中"用法"段落模板
pub static CLI_HELP_USE: OnceCell<String> = OnceCell::new();
/// 帮助信息头部模板
pub static CLI_HELP_HEADER: OnceCell<String> = OnceCell::new();
/// 帮助信息中"无参数"说明模板
pub static CLI_HELP_NO_ARG: OnceCell<String> = OnceCell::new();
/// 帮助信息中"帮助"命令说明
pub static CLI_HELP_HELP: OnceCell<String> = OnceCell::new();
/// 帮助信息中"API 版本"命令说明
pub static CLI_HELP_API: OnceCell<String> = OnceCell::new();
/// 帮助信息中"修复"命令说明
pub static CLI_HELP_FIX: OnceCell<String> = OnceCell::new();
/// 帮助信息中"路径"命令说明
pub static CLI_HELP_PATH: OnceCell<String> = OnceCell::new();
/// 帮助信息中"版本"命令说明
pub static CLI_HELP_VERSION: OnceCell<String> = OnceCell::new();
/// 帮助信息中"清空缓存"命令说明
pub static CLI_HELP_CLEAR_CACHE: OnceCell<String> = OnceCell::new();
/// 帮助信息中"清空数据"命令说明
pub static CLI_HELP_CLEAR_DATA: OnceCell<String> = OnceCell::new();
/// 清空缓存对话框标题
pub static CLI_CLEAR_CACHE_TITLE: OnceCell<String> = OnceCell::new();
/// 清空缓存日志提示
pub static CLI_CLEAR_CACHE_LOG: OnceCell<String> = OnceCell::new();
/// 缓存目录显示标签
pub static CLI_CLEAR_CACHE_CACHE: OnceCell<String> = OnceCell::new();
/// 清空缓存警告文本
pub static CLI_CLEAR_CACHE_WARN: OnceCell<String> = OnceCell::new();
/// 清空缓存"否"选项文本
pub static CLI_CLEAR_CACHE_NO: OnceCell<String> = OnceCell::new();
/// 清空缓存"是"选项文本
pub static CLI_CLEAR_CACHE_YES: OnceCell<String> = OnceCell::new();
/// 清空数据对话框标题
pub static CLI_CLEAR_DATA_TITLE: OnceCell<String> = OnceCell::new();
/// 数据目录显示标签
pub static CLI_CLEAR_DATA_DATA: OnceCell<String> = OnceCell::new();
/// 清空数据警告文本
pub static CLI_CLEAR_DATA_WARN: OnceCell<String> = OnceCell::new();
/// 清空数据"否"选项文本
pub static CLI_CLEAR_DATA_NO: OnceCell<String> = OnceCell::new();
/// 清空数据"是"选项文本
pub static CLI_CLEAR_DATA_YES: OnceCell<String> = OnceCell::new();
/// API 版本显示文本
pub static CLI_API: OnceCell<String> = OnceCell::new();

/// 命令行语言数据存储结构
/// 包含首选语言的文本和 fallback 语言的文本
#[derive(Default)]
struct CommandLanguage {
    /// 用户首选语言的文本映射（键 -> 文本）
    preferred_texts: HashMap<String, String>,
    /// 备选语言的文本映射（en_us）
    fallback_texts: HashMap<String, String>,
}

/// 加载所有命令行语言文本
pub fn load() {
    let command_language = load_command_language();

    // 将各语言键设置到对应的静态变量中
    set_text(
        &CLI_VERSION_CURRENT,
        &command_language,
        "cli.version.current",
    );
    set_text(&CLI_VERSION_LATEST, &command_language, "cli.version.latest");
    set_text(
        &CLI_VERSION_IS_UPDATE,
        &command_language,
        "cli.version.is_update",
    );
    set_text(&CLI_VERSION_IS_NEW, &command_language, "cli.version.is_new");
    set_text(&CLI_VERSION_URL, &command_language, "cli.version.url");
    set_text(
        &CLI_VERSION_CHECK_FAILED,
        &command_language,
        "cli.version.check_failed",
    );
    set_text(
        &CLI_ERROR_UNKNOWN_ARG,
        &command_language,
        "cli.error.unknown_arg",
    );
    set_text(&CLI_ERROR_HELP, &command_language, "cli.error.help");
    set_text(
        &CLI_ERROR_MISSING_KEY,
        &command_language,
        "cli.error.missing_key",
    );
    set_text(&CLI_PATH, &command_language, "cli.path");
    set_text(&CLI_HELP_USE, &command_language, "cli.help.use");
    set_text(&CLI_HELP_HEADER, &command_language, "cli.help.header");
    set_text(&CLI_HELP_NO_ARG, &command_language, "cli.help.no_arg");
    set_text(&CLI_HELP_HELP, &command_language, "cli.help.help");
    set_text(&CLI_HELP_API, &command_language, "cli.help.api");
    set_text(&CLI_HELP_FIX, &command_language, "cli.help.fix");
    set_text(&CLI_HELP_PATH, &command_language, "cli.help.path");
    set_text(&CLI_HELP_VERSION, &command_language, "cli.help.version");
    set_text(
        &CLI_HELP_CLEAR_CACHE,
        &command_language,
        "cli.help.clear_cache",
    );
    set_text(
        &CLI_HELP_CLEAR_DATA,
        &command_language,
        "cli.help.clear_data",
    );
    set_text(
        &CLI_CLEAR_CACHE_TITLE,
        &command_language,
        "cli.clear_cache.title",
    );
    set_text(
        &CLI_CLEAR_CACHE_LOG,
        &command_language,
        "cli.clear_cache.log",
    );
    set_text(
        &CLI_CLEAR_CACHE_CACHE,
        &command_language,
        "cli.clear_cache.cache",
    );
    set_text(
        &CLI_CLEAR_CACHE_WARN,
        &command_language,
        "cli.clear_cache.warn",
    );
    set_text(&CLI_CLEAR_CACHE_NO, &command_language, "cli.clear_cache.no");
    set_text(
        &CLI_CLEAR_CACHE_YES,
        &command_language,
        "cli.clear_cache.yes",
    );
    set_text(
        &CLI_CLEAR_DATA_TITLE,
        &command_language,
        "cli.clear_data.title",
    );
    set_text(
        &CLI_CLEAR_DATA_DATA,
        &command_language,
        "cli.clear_data.data",
    );
    set_text(
        &CLI_CLEAR_DATA_WARN,
        &command_language,
        "cli.clear_data.warn",
    );
    set_text(&CLI_CLEAR_DATA_NO, &command_language, "cli.clear_data.no");
    set_text(&CLI_CLEAR_DATA_YES, &command_language, "cli.clear_data.yes");
    set_text(&CLI_API, &command_language, "cli.api");
}

/// 获取静态文本内容
pub fn text(cell: &'static OnceCell<String>) -> &'static str {
    cell.get().map(String::as_str).unwrap_or("")
}

/// 获取并格式化静态文本（支持占位符替换）
/// 占位符格式：`{name}`
pub fn format_text(cell: &'static OnceCell<String>, replacements: &[(&str, &str)]) -> String {
    let mut text = text(cell).to_string();
    for (name, replacement) in replacements {
        text = text.replace(&format!("{{{name}}}"), replacement);
    }
    text
}

/// 加载命令行语言数据
fn load_command_language() -> CommandLanguage {
    let root_dir = root_dir();
    // 从配置文件读取用户首选语言
    let preferred_code =
        load_language_preference(&root_dir).unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string());
    // 加载用户首选语言的文本
    let preferred_texts = load_language_file(&root_dir, &preferred_code).unwrap_or_default();
    // 加载备选语言文本
    let fallback_texts =
        load_language_file(&root_dir, DEFAULT_LANGUAGE_CODE).unwrap_or_else(|| {
            // 备选语言缺失时触发修复
            repair_command_language_files();
            HashMap::new()
        });

    CommandLanguage {
        preferred_texts,
        fallback_texts,
    }
}

/// 从配置文件读取用户语言偏好
fn load_language_preference(root_dir: &Path) -> Option<String> {
    let raw_language = fs::read_to_string(root_dir.join(LANGUAGE_PROFILE_PATH)).ok()?;
    let language_code = raw_language.trim();
    if language_code.is_empty() {
        None
    } else {
        Some(language_code.to_string())
    }
}

/// 加载指定语言的 JSON 文本文件
fn load_language_file(root_dir: &Path, language_code: &str) -> Option<HashMap<String, String>> {
    let language_path = root_dir
        .join(COMMAND_LANGUAGE_DIR)
        .join(format!("{language_code}.json"));
    let raw_json = fs::read_to_string(language_path).ok()?;
    serde_json::from_str::<HashMap<String, String>>(&raw_json).ok()
}

/// 获取程序根目录（可执行文件所在目录或当前目录）
fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 设置静态文本的值
fn set_text(cell: &'static OnceCell<String>, command_language: &CommandLanguage, key: &str) {
    let _ = cell.set(
        command_language
            .find_text(key)
            .unwrap_or_else(|| missing_text(key)),
    );
}

/// 修复命令行语言文件（占位函数，实际修复由环境修复模块完成）
fn repair_command_language_files() {
    // Placeholder: the environment repair module will restore command language
    // files in a later step.
}

/// 生成缺失语言键的提示文本
fn missing_text(key: &str) -> String {
    let template = CLI_ERROR_MISSING_KEY
        .get()
        .map(String::as_str)
        .unwrap_or("[Missing i18n key: {key}]");
    template.replace("{key}", key)
}

impl CommandLanguage {
    /// 查找指定键的文本
    /// 查找顺序：首选语言 → 备选语言 → 触发修复 → 返回缺失模板
    fn find_text(&self, key: &str) -> Option<String> {
        self.preferred_texts
            .get(key)
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| {
                self.fallback_texts
                    .get(key)
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            })
            .or_else(|| {
                // 触发修复（可能重新加载文件）
                repair_command_language_files();
                None
            })
    }
}
