local M = {
    DEFAULT_DIFFICULTY = 2,
    MIN_DIFFICULTY = 1,
    MAX_DIFFICULTY = 3,
    DIFFICULTY_TO_SIZE = {
        [1] = 2,
        [2] = 4,
        [3] = 6,
    },
    FPS = 60,
    FRAME_MS = 16,
    CELL_W = 4,
    CELL_H = 3,
    CELL_STEP_X = 6,
    CELL_STEP_Y = 2,
    LABEL_W = 3,
    HIDE_DELAY_MS = 500,
    SYMBOLS = {
        "!", "@", "#", "$", "%", "^", "&", "*", "A",
        "B", "C", "D", "E", "F", "G", "H", "I", "J",
    },
    PALETTE = {
        "rgb(255,110,110)", "rgb(255,150,90)", "rgb(255,205,90)",
        "rgb(200,235,90)", "rgb(120,230,120)", "rgb(90,215,175)",
        "rgb(90,200,245)", "rgb(125,165,250)", "rgb(165,145,245)",
        "rgb(205,130,245)", "rgb(245,125,220)", "rgb(245,125,175)",
        "rgb(245,160,160)", "rgb(240,190,140)", "rgb(225,215,140)",
        "rgb(190,220,150)", "rgb(150,215,195)", "rgb(150,200,220)",
    },
}

return M
