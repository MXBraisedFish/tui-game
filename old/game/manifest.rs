// 定义游戏与包的清单文件数据结构，对应 package.json 和 game.json 的 JSON schema。提供 RuntimeManifest、PackageManifest、GameManifest 三个核心类型，供 package.rs 解析使用

use std::collections::BTreeMap; // 存储动作名称到 ActionBinding 的映射，保持有序

use serde::{Deserialize, Serialize}; // 序列化支持

use crate::game::action::ActionBinding; // 动作绑定类型

// 游戏运行时参数
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct RuntimeManifest {
    #[serde(default)]
    pub target_fps: Option<u16>, // 目标帧率：30/60/120，默认由宿主决定
}

// 对应 package.json 文件。namespace 用于标识 Mod 包，官方游戏可为空
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PackageManifest {
    #[serde(default)]
    pub namespace: String,
    #[serde(default, alias = "package")]
    pub package_name: String,
    #[serde(default)]
    pub mod_name: Option<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub introduction: Option<String>,
    #[serde(default, alias = "name")]
    pub game_name: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    #[serde(alias = "thumbnail")]
    pub icon: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<serde_json::Value>,
    #[serde(default)]
    pub api_version: Option<serde_json::Value>,
}

// 对应 game.json 文件。id 字段由宿主生成，不从 JSON 读取
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GameManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub introduction: Option<String>,
    #[serde(default)]
    pub icon: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<serde_json::Value>,
    pub entry: String,
    #[serde(default)]
    pub save: bool,
    #[serde(default)]
    pub best_none: Option<String>,
    #[serde(default)]
    pub min_width: Option<u16>,
    #[serde(default)]
    pub min_height: Option<u16>,
    #[serde(default)]
    pub max_width: Option<u16>,
    #[serde(default)]
    pub max_height: Option<u16>,
    #[serde(default)]
    pub actions: BTreeMap<String, ActionBinding>,
    #[serde(default)]
    pub runtime: RuntimeManifest,
    #[serde(default)]
    pub api: Option<serde_json::Value>,
    #[serde(default)]
    pub write: bool,
    #[serde(default)]
    pub case_sensitive: bool,
}
