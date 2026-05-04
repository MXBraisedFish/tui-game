//! 计时器状态仓库

use std::collections::BTreeMap;
use std::time::Instant;

pub const MAX_TIMERS: usize = 64;

/// 计时器仓库。
#[derive(Debug, Default)]
pub struct TimerStore {
    next_id: u64,
    timers: BTreeMap<String, TimerEntry>,
}

/// 单个计时器。
#[derive(Clone, Debug)]
pub struct TimerEntry {
    pub id: String,
    pub note: String,
    pub duration_ms: u64,
    elapsed_ms: u64,
    started_at: Option<Instant>,
    status: TimerStatus,
}

/// 计时器状态。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerStatus {
    Init,
    Running,
    Pause,
    Completed,
}

impl TimerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Running => "running",
            Self::Pause => "pause",
            Self::Completed => "completed",
        }
    }
}

impl TimerStore {
    /// 创建计时器。
    pub fn create_timer(&mut self, duration_ms: u64, note: String) -> mlua::Result<String> {
        if self.timers.len() >= MAX_TIMERS {
            return Err(mlua::Error::external("timer limit reached"));
        }
        self.next_id += 1;
        let id = format!("timer_{}", self.next_id);
        self.timers.insert(
            id.clone(),
            TimerEntry {
                id: id.clone(),
                note,
                duration_ms,
                elapsed_ms: 0,
                started_at: None,
                status: TimerStatus::Init,
            },
        );
        Ok(id)
    }

    /// 判断计时器是否存在。
    pub fn contains_timer(&self, id: &str) -> bool {
        self.timers.contains_key(id)
    }

    /// 删除计时器。
    pub fn kill_timer(&mut self, id: &str) -> mlua::Result<()> {
        self.timers
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| timer_not_found(id))
    }

    /// 获取可变计时器。
    pub fn timer_mut(&mut self, id: &str) -> mlua::Result<&mut TimerEntry> {
        self.timers.get_mut(id).ok_or_else(|| timer_not_found(id))
    }

    /// 计时器列表。
    pub fn timers_mut(&mut self) -> impl Iterator<Item = &mut TimerEntry> {
        self.timers.values_mut()
    }
}

impl TimerEntry {
    /// 启动计时器。
    pub fn start(&mut self) {
        self.normalize();
        if self.status == TimerStatus::Init {
            self.started_at = Some(Instant::now());
            self.status = TimerStatus::Running;
        }
    }

    /// 暂停计时器。
    pub fn pause(&mut self) {
        self.normalize();
        if self.status == TimerStatus::Running {
            self.elapsed_ms = self.elapsed_ms();
            self.started_at = None;
            self.status = TimerStatus::Pause;
        }
    }

    /// 恢复计时器。
    pub fn resume(&mut self) {
        self.normalize();
        if self.status == TimerStatus::Pause {
            self.started_at = Some(Instant::now());
            self.status = TimerStatus::Running;
        }
    }

    /// 重置计时器。
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.started_at = None;
        self.status = TimerStatus::Init;
    }

    /// 重启计时器。
    pub fn restart(&mut self) {
        self.elapsed_ms = 0;
        self.started_at = Some(Instant::now());
        self.status = TimerStatus::Running;
    }

    /// 设置备注。
    pub fn set_note(&mut self, note: String) {
        self.note = note;
    }

    /// 当前状态。
    pub fn status(&mut self) -> TimerStatus {
        self.normalize();
        self.status
    }

    /// 已过时间。
    pub fn elapsed_ms(&self) -> u64 {
        match self.status {
            TimerStatus::Init => 0,
            TimerStatus::Pause | TimerStatus::Completed => self.elapsed_ms.min(self.duration_ms),
            TimerStatus::Running => {
                let delta_ms = self
                    .started_at
                    .map(|started_at| started_at.elapsed().as_millis() as u64)
                    .unwrap_or(0);
                (self.elapsed_ms + delta_ms).min(self.duration_ms)
            }
        }
    }

    /// 剩余时间。
    pub fn remaining_ms(&self) -> u64 {
        self.duration_ms.saturating_sub(self.elapsed_ms())
    }

    /// 是否完成。
    pub fn is_completed(&mut self) -> bool {
        self.normalize();
        self.status == TimerStatus::Completed
    }

    /// 归一化运行态。
    pub fn normalize(&mut self) {
        if self.status == TimerStatus::Running {
            let elapsed_ms = self.elapsed_ms();
            if elapsed_ms >= self.duration_ms {
                self.elapsed_ms = self.duration_ms;
                self.started_at = None;
                self.status = TimerStatus::Completed;
            }
        }
    }
}

fn timer_not_found(id: &str) -> mlua::Error {
    mlua::Error::external(format!("timer not found: {id}"))
}
