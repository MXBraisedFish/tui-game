![LOGO](./README-i18n/image/logo.png)

# Language

**[中文](./README-i18n/README-zh-cn.md)**

# What is this project about?

Have you ever thought about playing games in the terminal? This project came to me as a sudden idea, and after days of intense development, I created this TUI game collection built with Rust and Lua!  
You can secretly play a game while pretending to code or operate a server.  
(Perfect for sneaky breaks)  
Basically supports all system terminals: Windows, Linux, MacOS

> Latest stable version:<br />[![Release](https://img.shields.io/github/v/release/MXBraisedFish/TUI-GAME?maxAge=3600&label=Release&labelColor=cc8400&color=ffa500)](https://github.com/MXBraisedFish/TUI-GAME/releases/latest)

> Official website<br />[TUI GAME](https://tui-game.vercel.app/)

## Table of Contents

- [Playable Games](#playable-games)
- [Language Support](#language-support)
- [Platform Support](#platform-support)
- [More Details](#more-details)
- [Command Usage Guide](#command-usage-guide)
- [Installation Guide](#installation-guide)
  - [Windows](#windows)
  - [Linux](#linux)
  - [MacOS](#macos)
- [Screenshots](#screenshots)
- [Support This Project - Give a Star!](#support-this-project---give-a-star)
- [Future Plans](#future-plans)

## Playable Games

- [2048](#2048)
- [Blackjack](#blackjack)
- [Color Memory Game](#color-memory-game)
- [Lights Out](#lights-out)
- [Maze Escape](#maze-escape)
- [Memory Flip](#memory-flip)
- [Minesweeper](#minesweeper)
- [Pac-Man](#pac-man)
- [Rock Paper Scissors](#rock-paper-scissors)
- [Air Shooter](#air-shooter)
- [Number Sliding Puzzle](#number-sliding-puzzle)
- [Snake](#snake)
- [Solitaire](#solitaire)
- [Sudoku](#sudoku)
- [Tetris](#tetris)
- [Tic-Tac-Toe](#tic-tac-toe)
- [24 Points](#24-points)
- [Wordle](#wordle)

## Language Support

- English
- 简体中文 (switch in settings)

## Platform Support

- Windows
- Linux
- MacOS (still needs testing for bugs)

## More Details

- Compiled version is **bytecode**, no need for additional Rust or Lua compilers — download and play
- Built-in terminal **responsiveness**, with size提示 if terminal is too small
- Supports **custom language additions**, just add JSON language files directly in the resource folder
- Some games support **save functionality** for continuous play

## Command Usage Guide

- Syntax: `tg [option]`
- Options
  - (no option) Launch the game
  - -v/-V/-version Get current installed version and latest online version
  - -h/-H/-help Get command usage instructions
  - -p/-P/-Path Get installation path (for package manager installs, points to symlink)

## Installation Guide

### Windows

#### Archive
```text
Create a tui-game folder

Download tui-game-[version]-windows.zip from Releases

Extract to the tui-game folder

Add the directory to your PATH environment variable

Use the tg command in the terminal to start the game
```

#### Package Manager

#### Scoop

```bash
Download the installation manifest tui-game-[version]-windows.json

# Create a tui-game folder
mkdir tui-game
cd tui-game

Place the installation manifest in the current directory

# Run the command and follow the prompts to install
scoop install tui-game-[version]-windows.json

# Installation successful, start the program
tg
```

#### winget

> Note: This installation method does not support command arguments! (Only the tg command can be used)

```bash
Download the installation manifest tui-game-[version]-windows.yaml

# Create a tui-game folder
mkdir tui-game
cd tui-game

Place the installation manifest in the current directory

# Run the command and follow the prompts to install
winget install --manifest .

# Installation successful, start the program
tg
```

#### Chocolatey

```bash
Download the installation manifests
 - tui-game-[version]-windows.nupkg
 - tui-game-[version]-windows.nuspec

# Create a tui-game folder
mkdir tui-game
cd tui-game

Place the installation manifests in the current directory

# Run the command and follow the prompts to install
choco install tui-game -s . -f

# Installation successful, start the program
tg
```

### Linux

#### Archive
```text
Create a tui-game folder

Download tui-game-[version]-linux.tar.gz from Releases

Extract to the tui-game folder

Add the directory to your PATH environment variable

Use the tg command in the terminal to start the game
```

#### APT

```bash
Download tui-game-[version]-linux.deb from Releases

# Run the command and follow the prompts to install
sudo apt install ./tui-game-[version]-linux.deb

# Installation successful, start the program
tg
```

#### DNF

```bash
Download tui-game-[version]-linux.rpm from Releases

# Run the command and follow the prompts to install
sudo dnf install ./tui-game-[version]-linux.rpm

# Installation successful, start the program
tg
```

### MacOS

> Note: MacOS has not been tested on actual hardware. If you encounter bugs, please submit an Issue. Thank you!

#### Archive
```text
Create a tui-game folder

Download tui-game-[version]-macos.tar.gz from Releases

Extract to the tui-game folder

Add the directory to your PATH environment variable

Use the tg command in the terminal to start the game
```

#### Homebrew

```bash
# Download the installation manifest
Download tui-game-[version]-macos.rb from Releases

# Create a tui-game folder
mkdir tui-game
cd tui-game

Place the installation manifest in the current directory

# Run the command and follow the prompts to install
brew install tui-game.rb

# Installation successful, start the program
tg
```

## Screenshots

### Main Page and Game List

![Main Page](./README-i18n/image/main-page.png)
![Game List](./README-i18n/image/game-list.png)

### 2048

![2048](./README-i18n/image/2048.png)

### Blackjack

![Blackjack](./README-i18n/image/blackjack.png)

### Color Memory Game

![Color Memory Game](./README-i18n/image/colormemory.png)

### Lights Out

![Lights Out](./README-i18n/image/lightout.png)

### Maze Escape

![Maze Escape](./README-i18n/image/mazeescape.png)

### Memory Flip

![Memory Flip](./README-i18n/image/memoryflip.png)

### Minesweeper

![Minesweeper](./README-i18n/image/minesweeper.png)

### Pac-Man
![Pac-Man](./README-i18n/image/pacman.png)

### Rock Paper Scissors

![Rock Paper Scissors](./README-i18n/image/rockpaperscissors.png)

### Air Shooter

![Air Shooter](./README-i18n/image/airshooter.png)

### Number Sliding Puzzle

![Number Sliding Puzzle](./README-i18n/image/numberslidingpuzzle.png)

### Snake

![Snake](./README-i18n/image/snake.png)

### Solitaire

![FreeCell](./README-i18n/image/freecell.png)
![Klondike](./README-i18n/image/klondike.png)
![Spider](./README-i18n/image/spider.png)

### Sudoku

![Sudoku](./README-i18n/image/sudoku.png)

### Tetris

![Tetris](./README-i18n/image/tetris.png)

### Tic-Tac-Toe

![Tic-Tac-Toe](./README-i18n/image/tic-tac-toe.png)

### 24 Points

![24 Points](./README-i18n/image/24-points.png)

### Wordle

![Wordle](./README-i18n/image/wordle.png)

## Support This Project - Give a Star!

If you like this project, please give my repository a star⭐! It's also my motivation to keep updating. If you have any ideas or suggestions, feel free to open an Issue.

The MacOS version hasn't been tested; I don't have the relevant system devices. If you find bugs, please report them. Thank you very much!

GitHub: [MXBraisedFish/TUI-GAME](https://github.com/MXBraisedFish/TUI-GAME)

## Future Plans
> Dreaming big

### Game Plans
1. **Roulette** - Includes classic Russian roulette and the famous Devil's roulette
2. **Code Breaker** - Includes various classic code-breaking games
3. **Blind Box** - A game of pure luck
4. **Highway Racing** - Race on the highway
5. **Dungeon Crawl** - Roguelike dungeon adventure
6. **Guess the Number** - Guess the number within a set number of rounds
7. **Color Matche**r - Match a reference color as closely as possible
8. **Hitori** - Black out duplicate numbers according to the rules
9. **Nonogram** - Paint shapes based on hints
10. **Pipe Mania** - Connect pipes so water can flow from start to end
11. **Breakout** - Bounce the ball to clear all bricks
12. **Paint by Numbers** - Fill in the colors on a given picture
13. **Sokoban** - Move the boxes to the designated locations.

### Additional Features
1. **Modding** - Users can create their own additional games (basic interfaces are already in place)
2. **Boss Key** - One-click to open a disguise interface, effectively preventing bosses from catching you slacking off (will support various computer-related professions: operations, frontend, backend, etc., configurable)
3. **Customizable Shortcuts** - You can modify the in-game key bindings to suit your own play style