-- The shared settings used by the game.
local config = {
  -- The number of columns on the board.
  width = 10,
  -- The number of rows on the board.
  height = 20,
  -- The countdown duration in seconds before play begins.
  ready_duration = 2,
  -- The delay in seconds before a held movement key starts repeating.
  move_initial_delay = 0.2,
  -- The delay in seconds between repeated sideways moves.
  move_interval = 0.1,
  -- The delay in seconds between rows of the end animation.
  end_row_interval = 0.08,
  -- The time in seconds to hold the completed end animation.
  end_hold_time = 0.3,
  -- The piece types in the order used by random selection and statistics.
  piece_order = { "i", "o", "t", "z", "l", "s", "j" },
  -- The cleared-line count names in display order.
  line_order = { "single", "double", "triple", "tetris", "lines" },
  -- The base scores for clearing one, two, three, or four rows.
  clear_scores = { 40, 100, 300, 1200 },
  -- The falling delays in 60 Hz frames for the first ten challenge levels.
  drop_frames = { [0] = 48, 43, 38, 33, 28, 23, 18, 13, 8, 6 },
  -- The color group used by each piece type.
  piece_groups = { i = 1, o = 1, t = 1, z = 2, l = 2, s = 3, j = 3 }
}

-- The full list of colors available to the game.
config.palette = {
  [0] = "#545454",
  [1] = "#001e74",
  [2] = "#081090",
  [3] = "#300088",
  [4] = "#440064",
  [5] = "#5c0030",
  [6] = "#540400",
  [7] = "#3c1800",
  [8] = "#202a00",
  [9] = "#083a00",
  [10] = "#014c01",
  [11] = "#003c00",
  [12] = "#00323c",
  [13] = "#989698",
  [14] = "#0072bc",
  [15] = "#3050f8",
  [16] = "#6424f4",
  [17] = "#8814b0",
  [18] = "#b41050",
  [19] = "#a82200",
  [20] = "#884000",
  [21] = "#645c00",
  [22] = "#347400",
  [23] = "#088800",
  [24] = "#008424",
  [25] = "#007868",
  [26] = "#005e8c",
  [27] = "#eceeec",
  [28] = "#00ccf0",
  [29] = "#609cfc",
  [30] = "#9c7cfc",
  [31] = "#c870dc",
  [32] = "#ec66a0",
  [33] = "#f06e48",
  [34] = "#e89e24",
  [35] = "#c8c430",
  [36] = "#84e230",
  [37] = "#30ec44",
  [38] = "#2ce684",
  [39] = "#3cd2c4",
  [40] = "#54ace8",
  [41] = "#eceeec",
  [42] = "#6ce2fc",
  [43] = "#acbefc",
  [44] = "#d4b0fc",
  [45] = "#f0aaec",
  [46] = "#f8a8b4",
  [47] = "#fcb27c",
  [48] = "#f4d270",
  [49] = "#e0e884",
  [50] = "#b4f88c",
  [51] = "#88fa98",
  [52] = "#90eeb8",
  [53] = "#9cdedc",
  [54] = "#a4c8f0",
}

-- The three palette indexes used by each repeating challenge color set.
config.fixed_color_rows = {
  [0] = { 27, 27, 27 },
  [1] = { 19, 19, 19 },
  [2] = { 40, 40, 40 },
  [3] = { 35, 30, 30 },
  [4] = { 14, 34, 23 },
  [5] = { 38, 15, 32 },
  [6] = { 26, 34, 19 },
  [7] = { 34, 38, 31 },
  [8] = { 2, 15, 23 },
  [9] = { 32, 19, 29 },
}

-- The row and column offsets of the four cells in each piece shape.
config.pieces = {
  i = { { 0, 1 }, { 1, 1 }, { 2, 1 }, { 3, 1 } },
  o = { { 0, 0 }, { 0, 1 }, { 1, 0 }, { 1, 1 } },
  t = { { 0, 1 }, { 1, 0 }, { 1, 1 }, { 2, 1 } },
  z = { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 1, 2 } },
  l = { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 2, 1 } },
  s = { { 0, 1 }, { 0, 2 }, { 1, 0 }, { 1, 1 } },
  j = { { 0, 0 }, { 0, 1 }, { 0, 2 }, { 1, 2 } }
}

return config
