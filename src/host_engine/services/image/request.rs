use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImagePresentPhase {
  BeforeCanvas,
  AfterCanvas,
}

/// 图片缩放策略。
///
/// 所有尺寸均以终端字符格为单位，不暴露像素。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageFit {
  /// 指定宽度，高度等比缩放
  Width(u16),
  /// 指定高度，宽度等比缩放
  Height(u16),
  /// 精确指定宽高（可能变形）
  Exact { width: u16, height: u16 },
  /// 使用图片原始像素尺寸换算（不做额外缩放）
  Original,
}

/// 上层绘图请求。
///
/// UI 层通过此结构声明"我要显示这张图片"，
/// 不包含协议私有参数，不包含像素尺寸。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DrawImageParams {
  /// 终端列坐标（字符格）
  pub x: u16,
  /// 终端行坐标（字符格）
  pub y: u16,
  /// 图片文件路径（已解析）
  pub path: PathBuf,
  /// 缩放策略
  pub fit: ImageFit,
  /// 是否保持宽高比（Exact 模式下忽略）
  pub preserve_aspect_ratio: bool,
}

/// 以终端字符格为单位的矩形区域。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageCellRect {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
}
