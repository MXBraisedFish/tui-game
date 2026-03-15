![LOGO](./README-i18n/image/logo.png)

# Language

**[English](./README-i18n/README-zh-cn.md)**

# What is this project?

Have you ever thought about playing games in the terminal?
This project started as a sudden idea, and after several days of intense coding, I created this TUI game collection built with **Rust and Lua**!

You can secretly open it and play a round while pretending to write code or operate servers.
(Perfect for a little sneaky break.)

Basically supports terminals on all systems: **Windows, Linux, MacOS**

> Latest release:
> [![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600\&label=Release\&labelColor=cc8400\&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

> Official website
> In development

## Table of Contents

* [Playable Games](#playable-games)
* [Language Support](#language-support)
* [Platform Support](#platform-support)
* [More Details](#more-details)
* [Installation Guide](#installation-guide)
  * [Windows](#windows)
  * [Linux](#linux)
  * [MacOS](#macos)
* [Screenshots](#screenshots)
* [Support the Project - Give it a Star!](#support-the-project---give-it-a-star)
* [Future Plans](#future-plans)

## Playable Games

* [2048](#2048)
* [Blackjack](#blackjack)
* [Color Memory Game](#color-memory-game)
* [Lights Out](#lights-out)
* [Maze Escape](#maze-escape)
* [Memory Flip](#memory-flip)
* [Minesweeper](#minesweeper)
* [Pac-Man](#pac-man)
* [Rock Paper Scissors](#rock-paper-scissors)
* [Air Shooter](#air-shooter)
* [Number Sliding Puzzle](#number-sliding-puzzle)
* [Snake](#snake)
* [Solitaire](#solitaire)
* [Sudoku](#sudoku)
* [Tetris](#tetris)
* [Tic-Tac-Toe](#tic-tac-toe)
* [24 Points](#24-points)
* [Wordle](#wordle)

## Language Support

* English
* Simplified Chinese

## Platform Support

* Windows
* Linux
* MacOS (still needs bug testing)

## More Details

* The compiled version is **bytecode**, no Rust or Lua compiler required, download and play directly
* Built-in **terminal responsive design**, terminals that are too small will show size warnings
* Supports **adding new languages**, you can directly add JSON language files in the resource folder
Some games support a **save feature**, allowing you to continue playing later.

## Installation Guide

### Windows

#### - Terminal Script Installation (Recommended)

> Includes all automatic services (compiled, auto update, quick uninstall, automatic environment variable registration)

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Download installation script
# Official source
curl -L -o windows-tui-game-init.bat https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/windows-tui-game-init.bat
# Mirror source
curl -L -o windows-tui-game-init.bat https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/windows-tui-game-init.bat

# Run installation script
windows-tui-game-init.bat
```

#### - Download ZIP Package

> Includes some automatic services (compiled, auto update, quick uninstall, no automatic environment variable registration)

```text
Go to Releases:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

Download tui-game-windows.zip

Extract tui-game-windows.zip

Run tg.bat
```

#### - Source Code

> Source code version

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Clone source code
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# Run debug
cargo run

# Build release
cargo build --release
```

### Linux

#### - Terminal Script Installation (Recommended)

> Includes all automatic services (compiled, auto update, quick uninstall, automatic environment variable registration)

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Download installation script
# Official source
curl -L -o linux-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/linux-tui-game-init.sh
# Mirror source
curl -L -o linux-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/linux-tui-game-init.sh

# Run installation script
sh linux-tui-game-init.sh
```

#### - Download Package

> Includes some automatic services (compiled, auto update, quick uninstall, no automatic environment variable registration)

```text
Go to Releases:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

Download tui-game-linux.tar.gz

Extract tui-game-linux.tar.gz

Run tui-game.sh
```

#### - Source Code

> Source code version, no automatic services

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Clone source code
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# Run debug
cargo run

# Build release
cargo build --release
```

### MacOS (still needs bug testing)

#### - Terminal Script Installation (Recommended)

> Includes all automatic services (compiled, auto update, quick uninstall, automatic environment variable registration)

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Download installation script
# Official source
curl -L -o macos-tui-game-init.sh https://raw.githubusercontent.com/MXBraisedFish/TUI-GAME/main/macos-tui-game-init.sh
# Mirror source
curl -L -o macos-tui-game-init.sh https://fastly.jsdelivr.net/gh/MXBraisedFish/TUI-GAME@main/macos-tui-game-init.sh

# Run installation script
sh macos-tui-game-init.sh
```

#### - Download Compiled Version

> No quick uninstall program, no automatic update program

```text
Go to Releases:
https://github.com/MXBraisedFish/TUI-GAME/releases/latest

Download tui-game-macos.zip

Extract tui-game-macos.zip

Run tui-game.sh
```

#### - Source Code

> Source code version, no automatic services

```Shell
# Create folder
mkdir tui-game

# Enter folder
cd tui-game

# Clone source code
git clone https://github.com/MXBraisedFish/TUI-GAME.git

# Run debug
cargo run

# Build release
cargo build --release
```

## Screenshots

### Home Page & Game List

![Home](./README-i18n/image/main-page.png)
![Game List](./README-i18n/image/game-list.png)

### 2048

![2048](./README-i18n/image/2048.png)

### Blackjack

![Blackjack](./README-i18n/image/blackjack.png)

### Color Memory Game

![Color Memory](./README-i18n/image/colormemory.png)

### Lights Out

![Lights Out](./README-i18n/image/lightout.png)

### Maze Escape

![Maze Escape](./README-i18n/image/mazeescape.png)

### Memory Flip

![Memory Flip](./README-i18n/image/memoryflip.png)

### Minesweeper

![Minesweeper](./README-i18n/image/minesweeper.png)

### Pac-Man

![Pacman](./README-i18n/image/pacman.png)

### Rock Paper Scissors

![RPS](./README-i18n/image/rockpaperscissors.png)

### Air Shooter

![Air Shooter](./README-i18n/image/airshooter.png)

### Number Sliding Puzzle

![Sliding Puzzle](./README-i18n/image/numberslidingpuzzle.png)

### Snake

![Snake](./README-i18n/image/snake.png)

### Solitaire

![Solitaire](./README-i18n/image/freecell.png)
![Solitaire](./README-i18n/image/klondike.png)
![Solitaire](./README-i18n/image/spider.png)

### Sudoku

![Sudoku](./README-i18n/image/sudoku.png)

### Tetris

![Tetris](./README-i18n/image/tetris.png)

### Tic-Tac-Toe

![TicTacToe](./README-i18n/image/tic-tac-toe.png)

### 24 Points

![24 Points](./README-i18n/image/24-points.png)

### Wordle

![Wordle](./README-i18n/image/wordle.png)

## Support the Project - Give it a Star!

If you like this project, please give my repository a star ⭐!
It is also the motivation for me to keep updating.
If you have better ideas or suggestions, feel free to open an Issue.

The MacOS version has not been tested because I do not have the device.
If you find any bugs, please report them. Thank you very much!

GitHub Repo:
[MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)

## Future Plans

> Big plans ahead

### Game Plans

1. Roulette - Includes classic Russian roulette and the famous devil roulette
2. Cipher Breaking - Includes multiple classic cipher cracking games
3. Blind Box - A pure luck-based game
4. Road Racing - Speed on the highway
5. Dungeon Adventure - Roguelike dungeon exploration
6. Guess the Number - Guess the number within limited attempts
7. Color Matching - Mix colors to match a reference color as closely as possible
8. Hitori - Eliminate duplicate numbers according to rules
9. Nonogram - Fill grids according to hints to reveal shapes
10. Pipes - Connect pipes so water flows from start to end
11. Breakout - Bounce the ball to clear all bricks
12. Coloring - Fill the given picture with correct colors

### Extra Features

1. Modding Support - Users can create their own games (API already completed)
2. Anti-Boss Mode - One-click disguise interface to prevent your boss from discovering you playing (will support multiple computer professions such as DevOps, Frontend, Backend, etc., customizable)

