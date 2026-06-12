use std::io;

/// 图片服务统一错误类型。
#[derive(Debug)]
pub enum ImageError {
  /// IO 错误（文件读取、终端写入等）
  Io(io::Error),
  /// 图片解码失败
  Decode(String),
  /// 图片编码失败
  Encode(String),
  /// 不支持的文件扩展名
  UnsupportedExtension,
  /// 当前终端不支持所选图片协议
  UnsupportedProtocol,
  /// 图片超出终端可视区域
  OutOfBounds,
  /// 每帧只支持一个图片请求（多图支持待实现）
  /// TODO: 替换 pending 为 Vec 后移除此变体
  MultipleImagesUnsupported,
  /// 终端 writer 不可用
  MissingTerminalWriter,
}

impl From<io::Error> for ImageError {
  fn from(error: io::Error) -> Self {
    Self::Io(error)
  }
}
