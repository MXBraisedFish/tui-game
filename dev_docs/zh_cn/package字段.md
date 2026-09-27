# 游戏包

```jsonc
{
  "mod_id": "id",                           // 模组唯一ID，由用户自行定义
  "schema_version": 1,                      // 配置版本，必须等于当前宿主要求
  "type": "game",                           // 包类型：game / screensaver
  "version": {                              // 展示版本号
		"type": "i18n",                         // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
		"key": "title",                         // i18n 时必填：匹配键
		"callback": "Title",                    // i18n 时必填：回退值

		"text": "Title"                         // text 时必填：直接文本（i18n 时忽略）
	},                       
  "version_code": 1,                        // 版本真值，正整数，必须递增（当前版本无社区，仅作为保留字段，实际上并不影响包加载）
  "api": {
    "min": 1,                               // 最小版本，必须 ≤ max
    "max": 2                                 // 最大版本，必须 ≥ min
  },

  "entry": "init.lua",
}

```

```
// ============================================================
  //  显示信息
  // ============================================================
  "display": {
    // ----- 标题 -----
    "title": {
      "type": "i18n",                       // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
      "path": "display.json",               // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
      "key": "title",                       // i18n 时必填：匹配键
      "callback": "Title",                  // i18n 时必填：回退值
      "text": "Title"                       // text 时必填：直接文本（i18n 时忽略）
    },
    // ----- 简介 -----
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "Description",
      "text": "Description"
    },
    // ----- 作者 -----
    "author": {
      "type": "i18n",
      "path": "display.json",
      "key": "author",
      "callback": "Author",
      "text": "Author"
    },
    // ----- 图标（选填，默认使用宿主内置）-----
    "icon": {
      "type": "image",                      // image 或 text
      "path": "path"                        // 相对于包的 assets/ 路径
    },
    // ----- 横幅（选填，默认使用宿主内置）-----
    "banner": {
      "type": "image",                      // image 或 text
      "path": "path"                        // 相对于包的 assets/ 路径
    }
  },

  // ============================================================
  //  运行配置
  // ============================================================
  "runtime": {
  },

  // ============================================================
  //  游戏包专属配置，type为game时必填
  // ============================================================
  "game": {
    "min_width": 60,                        // 最小宽度，0 表示不限制
    "min_height": 20,                        // 最小高度，0 表示不限制
    // ----- 游戏名称 -----
    "name": {
      "type": "i18n",
      "path": "display.json",
      "key": "game_name",
      "callback": "Minefield",
      "text": "Minefield"
    },
    // ----- 游戏详情 -----
    "detail": {
      "type": "i18n",
      "path": "display.json",
      "key": "game_detail",
      "callback": "A full-featured minesweeper...",
      "text": "A full-featured minesweeper..."
    },

    // ----- 功能开关 -----
    "truecolor": false,                     // 【选填，默认 false】是否需要真彩支持
    "mouse": false,                         // 【选填，默认 false】是否需要鼠标操作
    "target_fps": 30,                       // 【选填，默认 60】目标帧率：30/60/120
    "save": true,                           // 【选填，默认 false】是否支持存档
    
    // ----- 支持语言 -----
    "language": [zh_cn, en_us, ...]         // 填写语言代码，用于表示支持哪些语言

    // ----- 最佳纪录配置（选填）-----
    "score": { 
      "enabled": true,                      // 【选填，默认 false】是否启用最佳纪录
      "empty_text": {                       // 【选填，默认程序内置】无纪录时显示的占位文本
	      "type": "i18n",
	      "path": "display.json",
	      "key": "description",
	      "callback": "Description",
	      "text": "Description"
	    },       
    },

    // ----- 按键注册（动作可任意扩展）-----
    "actions": {
      "move_up": {                          // 动作语义名（可自定义）
        "description": {                    // 动作描述（用于按键修改界面展示）
          "type": "i18n",
          "path": "display.json",
          "key": "action_move_up",
          "callback": "Move cursor up",
          "text": "Move cursor up"
        },
        "keys": [                           // 键位数组（数组至多 2 个元素，多出来的取前两个）
          ["w"],                            // 主键（数组至多 2 个元素，多出来的取前两个）
          ["up"]                            // 备选键（数组至多 2 个元素，多出来的取前两个）
        ]
      }
      // 可继续添加 ...
    }
  }
}
```

# 屏保包

