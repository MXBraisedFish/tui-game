#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNode {
    Root,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTree {
    active_path: Vec<UiNode>,
}

impl UiTree {
    pub fn new() -> Self {
        Self {
            active_path: vec![UiNode::Root],
        }
    }

    pub fn active_path(&self) -> &[UiNode] {
        &self.active_path
    }

    pub fn current(&self) -> Option<UiNode> {
        self.active_path.last().copied()
    }

    pub fn enter(&mut self, node: UiNode) {
        self.active_path.push(node);
    }

    pub fn back(&mut self) -> Option<UiNode> {
        if self.active_path.len() <= 1 {
            return None;
        }
        self.active_path.pop()
    }

    pub fn reset(&mut self) {
        self.active_path.clear();
        self.active_path.push(UiNode::Root);
    }
}
