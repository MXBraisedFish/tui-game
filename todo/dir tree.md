```lua
src/                        - 根目录
├ host_engine/              - 引擎逻辑目录
│ ├ boot/                   - 启动逻辑目录
│ │ ├ boot_output.rs        - 启动输出结构体
│ │ └ mod.rs                - 启动逻辑代码
│ ├ core/                   - 核心逻辑目录
│ │ ├ boot_output.rs        - 启动输出结构体
│ │ ├ exit_state.rs         - 退出信息块结构体
│ │ ├ frame.rs              - 基本帧循环
│ │ ├ mod.rs                - 核心逻辑导出
│ │ └ world.rs              - 运行时世界结构体
│ ├ runtime/                - 运行逻辑目录
│ │ └ mod.rs                - 运行逻辑代码
│ ├ services/               - 引擎服务项目目录
│ │ ├ game.rs               - 负责游戏职责
│ │ ├ input.rs              - 负责输入职责
│ │ ├ lua.rs                - 负责Lua环境职责
│ │ ├ mod.rs                - 统一导出，和引擎服务结构体
│ │ ├ overlay.rs            - 负责覆盖屏幕职责
│ │ ├ package.rs            - 包管理职责
│ │ ├ render.rs             - 画布渲染职责
│ │ ├ storage.rs            - 数据管理职责
│ │ └ ui.rs                 - ui控件职责
│ ├ shutdown/               - 关闭逻辑目录
│ │ └ mod.rs                - 关闭逻辑代码
│ └ mod.rs                  - 统一导出三个运行状态，三个阶段主入口
└ main.rs                   - 入口代码
```

```json
{
  "schema_version": 1, -- 配置解析版本号（int） *
  "type": "game", -- 包类型（game|screensaver|boss） *
  "package": "com.example.mygame", -- 包全局唯一标志符（str） *
  "namespace": "mygame", -- 命名空间（str） *
  "version": "1.0.0", -- 展示版本号（str）
  "version_code": 1, -- 版本号真值（int） *
  "api": { -- api版本支持（int|object） *
    "min": 1, -- 最小api版本支持（int）
    "max": 2 -- 最大api版本支持（int）
  },
  "entry": "init.lua", -- 脚本入口（str） *
  "display": { -- 包信息展示（object） *
    "title": "My Game Pack", -- 包名（str|language_key） *
    "description": "A collection of classic terminal games.", -- 包简介（str|language_key） *
    "author": "Alex", -- 作者（str|language_key） *
    "icon": "pack_icon.png", -- 包图标（Array<string>|image）
    "banner": "pack_banner.png" -- 包图标（Array<string>|image）
  },
  
  "runtime": { -- 游戏包运行时设置（object） * G
    "min_width": 60, -- 最小宽度（int）
    "min_height": 20, -- 最小高度（int）
    "write": false, -- 写请求（bool） *
    "target_fps": 30 -- FPS限制（object） *
  },
  "game": { -- 游戏包特有信息（object） * G
    "name": "Minefield", -- 游戏名字（str） *
    "detail": "A full-featured minesweeper...", -- 游戏细节（str） *
    "save": true, -- 是否可存档（bool） *
    "score": { -- 最佳记录存储（object） *
      "enabled": true, -- 是否启用（bool） *
      "empty_text": "$game.no_record" -- 无记录时显示的代替内容（str|language_key）
    },
    "actions": { -- 动作注册表（object）
      "move_up": { -- 动作（object） *
	    "description": "Jump", -- 动作描述（str|language_key） *
	    "keys": [ -- 动作按键（Array<Array>） *
		  ["shift", "w"], -- 物理键（Array<string>） *
		  ["space"]
	    ]
      }
    }
  },
  
  "screensaver": { -- 屏保包特有信息（object） * S
    "name": "Minefield", -- 屏保名字（str） *
  },
  
  "boss": { -- 老板包特有信息（object） * B
    "name": "Minefield", -- 老板界面名字（str） *
  }
}
```
