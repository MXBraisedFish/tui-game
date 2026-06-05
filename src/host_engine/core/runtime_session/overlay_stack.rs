#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
    ConfirmExit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayStack {
    stack: Vec<OverlayKind>,
}

impl OverlayStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, overlay: OverlayKind) {
        self.stack.push(overlay);
    }

    pub fn pop(&mut self) -> Option<OverlayKind> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<OverlayKind> {
        self.stack.last().copied()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