```jsonc
{
  // ============================================================
  //  必填基础信息
  // ============================================================
  "mod_id": "id",                           // 模组唯一ID，由用户自行定义
  "schema_version": 1,                      // 配置版本，必须等于当前宿主要求
  "type": "screensaver",                    // 包类型：game / screensaver
  "version": {                              // 展示版本号
    "type": "i18n",                         // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
    "path": "display.json",                 // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
    "key": "version",                       // i18n 时必填：匹配键
    "callback": "1.0.0",                    // i18n 时必填：回退值
    "text": "1.0.0"                         // text 时必填：直接文本（i18n 时忽略）
  },
  "version_code": 1,                        // 版本真值，正整数，必须递增（当前版本无社区，仅作为保留字段，实际上并不影响包加载）

  // ============================================================
  //  API 版本支持（区间闭合）
  // ============================================================
  "api": {
    "min": 1,                               // 最小版本，必须 ≤ max
    "max": 2                                 // 最大版本，必须 ≥ min
  },

  // ============================================================
  //  入口脚本（相对于包 scripts/ 目录）
  // ============================================================
  "entry": "init.lua",

  // ============================================================
  //  显示信息
  // ============================================================
  "display": {
    // ----- 标题 -----
    "title": {
      "type": "i18n",                       // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
      "path": "display.json",               // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
      "key": "title",                       // i18n 时必填：匹配键
      "callback": "My Screensaver Pack",    // i18n 时必填：回退值
      "text": "My Screensaver Pack"         // text 时必填：直接文本（i18n 时忽略）
    },
    // ----- 简介 -----
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "A collection of classic terminal screensavers.",
      "text": "A collection of classic terminal screensavers."
    },
    // ----- 作者 -----
    "author": {
      "type": "i18n",
      "path": "display.json",
      "key": "author",
      "callback": "Alex",
      "text": "Alex"
    },
    // ----- 图标（选填，默认使用宿主内置）-----
    "icon": {
      "type": "image",                      // image 或 text
      "path": "pack_icon.png"               // 相对于包的 assets/ 路径
    },
    // ----- 横幅（选填，默认使用宿主内置）-----
    "banner": {
      "type": "image",                      // image 或 text
      "path": "pack_banner.png"             // 相对于包的 assets/ 路径
    }
  },

  // ============================================================
  //  运行配置
  // ============================================================
  "runtime": {
    "min_width": 60,                        // 最小宽度，0 表示不限制
    "min_height": 20                        // 最小高度，0 表示不限制
  },

  // ============================================================
  //  屏保包专属配置，type 为 screensaver 时必填
  // ============================================================
  "screensaver": {
    // ----- 屏保名称 -----
    "name": {
      "type": "i18n",
      "path": "display.json",
      "key": "screensaver_name",
      "callback": "Minefield",
      "text": "Minefield"
    },
    // ----- 快捷指令（用于快捷启动）-----
    "command": "mind",                      // 【必填】快捷指令：字符串

    // ----- 功能开关 -----
    "truecolor": false,                     // 【选填，默认 false】是否需要真彩支持
  }
}
```

安全详细里的markdown没有及时的转换语言，说明markdown对象创建需要优化。
若原本游戏已经存储了继续游戏或最佳记录的字段，但是游戏包更新后不再支持这两个字段如何处理这个边界条件需要修复：我的建议是扫描后删除。
几个代办的记录：
1.完善继续游戏，在继续游戏后传递真正的继续游戏上下文表，并完善 ui 界面的继续游戏，有继续游戏时，为白色，后接目前保存的游戏的名字，否则为灰色且不可聚焦不可选中不可确认；所有新游戏都会导致旧的继续游戏被清理，无论该游戏是否支持保存继续。
2. 添加初始的加载界面，以提醒玩家启动器正在初始化。
3. 游戏包添加新的字段language，里面填写语言代码数组，然后i18n 添加对应的语言警告，提醒玩家是否支持当前偏好语言，两个包的将i18n 字段改为text，i18n显式声明，包含文件，字段，回退。
4. 添加快捷参数指令tg -x相关，目前有查询版本与更新，快速在当前目录创建一个游戏包或屏保包结构，快捷启动屏保，查询安装位置，查询占用存储，初始化data目录（完全清理，二次确认），导出全部数据（可指定路径）。
5. 完善错误捕获，我们不必那么完善，但是必须保证每个服务都可以在出现错误后，被捕获，先走报错路径，除非我们控制不了再交给panic hook。
6.游戏时长记录，手柄支持，成就系统。

---

接下来我发现了很多个问题，来一一优化：
1. 我们完全重写包的检测部分，字段整合更新，尤其是针对文本展示的部分，使用更明确的对象结构来表达是纯文本还是i18n所需，彻底明确哪些是可选字段，可不写，宿主会默认对齐，哪些是必填字段，只要不写或不合法，就会被拒绝扫描（仅这个包，不会影响其他的），并在系统（宿主）的日志下写入问题所在（为什么被拒绝扫描），具体如下：
# 游戏包

