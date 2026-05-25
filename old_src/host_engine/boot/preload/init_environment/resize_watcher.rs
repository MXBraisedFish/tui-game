//! 终端尺寸变化通道

use std::sync::mpsc::{self, Receiver, Sender};

/// 尺寸变化事件
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResizeEvent {
    pub width: u16,
    pub height: u16,
}

/// 尺寸变化通道
pub struct ResizeWatcher {
    resize_sender: Sender<ResizeEvent>,
    resize_receiver: Receiver<ResizeEvent>,
}

/// 创建尺寸变化通道
pub fn create() -> ResizeWatcher {
    let (resize_sender, resize_receiver) = mpsc::channel();
    ResizeWatcher {
        resize_sender,
        resize_receiver,
    }
}

impl ResizeWatcher {
    /// 获取发送端，供事件监听线程发送尺寸变化
    pub fn sender(&self) -> Sender<ResizeEvent> {
        self.resize_sender.clone()
    }

    /// 消费并返回接收端
    pub fn into_receiver(self) -> Receiver<ResizeEvent> {
        self.resize_receiver
    }
}
