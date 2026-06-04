//! ANSI256 颜色降级模块
//!
//! 当终端不支持真彩色（truecolor）时，将 RGB 颜色近似映射到
//! 256 色调色板中最近的匹配色。
//!
//! 算法使用 6×6×6 颜色立方体（216 色）+ 灰度渐变（24 色），
//! 通过欧几里德距离选择最接近的 ANSI256 索引。

/// 将 RGB 颜色映射到最近的 ANSI256 颜色索引
///
/// 分别计算 RGB 到 6×6×6 颜色立方体和灰度渐变的最短欧几里德距离，
/// 返回距离更近的那个索引。
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
  let color_cube_index = rgb_to_color_cube_index(r, g, b);
  let grayscale_index = rgb_to_grayscale_index(r, g, b);

  let cube_rgb = ansi256_to_rgb(color_cube_index);
  let gray_rgb = ansi256_to_rgb(grayscale_index);

  let cube_distance = color_distance(r, g, b, cube_rgb.0, cube_rgb.1, cube_rgb.2);
  let gray_distance = color_distance(r, g, b, gray_rgb.0, gray_rgb.1, gray_rgb.2);

  if gray_distance < cube_distance {
    grayscale_index
  } else {
    color_cube_index
  }
}

/// 将 RGB 映射到 6×6×6 颜色立方体索引（16..231）
fn rgb_to_color_cube_index(r: u8, g: u8, b: u8) -> u8 {
  let r_level = rgb_to_cube_level(r);
  let g_level = rgb_to_cube_level(g);
  let b_level = rgb_to_cube_level(b);
  16 + 36 * r_level + 6 * g_level + b_level
}

/// 将 RGB 通道值映射到颜色立方体的 6 个级别之一
fn rgb_to_cube_level(value: u8) -> u8 {
  if value < 48 {
    0
  } else if value < 115 {
    1
  } else {
    ((value as u16 - 35) / 40) as u8
  }
}

/// 将 RGB 映射到灰度渐变索引（232..255）
fn rgb_to_grayscale_index(r: u8, g: u8, b: u8) -> u8 {
  let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
  if gray < 8 {
    return 16;
  }
  if gray > 248 {
    return 231;
  }
  232 + ((gray as u16 - 8) / 10) as u8
}

/// 将 ANSI256 索引转换回 RGB 值（用于距离计算）
fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
  // 颜色立方体区域（16..231）
  if index >= 16 && index <= 231 {
    let value = index - 16;
    let r = value / 36;
    let g = (value % 36) / 6;
    let b = value % 6;
    return (cube_level_to_rgb(r), cube_level_to_rgb(g), cube_level_to_rgb(b));
  }

  // 灰度渐变区域（232..255）
  if index >= 232 {
    let gray = 8 + (index - 232) * 10;
    return (gray, gray, gray);
  }

  // 基本 16 色（0..15），不参与降级比较
  (0, 0, 0)
}

/// 将颜色立方体级别（0..5）转换为 RGB 通道值
fn cube_level_to_rgb(level: u8) -> u8 {
  if level == 0 {
    0
  } else {
    55 + level * 40
  }
}

/// 计算两个 RGB 颜色之间的欧几里德距离的平方
fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
  let dr = r1 as i32 - r2 as i32;
  let dg = g1 as i32 - g2 as i32;
  let db = b1 as i32 - b2 as i32;
  (dr * dr + dg * dg + db * db) as u32
}
