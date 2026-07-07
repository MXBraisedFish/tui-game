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
    "icon":  {
		"type": "image" | "text",
		"path": "path" -> 相对于包的assets/路径
    }
    "banner": {
		"type": "image" | "text",
		"path": "path" -> 相对于包的assets/路径
    }
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
	"mouse": false, - 是否需要鼠标操作：布尔值-表示该包需要鼠标操作
  }
}

  

完整表
████████ [调试]包名      █
████████ 作者：          █
████████ 版本：          █
████████ 状态：启用/禁用 █

简洁表
▌[*_pack.list.debug]包名           [*_pack.list.status.*] █
[*_pack.list.status.*]和最后的█之间要留一个空格，绘制时先计算[*_pack.list.status.*] █占据的位置（要动态计算，一些语言的*_pack.list.status.*的on和off不等长，然后[*_pack.list.debug]包名两者拼接占据剩余区域，超出使用...表示未展示完
▌仍是聚焦选项
简介表之间没有间隙，并且不垂直居中，直接向上顶到头
文本颜色必须参考详细表
然后接入L键切换

信息展示
banner图，按照36列*9行展示
(空一行，注意：banner要居中展示)
                                  包标题
  (空一行，注意：包标题要居中展示，以下内容居左展示)
*_pack.info.subtitle.base（黄色文字）
*_pack.info.pack_name（亮蓝色文字）[后面接包名而非包标题]
*_pack.info.author*（亮蓝色文字）[后面接包作者]
*_pack.info.version*（亮蓝色文字）[后面接包版本]
  (空一行）
*_pack.info.subtitle.config（黄色文字）
*_pack.info.status（亮蓝色文字）*_pack.info.status.*（on使用亮绿色，off使用亮红色）
*_pack.info.debug（亮蓝色文字）*_pack.info.debug.*（on使用亮玫红，off使用灰色[hint同色，下面灰色均为该颜色]）
*_pack.info.mouse（亮蓝色文字）*_pack.info.mouse.*（on.support使用亮绿色，on.unsupport使用两红色，off使用灰色[hint同色]）
*_pack.info.write（亮蓝色文字，游戏包独有）game_pack.info.write.*（on使用亮红色，off使用灰色）
*_pack.info.safe_mode（亮蓝色文字，游戏包独有）game_pack.info.safe_mode.*（on使用灰色，off.*使用亮红色）
  (空一行）
*_pack.info.subtitle.description（黄色文字）
[后面接包简介]


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

safe_mode_warning.description （使用默认颜色即可）
(空一行)
safe_mode_warning.no （使用亮绿色文字）
safe_mode_warning.yes.temporary[时间safe_mode_warning.second] （使用亮红色文字）
safe_mode_warning.yes.permanent[时间safe_mode_warning.second]（使用亮红色文字）

safe_mode_warning.description除了主动换行以外，还需要将自动换行限制在两侧距离终端距离各16格（即总宽度-16）
然后这上面的所有内容均左对齐（包括换行）
safe_mode_warning.yes.*在一开始为灰色（也就是正常页面具有的hint灰色，注意这是自定义色），temporary为倒计时3秒，permanent为倒计时5秒，倒计时结束后变为指定的亮红色
操作：
使用那两个：1. 禁用action map入队。2. 开启原始按键流入队。
只有1对应临时关闭safe_mode
只有2对应永久关闭safe_mode
其它所有按键均指向取消
但是action_map还是要写
1
2
Esc
同时包含对应的鼠标事件
这个界面没有hint

你先阅读一下文本json，然后依旧和之前一样先写在那个临时的python文件里，我确保你明白我的布局

我已将中文的safe_mode_warning.description改为safe_mode_warning.description.one（按照这个版本）
然后应当是 总宽度-32，这是我的问题
然后这个倒计时属于拼接，用中文举例
[1]关闭（仅本次）[3秒]
此时 [1]关闭（仅本次） 为灰色，[3秒]种[]为原始颜色，而里面的3秒为亮红色
结束后
[1]关闭（仅本次）
此时 [1]关闭（仅本次） 变为亮红色，后面的计时消失不显示
对应的鼠标事件，
好的，可以开始依照现在的版本制作
优先级
终端尺寸警告 > 安全模式关闭警告 = 语言重加载进度条
=的含义为谁先入栈谁在上