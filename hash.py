ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
OFFSET = 0x9E3779B97F4A7C15


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def splitmix64(value: int) -> int:
    value = (value + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
    return value ^ (value >> 31)


def stable_hash16(seed: str) -> str:
    state = fnv1a64(seed.encode("utf-8"))
    out = []
    for index in range(16):
        state = splitmix64((state + OFFSET + index) & 0xFFFFFFFFFFFFFFFF)
        out.append(ALPHABET[state % 62])
    return "".join(out)


def sanitize_segment(raw: str, fallback: str) -> str:
    out = []
    last_sep = False
    for ch in raw.strip():
        if ch.isalnum():
            mapped = ch.lower()
        elif ch in "_-. ":
            mapped = "_"
        else:
            continue
        if mapped == "_":
            if last_sep or not out:
                continue
            last_sep = True
        else:
            last_sep = False
        out.append(mapped)
    value = "".join(out).strip("_")
    return value or fallback.lower()


def generate_mod_game_id(author: str, package_name: str, game_name: str, namespace: str = "mod") -> str:
    package_segment = sanitize_segment(package_name, namespace)
    seed = f"{author.strip()}\n{package_name.strip()}\n{game_name.strip()}"
    return f"mod_game_{package_segment}_{stable_hash16(seed)}"


if __name__ == "__main__":
    official = [
        "2048",
        "blackjack",
        "color_memory",
        "lights_out",
        "maze_escape",
        "memory_flip",
        "minesweeper",
        "pacman",
        "rock_paper_scissors",
        "shooter",
        "sliding_puzzle",
        "snake",
        "solitaire",
        "sudoku",
        "tetris",
        "tic_tac_toe",
        "twenty_four",
        "wordle",
    ]
    for name in official:
        print(f"{name}: {stable_hash16(name)}")
