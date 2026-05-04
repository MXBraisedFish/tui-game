//! Lua API 公开作用域

/// API 使用方。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiConsumer {
    /// 游戏包脚本。
    GamePackage,
    /// 官方 UI 包脚本。
    OfficialUiPackage,
}

/// API 公开作用域。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApiScope {
    pub consumer: ApiConsumer,
}

impl ApiScope {
    /// 游戏包 API 作用域。
    pub fn game_package() -> Self {
        Self {
            consumer: ApiConsumer::GamePackage,
        }
    }

    /// 官方 UI 包 API 作用域。
    pub fn official_ui_package() -> Self {
        Self {
            consumer: ApiConsumer::OfficialUiPackage,
        }
    }

    /// 当前作用域是否允许使用游戏专属声明式 API。
    pub fn allows_game_callbacks(self) -> bool {
        matches!(self.consumer, ApiConsumer::GamePackage)
    }

    /// 当前作用域是否允许使用 UI 共享声明式 API。
    pub fn allows_ui_callbacks(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许使用调试日志 API。
    pub fn allows_debug_log(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许查询游戏元信息。
    pub fn allows_game_debug_info(self) -> bool {
        matches!(self.consumer, ApiConsumer::GamePackage)
    }

    /// 当前作用域是否允许查询按键映射。
    pub fn allows_key_query(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许使用游戏系统查询 API。
    pub fn allows_game_system_query(self) -> bool {
        matches!(self.consumer, ApiConsumer::GamePackage)
    }

    /// 当前作用域是否允许发送通用系统请求。
    pub fn allows_common_system_request(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许发送游戏存储请求。
    pub fn allows_game_storage_request(self) -> bool {
        matches!(self.consumer, ApiConsumer::GamePackage)
    }

    /// 当前作用域是否允许绘制画布。
    pub fn allows_canvas_drawing(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许内容尺寸计算。
    pub fn allows_measurement(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许布局定位计算。
    pub fn allows_layout(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许读取数据。
    pub fn allows_file_reading(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许写入数据。
    pub fn allows_file_writing(self) -> bool {
        matches!(self.consumer, ApiConsumer::GamePackage)
    }

    /// 当前作用域是否允许表处理工具。
    pub fn allows_table_utilities(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许加载辅助脚本。
    pub fn allows_module_loading(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许时间处理。
    pub fn allows_timer(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }

    /// 当前作用域是否允许随机数处理。
    pub fn allows_random(self) -> bool {
        matches!(
            self.consumer,
            ApiConsumer::GamePackage | ApiConsumer::OfficialUiPackage
        )
    }
}