```jsonc
{
  // ============================================================
  //  必填基础信息
  // ============================================================
  "mod_id": "id",                           // 模组唯一ID，由用户自行定义
  "schema_version": 1,                      // 配置版本，必须等于当前宿主要求
  "type": "game",                           // 包类型：game / screensaver
  "version": {                              // 展示版本号
		"type": "i18n",                         // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
		"path": "display.json",                 // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
		"key": "title",                         // i18n 时必填：匹配键
		"callback": "Title",                    // i18n 时必填：回退值
		"text": "Title"                         // text 时必填：直接文本（i18n 时忽略）
	},                       
  "version_code": 1,                        // 版本真值，正整数，必须递增（当前版本无社区，仅作为保留字段，实际上并不影响包加载）

  // ============================================================
  //  API 版本支持（区间闭合）
  // ============================================================
  "api": {
    "min": 1,                               // 最小版本，必须 ≤ max
    "max": 2                                 // 最大版本，必须 ≥ min
  },

  // ============================================================
  //  入口脚本（相对于包 scripts/ 目录）
  // ============================================================
  "entry": "init.lua",

  // ============================================================
  //  显示信息
  // ============================================================
  "display": {
    // ----- 标题 -----
    "title": {
      "type": "i18n",                       // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
      "path": "display.json",               // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
      "key": "title",                       // i18n 时必填：匹配键
      "callback": "Title",                  // i18n 时必填：回退值
      "text": "Title"                       // text 时必填：直接文本（i18n 时忽略）
    },
    // ----- 简介 -----
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "Description",
      "text": "Description"
    },
    // ----- 作者 -----
    "author": {
      "type": "i18n",
      "path": "display.json",
      "key": "author",
      "callback": "Author",
      "text": "Author"
    },
    // ----- 图标（选填，默认使用宿主内置）-----
    "icon": {
      "type": "image",                      // image 或 text
      "path": "path"                        // 相对于包的 assets/ 路径
    },
    // ----- 横幅（选填，默认使用宿主内置）-----
    "banner": {
      "type": "image",                      // image 或 text
      "path": "path"                        // 相对于包的 assets/ 路径
    }
  },

  // ============================================================
  //  运行配置
  // ============================================================
  "runtime": {
    "min_width": 60,                        // 最小宽度，0 表示不限制
    "min_height": 20                        // 最小高度，0 表示不限制
  },

  // ============================================================
  //  游戏包专属配置，type为game时必填
  // ============================================================
  "game": {
    // ----- 游戏名称 -----
    "name": {
      "type": "i18n",
      "path": "display.json",
      "key": "game_name",
      "callback": "Minefield",
      "text": "Minefield"
    },
    // ----- 游戏详情 -----
    "detail": {
      "type": "i18n",
      "path": "display.json",
      "key": "game_detail",
      "callback": "A full-featured minesweeper...",
      "text": "A full-featured minesweeper..."
    },

    // ----- 功能开关 -----
    "truecolor": false,                     // 【选填，默认 false】是否需要真彩支持，仅作为提示，并不影响实际使用
    "mouse": false,                         // 【选填，默认 false】是否需要鼠标操作，仅作为提示，并不影响实际使用
    "target_fps": 30,                       // 【选填，默认 60】目标帧率：30/60/120
    "save": true,                           // 【选填，默认 false】是否支持存档
    
    // ----- 支持语言 -----
    "language": [zh_cn, en_us, ...]         // 填写语言代码，用于表示支持哪些语言，仅作为提示，并不影响实际使用

    // ----- 最佳纪录配置（选填）-----
    "score": { 
      "enabled": true,                      // 【选填，默认 false】是否启用最佳纪录
      "empty_text": {                       // 【选填，默认程序内置】无纪录时显示的占位文本
	      "type": "i18n",
	      "path": "display.json",
	      "key": "description",
	      "callback": "Description",
	      "text": "Description"
	    },       
    },

    // ----- 按键注册（动作可任意扩展）-----
    "actions": {
      "move_up": {                          // 动作语义名（可自定义）
        "description": {                    // 动作描述（用于按键修改界面展示）
          "type": "i18n",
          "path": "display.json",
          "key": "action_move_up",
          "callback": "Move cursor up",
          "text": "Move cursor up"
        },
        "keys": [                           // 键位数组（数组至多 2 个元素，多出来的取前两个）
          ["w"],                            // 主键（数组至多 2 个元素，多出来的取前两个）
          ["up"]                            // 备选键（数组至多 2 个元素，多出来的取前两个）
        ]
      }
      // 可继续添加 ...
    }
  }
}
```

# 屏保包

