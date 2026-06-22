import msvcrt
import sys

# ---------- 状态管理 ----------
game_mode = True
text_mode = False
input_buffer = ""           # 当前输入的文本
cursor_pos = 0              # 光标位置（暂未实现，可扩展）

# ---------- 辅助函数 ----------
def is_printable_ascii(char_code: int) -> bool:
    """判断是否为可打印ASCII字符（包括空格）"""
    return 32 <= char_code <= 126

def is_extended_key(char_code: int) -> bool:
    """Windows扩展键前缀（方向键、功能键等）"""
    return char_code == 0xe0

def handle_game_key(key_code: int):
    """游戏模式下的按键处理（模拟游戏动作）"""
    if key_code == 0xe0:   # 扩展键前缀，需要再读一个字节
        ext = ord(msvcrt.getch())
        if ext == 0x48:   print("[游戏] 上")
        elif ext == 0x50: print("[游戏] 下")
        elif ext == 0x4b: print("[游戏] 左")
        elif ext == 0x4d: print("[游戏] 右")
        else:             print(f"[游戏] 扩展键 {ext}")
        return

    ch = chr(key_code)
    if ch == 't':
        print("\n[系统] 进入文本模式")
        enter_text_mode()
    elif ch == ' ':
        print("[游戏] 跳跃")
    elif ch == '\r':      # Enter键（游戏模式下的特殊用途，可忽略）
        pass
    else:
        print(f"[游戏] 按键 {ch} (码 {key_code})")

def handle_text_key(key_code: int):
    """文本模式下的按键处理"""
    global input_buffer, cursor_pos

    if key_code == 0xe0:   # 扩展键（方向键等）在文本模式下可做光标移动，简化实现略过
        ext = ord(msvcrt.getch())
        # 可扩展：方向键移动光标
        return

    ch = chr(key_code) if key_code < 256 else '?'

    if ch == '\r':         # Enter 提交
        print(f"\n[提交] {input_buffer}")
        exit_text_mode(commit=True)
    elif ch == '\x1b':     # Esc 取消
        print("\n[取消] 丢弃草稿")
        exit_text_mode(commit=False)
    elif ch == '\x08':     # Backspace
        if input_buffer:
            input_buffer = input_buffer[:-1]
            print(f"\r[输入] {input_buffer}  ", end='')  # 简单回显，不精细光标
    elif is_printable_ascii(key_code):
        input_buffer += ch
        print(f"\r[输入] {input_buffer}  ", end='')
    # 其他控制键忽略

def enter_text_mode():
    global game_mode, text_mode, input_buffer
    game_mode = False
    text_mode = True
    input_buffer = ""
    print("\n[文本模式] 输入文字 (Enter提交, Esc取消)")

def exit_text_mode(commit: bool):
    global game_mode, text_mode, input_buffer
    if commit:
        # 模拟发送消息（如网络发送）
        print(f"[发送] {input_buffer}")
    else:
        print("[丢弃] 草稿已清除")
    input_buffer = ""
    game_mode = True
    text_mode = False
    print("\n[游戏模式] 按 t 打开聊天")

# ---------- 主循环 ----------
def main():
    print("=== 终端输入模式切换模拟 (Windows) ===")
    print("[游戏模式] 按 t 进入文本模式")
    print("方向键/空格 模拟游戏动作")
    print("按 Ctrl+C 退出\n")

    while True:
        try:
            key_code = ord(msvcrt.getch())
        except KeyboardInterrupt:
            print("\n退出")
            sys.exit(0)

        if game_mode:
            handle_game_key(key_code)
        elif text_mode:
            handle_text_key(key_code)

if __name__ == "__main__":
    main()