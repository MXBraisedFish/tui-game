![LOGO](./image/logo.png)

# 语言

**[English](../README.md)**

# 本项目是做什么的？

你有想过在终端里玩游戏吗？这个项目就是我突发奇想，经过数日爆肝后，做出了这个由Rust和Lua共同打造的TUI游戏合集！
在假装敲代码或者操作服务器的时候，悄摸摸的打开偷偷玩一把。
(摸鱼这块)
基本支持所有系统的终端：Windows，Linux，MacOS

> 最新正式版：  
> [![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600&label=Release&labelColor=cc8400&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

> 官方网页
> 开发中

## 目录

- [可玩的游戏](#可玩的游戏)
- [语言支持](#语言支持)
- [平台支持](#平台支持)
- [更多细节](#更多细节)
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
- 简体中文

## 平台支持

- Windows
- Linux
- MacOS (仍需测试 bug)

## 更多细节

- 编译后的版本为**字节码**，不需要额外rust和lua编译器，下载即游玩
- 自带终端**响应式**，大小不够的终端会有尺寸提示
- 支持**自定义新增语言**，可在资源文件夹直接添加json语言文件
- 部分游戏支持**存档**功能，便于持续性游玩

## 安装指南

### Windows

#### - 终端脚本安装(推荐)

> 包含所有自动服务(已编译，自动更新，快捷卸载，自动注册环境变量)

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取安装脚本
# 官方源
curl -L -o windows-tui-game-init.bat https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/windows-tui-game-init.bat
# 镜像源
curl -L -o windows-tui-game-init.bat https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/windows-tui-game-init.bat

# 运行安装脚本
windows-tui-game-init.bat
```

#### - 下载压缩包

> 包含部分自动服务(已编译，自动更新，快捷卸载，无自动注册环境变量)

```text
进入Releases界面:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

下载压缩包 tui-game-windows.zip

解压 tui-game-windows.zip

运行 tg.bat 脚本
```

#### - 源代码

> 源代码版本

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取源代码
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# 运行调试
cargo run

# 构建编译
cargo build --release
```

### Linux

#### - 终端脚本安装(推荐)

> 包含所有自动服务(已编译，自动更新，快捷卸载，自动注册环境变量)

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取安装脚本
# 官方源
curl -L -o linux-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/linux-tui-game-init.sh
# 镜像源
curl -L -o linux-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/linux-tui-game-init.sh

# 运行安装脚本
sh linux-tui-game-init.sh
```

#### - 下载压缩包

> 包含部分自动服务(已编译，自动更新，快捷卸载，无自动注册环境变量)

```text
进入Releases界面:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

下载压缩包 tui-game-linux.tar.gz

解压 tui-game-linux.tar.gz

运行 tui-game.sh 脚本
```

#### - 源代码

> 源代码版本，无任何自动服务

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取源代码
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# 运行调试
cargo run

# 构建编译
cargo build --release
```

### MacOS (仍需测试 bug)

#### - 终端脚本安装(推荐)

> 包含所有自动服务(已编译，自动更新，快捷卸载，自动注册环境变量)

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取安装脚本
# 官方源
curl -L -o macos-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/macos-tui-game-init.sh
# 镜像源
curl -L -o macos-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/macos-tui-game-init.sh

# 运行安装脚本
sh macos-tui-game-init.sh
```

#### - 下载编译版本

> 无快捷卸载程序，无自动更新程序

```text
进入Releases界面:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

下载压缩包 tui-game-macos.zip

解压 tui-game-macos.zip

运行 tui-game.sh 脚本
```

#### - 源代码

> 源代码版本，无任何自动服务

```Shell
# 新建文件夹
mkdir tui-game

# 进入文件夹
cd tui-game

# 拉取源代码
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# 运行调试
cargo run

# 构建编译
cargo build --release
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

GitHub Repo: [MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)

## 未来计划
> 画大饼咯

### 游戏计划
1. 轮盘赌 - 包含经典的俄罗斯轮盘赌和有名的恶魔轮盘赌游戏
2. 破译密码 - 包含多种经典密码破译
3. 抽盲盒 - 更纯粹的拼运气游戏
4. 公路赛车 - 在公路上飙车
5. 地牢探险 - Roguelike地牢冒险
6. 猜数字 - 在规定的对举内猜出数字
7. 配颜色 - 根据参考色尽可能地调出相近地颜色
8. Hitori(数阵去重) - 按照规则将重复的数字涂黑
9. Nonogram(数织) - 按照提示涂出形状
10. 接水管 - 让水流可以从起点流向重点
11. Breakout(打砖块) - 反弹小球清空所有方块
12. 绘图填色 - 在给定的图片里添上对应的颜色

### 额外功能
1. 模组制作 - 用户自己制作额外的游戏(目前已经完成了接口)
2. 防老板功能 - 一键打开伪装功能界面，有效防止被老板发现摸鱼(而且会支持各个计算机行业：运维、前端、后端等等都会有，可设置)

数独 -> 切换难度没有数字提示