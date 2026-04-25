local M = {
    FPS = 60,
    FRAME_MS = 16,
    MIN_COLS = 10,
    MAX_COLS = 32,
    MIN_ROWS = 8,
    MAX_ROWS = 22,
    MIN_MODE = 1,
    MAX_MODE = 4,
    TILE_PATH = 0,
    TILE_WALL = 1,
    TILE_DOOR = 2,
    TILE_KEY = 3,
    TILE_EXIT = 4,
    WALL_GLYPH = utf8.char(0x2588),
    DEFAULT_COLS = 18,
    DEFAULT_ROWS = 12,
    DEFAULT_MODE = 1,
}

return M
