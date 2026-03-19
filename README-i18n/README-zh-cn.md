![LOGO](./image/logo.png)

# 语言

**[English](../README.md)**

# 本项目是做什么的？

你有想过在终端里玩游戏吗？这个项目就是我突发奇想，经过数日爆肝后，做出了这个由Rust和Lua共同打造的TUI游戏合集！
在假装敲代码或者操作服务器的时候，悄摸摸的打开偷偷玩一把。
(摸鱼这块)
基本支持所有系统的终端：Windows，Linux，MacOS

> 最新正式版：<br />[![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600&label=Release&labelColor=cc8400&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

> 官方网页
> 开发中

## 目录

- [可玩的游戏](#可玩的游戏)
- [语言支持](#语言支持)
- [平台支持](#平台支持)
- [更多细节](#更多细节)
- [指令使用指南](#指令使用指南)
- [安装指南](#安装指南)
  - [Windows](#Windows)
  - [Linux](#Linux)
  - [MacOS](#MacOS)
- [界面截图](#界面截图)
- [支持本项目-点个星星！](#支持本项目-点个星星！)
- [未来计划](#未来计划)

## 可玩的游戏

- [2048](#2048)
- [二十一点](#二十一点)
- [颜色记忆游戏](#颜色记忆游戏)
- [点灯游戏](#点灯游戏)
- [走迷宫](#走迷宫)
- [记忆翻牌](#记忆翻牌)
- [扫雷](#扫雷)
- [吃豆人](#吃豆人)
- [石头剪刀布](#石头剪刀布)
- [空中射击](#空中射击)
- [数字华容道](#数字华容道)
- [贪吃蛇](#贪吃蛇)
- [纸牌接龙](#纸牌接龙)
- [数独](#数独)
- [俄罗斯方块](#俄罗斯方块)
- [井字棋](#井字棋)
- [24点](#24点)
- [Wordle](#wordle)

## 语言支持

- English
- 简体中文 (在设置中切换)

## 平台支持

- Windows
- Linux
- MacOS (仍需测试 bug)

## 更多细节

- 编译后的版本为**字节码**，不需要额外rust和lua编译器，下载即游玩
- 自带终端**响应式**，大小不够的终端会有尺寸提示
- 支持**自定义新增语言**，可在资源文件夹直接添加json语言文件
- 部分游戏支持**存档**功能，便于持续性游玩

## 指令使用指南

- 语法 `tg [参数]`
- 参数
  - (无参数) 启动游戏
  - -v/-V/-version 获取当前安装版本与线上最新版本
  - -h/-H/-help 获取指令使用说明
  - -p/-P/- Path  获取安装路径(包管理器安装会指向符号链接)

## 安装指南

### Windows

#### 压缩包
```text
新建 tui-game 文件夹

在 Releases 下载 tui-game-[version]-windows.zip

解压至 tui-game 文件夹

在 Path 环境中注册该目录

在终端中使用 tg 指令启动游戏
```

#### 包管理器

#### Scoop

```bash
下载安装引导文件 tui-game-[version]-windows.json

# 新建 tui-game 文件夹
mkdir tui-game
cd tui-game

将安装引导文件放在当前目录

# 运行指令，并按照引导进行安装
scoop install tui-game-[version]-windows.json

# 安装成功，启动程序
tg
```

#### winget

> 注意：该安装方式不支持指令参数！(只可使用 tg 指令)

```bash
下载安装引导文件 tui-game-[version]-windows.yaml

# 新建 tui-game 文件夹
mkdir tui-game
cd tui-game

将安装引导文件放在当前目录

# 运行指令，并按照引导进行安装
winget install --manifest .

# 安装成功，启动程序
tg
```

#### Chocolatey

```bash
下载安装引导文件
 - tui-game-[version]-windows.nupkg 
 - tui-game-[version]-windows.nuspec 

# 新建 tui-game 文件夹
mkdir tui-game
cd tui-game

将安装引导文件放在当前目录

# 运行指令，并按照引导进行安装
choco install tui-game -s . -f

# 安装成功，启动程序
tg
```

### Linux

#### 压缩包
```text
新建 tui-game 文件夹

在 Releases 下载 tui-game-[version]-linux.tar.gz

解压至 tui-game 文件夹

在 Path 环境中注册该目录

在终端中使用 tg 指令启动游戏
```

#### APT

```bash
在 Releases 下载 tui-game-[version]-linux.deb

# 运行指令，并按照引导进行安装
sudo apt install ./tui-game-[version]-linux.deb

# 安装成功，启动程序
tg
```

#### DNF

```bash
在 Releases 下载 tui-game-[version]-linux.rpm

# 运行指令，并按照引导进行安装
sudo dnf install ./tui-game-[version]-linux.rpm

# 安装成功，启动程序
tg
```

### MacOS

> 注意：MacOS 系统未经实机测试，如果遇到BUG请及时提交 Issue 反馈，十分感谢！

#### 压缩包
```text
新建 tui-game 文件夹

在 Releases 下载 tui-game-[version]-macos.tar.gz 

解压至 tui-game 文件夹

在 Path 环境中注册该目录

在终端中使用 tg 指令启动游戏
```

#### Homebrew

```bash
# 下载安装引导文件
在 Releases 下载 tui-game-[version]-macos.rb

# 新建 tui-game 文件夹
mkdir tui-game
cd tui-game

将安装引导文件放在当前目录

# 运行指令，并按照引导进行安装
brew install tui-game.rb

# 安装成功，启动程序
tg
```

## 界面截图

### 主页和游戏列表

![主页](./image/main-page-zh-cn.png)
![游戏列表](./image/game-list-zh-cn.png)

### 2048

![2048](./image/2048-zh-cn.png)

### 二十一点

![二十一点](./image/blackjack-zh-cn.png)

### 颜色记忆游戏

![颜色记忆游戏](./image/colormemory-zh-cn.png)

### 点灯游戏

![点灯游戏](./image/lightout-zh-cn.png)

### 走迷宫

![走迷宫](./image/mazeescape-zh-cn.png)

### 记忆翻牌

![记忆翻牌](./image/memoryflip-zh-cn.png)

### 扫雷

![扫雷](./image/minesweeper-zh-cn.png)

### 吃豆人
![吃豆人](./image/pacman-zh-cn.png)

### 石头剪刀布

![石头剪刀布](./image/rockpaperscissors-zh-cn.png)

### 空中射击

![空中射击](./image/airshooter-zh-cn.png)

### 数字华容道

![数字华容道](./image/numberslidingpuzzle-zh-cn.png)

### 贪吃蛇

![贪吃蛇](./image/snake-zh-cn.png)

### 纸牌接龙

![空当接龙](./image/freecell-zh-cn.png)
![Klondike](./image/klondike-zh-cn.png)
![蜘蛛纸牌](./image/spider-zh-cn.png)

### 数独

![数独](./image/sudoku-zh-cn.png)

### 俄罗斯方块

![俄罗斯方块](./image/tetris-zh-cn.png)

### 井字棋

![井字棋](./image/tic-tac-toe-zh-cn.png)

### 24点

![24点](./image/24-points-zh-cn.png)

### Wordle

![Wordle](./image/wordle-zh-cn.png)

## 支持本项目-点个星星！

如果您喜欢这个项目，请为我的仓库点一颗星星⭐！这也是我持续更新的动力。如果您有更好的想法或建议，欢迎提出 Issue。

MacOS版本未经过测试，我没有相关的系统设备，如果您发现有Bug请及时反馈，十分感谢！

GitHub: [MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)

## 未来计划
> 画大饼咯

### 游戏计划
1. **轮盘赌** - 包含经典的俄罗斯轮盘赌和有名的恶魔轮盘赌游戏
2. **破译密码** - 包含多种经典密码破译
3. **抽盲盒** - 更纯粹的拼运气游戏
4. **公路赛车** - 在公路上飙车
5. **地牢探险** - Roguelike地牢冒险
6. **猜数字** - 在规定的对局内猜出数字
7. **调色师** - 根据参考色尽可能地调出相近的颜色
8. **Hitori(数阵去重)** - 按照规则将重复的数字涂黑
9. **Nonogram(数织)** - 按照提示涂出形状
10. **接水管** - 让水流可以从起点流向重点
11. **Breakout(打砖块)** - 反弹小球清空所有方块
12. **绘图填色** - 在给定的图片里添上对应的颜色

### 额外功能
1. **模组制作** - 用户自己制作额外的游戏(目前已经完成了基础接口)
2. **防老板功能** - 一键打开伪装功能界面，有效防止被老板发现摸鱼(而且会支持各个计算机行业：运维、前端、后端等等都会有，可设置)
3. **自定义快捷键** - 可以修改游戏中的按键设置来适配自己的操作