```jsonc
{
  // ============================================================
  //  必填基础信息
  // ============================================================
  "mod_id": "id",                           // 模组唯一ID，由用户自行定义
  "schema_version": 1,                      // 配置版本，必须等于当前宿主要求
  "type": "screensaver",                    // 包类型：game / screensaver
  "version": {                              // 展示版本号
    "type": "i18n",                         // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
    "path": "display.json",                 // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
    "key": "version",                       // i18n 时必填：匹配键
    "callback": "1.0.0",                    // i18n 时必填：回退值
    "text": "1.0.0"                         // text 时必填：直接文本（i18n 时忽略）
  },
  "version_code": 1,                        // 版本真值，正整数，必须递增（当前版本无社区，仅作为保留字段，实际上并不影响包加载）

  // ============================================================
  //  API 版本支持（区间闭合）
  // ============================================================
  "api": {
    "min": 1,                               // 最小版本，必须 ≤ max
    "max": 2                                 // 最大版本，必须 ≥ min
  },

  // ============================================================
  //  入口脚本（相对于包 scripts/ 目录）
  // ============================================================
  "entry": "init.lua",

  // ============================================================
  //  显示信息
  // ============================================================
  "display": {
    // ----- 标题 -----
    "title": {
      "type": "i18n",                       // i18n 或 text，也可以直接传递一个字符串，"..."等价于{"type": "text", "text": "..."}
      "path": "display.json",               // i18n 时必填：相对于 assets/language/[language_code]/ 的路径
      "key": "title",                       // i18n 时必填：匹配键
      "callback": "My Screensaver Pack",    // i18n 时必填：回退值
      "text": "My Screensaver Pack"         // text 时必填：直接文本（i18n 时忽略）
    },
    // ----- 简介 -----
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "A collection of classic terminal screensavers.",
      "text": "A collection of classic terminal screensavers."
    },
    // ----- 作者 -----
    "author": {
      "type": "i18n",
      "path": "display.json",
      "key": "author",
      "callback": "Alex",
      "text": "Alex"
    },
    // ----- 图标（选填，默认使用宿主内置）-----
    "icon": {
      "type": "image",                      // image 或 text
      "path": "pack_icon.png"               // 相对于包的 assets/ 路径
    },
    // ----- 横幅（选填，默认使用宿主内置）-----
    "banner": {
      "type": "image",                      // image 或 text
      "path": "pack_banner.png"             // 相对于包的 assets/ 路径
    }
  },

  // ============================================================
  //  运行配置
  // ============================================================
  "runtime": {
    "min_width": 60,                        // 最小宽度，0 表示不限制
    "min_height": 20                        // 最小高度，0 表示不限制
  },

  // ============================================================
  //  屏保包专属配置，type 为 screensaver 时必填
  // ============================================================
  "screensaver": {
    // ----- 屏保名称 -----
    "name": {
      "type": "i18n",
      "path": "display.json",
      "key": "screensaver_name",
      "callback": "Minefield",
      "text": "Minefield"
    },
    // ----- 快捷指令（用于快捷启动）-----
    "command": "mind",                      // 【必填】快捷指令：字符串

    // ----- 功能开关 -----
    "truecolor": false,                     // 【选填，默认 false】是否需要真彩支持，仅作为提示，并不影响实际使用
  }
}
```
2. 优化完字段后，注意到游戏添加了一个新的字段 `game.language` ，传递一个数组，里面填写语言代码字符串（注意，是语言代码），然后系统会根据当前玩家选择语言做出提醒，这个字段实际上只作为一个提示，不支持玩家的语言也不会影响游戏的正常启动。
	提示字段位于game_list.json的game_list.info.language.error，其展示位置位于如第一张图所示的位置，然后在模组包列表的右侧信息部分，在配置信息额外添加一行语言支持，使用game_pack.json中的game_pack.info.language，和game_pack.info.language.not_support（红色）、game_pack.info.language.support（绿色），遍历数组后得出，其展示如第二张图所示。
3. 正式加入继续游戏逻辑，首先优化首页的”继续游戏“这个选项，如果当前继续游戏槽位没有数据，则该选项为灰色，且不可选中、交互、聚焦；若当前存在一个游戏，则会展示”继续游戏-游戏名“，游戏名限制最多6个字符宽度，超出显示...，会自动锚定相对应的游戏名字（注意i18n扫描），然后继续游戏，现在哪怕是继续游戏可以启动脚本，传递的依旧是new而且没有continue_data数据，正式加入相关ctx和继续游戏逻辑
4. 添加新的覆盖屏：覆盖存档提示。触发条件：当前继续游戏有存档数据，但是用户开启了新游戏，在正式进入游戏前提示。前提条件：我们的继续游戏数据会被新游戏无条件删除和覆盖，哪怕是当前游戏没有存档功能，也会直接删除继续游戏的数据，直接置空即可。
	布局：上title，中间提示和操作，操作：Enter开始，Esc返回。
	18n：cover_continue.json。相关颜色具体布局参考其他的警告页面即可。
5. 如第三张和第四张图所示，当前右侧详细信息的滑动框会在下面的操作提示两行时显示出超出的内容，说明滑动框的高度没有随着实际占用高度变化，修复一下，该bug包含游戏包和屏保包两个界面。
