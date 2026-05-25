//! Runtime scheduling and render cadence.

use std::time::Duration;

use crate::host_engine::runtime::frame_rate::FrameRateController;
use crate::host_engine::runtime::ui_runtime::ActiveUiPage;

/// Owns runtime frame cadence and idle state.
#[derive(Clone, Debug)]
pub struct RuntimeScheduler {
    frame_rate: FrameRateController,
    was_running_game: bool,
    dirty: bool,
}

impl RuntimeScheduler {
    pub fn root_ui(idle_timeout_secs: u64) -> Self {
        Self {
            frame_rate: FrameRateController::root_ui(idle_timeout_secs),
            was_running_game: false,
            dirty: true,
        }
    }

    pub fn frame_interval(&self) -> Duration {
        self.frame_rate.frame_interval()
    }

    pub fn mark_input(&mut self) {
        self.frame_rate.mark_input();
        self.mark_dirty();
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    #[allow(dead_code)]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    #[allow(dead_code)]
    pub fn should_render(&self) -> bool {
        self.dirty
    }

    pub fn is_root_idle(&self) -> bool {
        self.frame_rate.is_root_idle()
    }

    pub fn enter_overlay(&mut self) {
        self.frame_rate = FrameRateController::overlay();
        self.mark_dirty();
    }

    pub fn sync_runtime_mode(&mut self, active_ui_page: &ActiveUiPage) {
        if !active_ui_page.has_game_session() {
            self.frame_rate
                .set_root_idle_timeout(active_ui_page.root_idle_threshold());
        }

        if active_ui_page.has_game_session() && !self.was_running_game {
            if let Some(game_session) = active_ui_page.game_session() {
                self.frame_rate = FrameRateController::game(
                    game_session.afk_time_secs(),
                    game_session.target_fps(),
                );
            }
            self.was_running_game = true;
        } else if !active_ui_page.has_game_session() && self.was_running_game {
            self.frame_rate = FrameRateController::root_ui(active_ui_page.root_idle_threshold());
            self.was_running_game = false;
        }
    }

    pub fn restore_current_runtime_mode(&mut self, active_ui_page: &ActiveUiPage) {
        if let Some(game_session) = active_ui_page.game_session() {
            self.frame_rate =
                FrameRateController::game(game_session.afk_time_secs(), game_session.target_fps());
            self.was_running_game = true;
        } else {
            self.frame_rate = FrameRateController::root_ui(active_ui_page.root_idle_threshold());
            self.was_running_game = false;
        }
        self.mark_dirty();
    }
}
