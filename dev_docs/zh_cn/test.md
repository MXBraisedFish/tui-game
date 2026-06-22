# 游戏包

{
  "mod_id": "id", - 模组唯一ID：字符串，由用户自行定义
  "schema_version": 1, - 配置版本：必须等于当前宿主要求
  "type": "game", - 包类型：game/screensaver
  "version": "1.0.0", - 版本号（展示）：可以是任意值，也可以是i18n键
  "version_code": 1, - 版本真值：为正整数，必须递增（但现在我们没有社区管理，只检查类型即可）
  "api": { - 支持的API版本：对象，区间闭合，宿主版本必须包含在范围内
    "min": 1, - 最小范围：正整数，必须小于等于max
    "max": 2 - 最大范围：正整数，必须大于等于max
  },
  "entry": "init.lua", - 入口脚本：相对于包scripts/目录的lua脚本（可以不包含后缀，宿主会自动检测）
  "display": { - 显示信息：对象
    "title": "My Game Pack", - 包标题：字符串或i18n键
    "description": "A collection of classic terminal games.", - 包简介：字符串或i18n键
    "author": "Alex", - 包作者：字符串或i18n键
    "icon": "pack_icon.png", - 包图标：艺术画数组，或相对于包assets/图片路径（支持png，jpg，jpeg）
    "banner": "pack_banner.png" - 包头图：艺术画数组，或相对于包assets/图片路径（支持png，jpg，jpeg）
  },
  "runtime": { - 运行配置：对象
    "min_width": 60, - 最小宽度：正整数，或为0（不限制）
    "min_height": 20, - 最小高度：正整数，或为0（不限制）
  },
  "game": { - 游戏包专属配置：对象
    "name": "Minefield", - 游戏名：字符串或i18n键
    "detail": "A full-featured minesweeper with multiple difficulty levels.", - 游戏简介：字符串或i18n键
    "write": false, - 是否需要写操作请求：布尔值-表示该包需要写请求
	"mouse": false, - 是否需要鼠标操作：布尔值-表示该包需要鼠标操作
    "target_fps": 30 - 默认目标帧率：正整数30，60，120，游戏期望帧率上限
    "save": true, - 是否支持存档：布尔值-表示该游戏允许存档
    "score": { - 是否支持存储最佳纪录：对象
      "enabled": true, - 是否支持：布尔值-表示该游戏需要记录最佳记录
      "empty_text": "$game.no_record" - 当该游戏无最佳纪录数据的时候显示的默认值：字符串或i18n键
    },
    "actions": { - 按键注册：对象
      "move_up": { - 动作名：对象
        "description": "Move cursor up", - 动作描述（用于后续修改按键界面展示）：字符串或i18n键
        "keys": [ - 键内容：数组
          ["w"], - 第一个键：数组（至多两个元素）
          ["arrow_up"] - 第二个键：数组（至多两个元素）
        ]
      },
    }
  }
}

>以下内容要求均同上

# 屏保包

{
  "mod_id": "id", - 模组唯一ID
  "schema_version": 1, - 配置版本
  "type": "screensaver", - 包类型
  "version": "1.0.0", - 版本号（展示）
  "version_code": 1, - 版本真值
  "api": { - 支持的API版本
    "min": 1, - 最小范围
    "max": 2 - 最大范围
  },
  "entry": "init.lua", - 入口脚本
  "display": { - 显示信息
    "title": "My Game Pack", - 包标题
    "description": "A collection of classic terminal games.", - 包简介
    "author": "Alex", - 包作者
    "icon": "pack_icon.png", - 包图标
    "banner": "pack_banner.png" - 包头图
  },
  "runtime": { - 运行配置
    "min_width": 60, - 最小宽度
    "min_height": 20, - 最小高度
  },
  "screensaver": { - 游戏包专属配置
    "name": "Minefield", - 屏保名
  }
}

  

完整表
████████ [调试]包名      █
████████ 作者：          █
████████ 版本：          █
████████ 状态：启用/禁用 █

简洁表
[调试]包名           状态 █

信息展示
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████
████████████████████████████████████████

                               包标题

  
基本信息：
包名：
作者：
版本：

配置信息：
鼠标需求：
直写需求：
安全模式：
调试模式：
启用状态：

简介：


操作：
Q/E翻页
↑/↓切换选项
Esc返回/推出搜索
Enter切换启用状态/提交
D开启或关闭Debug模式
J跳页
L切换详细/简表
S开启/关闭当前包的安全模式
Z切换排序
X切换顺序
S搜索