//! Completion routing, profile reconciliation, and context-aware failure logging.

use super::*;
use crate::host_engine::app::EngineEvent;
use crate::host_engine::services::{
  AudioAsyncEvent, ExportAsyncEvent, GameSaveCapabilities, NetworkEvent, ScreenshotAsyncEvent,
  VideoAsyncEvent,
};

/// The runtime engine events representation used by this module.
///
/// # Fields
///
/// * `package` - The ordered package retained by this owner.
/// * `export` - The ordered export retained by this owner.
/// * `screenshot` - The ordered screenshot retained by this owner.
/// * `video` - The ordered video retained by this owner.
pub(super) struct RuntimeEngineEvents {
  /// The ordered package retained by this owner.
  pub package: Vec<PackageEvent>,
  /// The ordered export retained by this owner.
  pub export: Vec<ExportAsyncEvent>,
  /// The ordered screenshot retained by this owner.
  pub screenshot: Vec<ScreenshotAsyncEvent>,
  /// The ordered video retained by this owner.
  pub video: Vec<VideoAsyncEvent>,
}

/// Drain and return the queued engine events.
///
/// # Arguments
///
/// * `services` - The application services supplied by the lifecycle phase.
/// * `lua_events` - The Lua events.
/// * `frame` - The composed terminal-cell frame.
pub(super) fn drain_engine_events(
  services: &mut EngineServices,
  lua_events: &mut LuaEventBroker,
  frame: u64,
) -> RuntimeEngineEvents {
  let mut package_events = Vec::new();
  let mut export_events = Vec::new();
  let mut screenshot_events = Vec::new();
  let mut video_events = Vec::new();

  for event in services.engine_events.drain() {
    let routed = event
      .lua_routable()
      .map(|routable| lua_events.route_service_event(frame, routable));
    if let Some(Err(error)) = routed {
      match error {
        LuaEnqueueError::StaleTaskCompletion(_) | LuaEnqueueError::StaleAudioEvent(_) => {}
        error => services.log.warn_message(
          LogSource::Lua,
          HostLogMessage::new(
            "log_info.fallback.activated",
            "{domain} entered fallback mode: {reason}",
          )
          .param("domain", "lua-event-router")
          .param("reason", format!("{error:?}")),
        ),
      }
    }
    match event {
      EngineEvent::InputKey(event) => services.input.queue_key_event(event, &mut services.log),
      EngineEvent::System(event) => services.input.queue_system_event(event, &mut services.log),
      EngineEvent::Package(event) => {
        let Some(event) = services
          .package
          .handle_async_event(event, &mut services.log)
        else {
          continue;
        };
        if matches!(event, PackageEvent::ScanFinished { .. }) {
          synchronize_key_bindings_profile(services);
          reconcile_game_save_profile(services);
        }
        if matches!(event, PackageEvent::WatchChanged { .. }) {
          let _ = services.package.request_rescan(&services.async_runtime);
        }
        package_events.push(event);
      }
      EngineEvent::Export(event) => export_events.push(event),
      EngineEvent::Screenshot(event) => {
        services.screenshot.handle_engine_event(&event);
        match event {
          ScreenshotAsyncEvent::Progress {
            task_id,
            completed_rows,
            total_rows,
          } => screenshot_events.push(ScreenshotAsyncEvent::Progress {
            task_id,
            completed_rows,
            total_rows,
          }),
          ScreenshotAsyncEvent::Saved { task_id, png_path } => {
            services.log.info_message(
              LogSource::Storage,
              HostLogMessage::new(
                "log_info.export.image_finished",
                "Image export {id} finished: {path}",
              )
              .param("id", format!("{task_id:?}"))
              .param("path", png_path.display().to_string()),
            );
            screenshot_events.push(ScreenshotAsyncEvent::Saved { task_id, png_path });
          }
          ScreenshotAsyncEvent::Failed { task_id, error } => {
            services.log.warn_message(
              LogSource::Storage,
              HostLogMessage::new(
                "log_info.export.image_failed",
                "Image export {id} failed: {error}",
              )
              .param("id", format!("{task_id:?}"))
              .param("error", &error),
            );
            screenshot_events.push(ScreenshotAsyncEvent::Failed { task_id, error });
          }
        }
      }
      EngineEvent::Recording(event) => {
        services.recording.handle_engine_event(&event);
        match event {
          crate::host_engine::services::RecordingAsyncEvent::Saved { task_id, path } => {
            services.log.info_message(
              LogSource::Storage,
              HostLogMessage::new(
                "log_info.external.operation",
                "Host {operation} operation entered {state}.",
              )
              .param("operation", format!("recording-{task_id:?}"))
              .param("state", format!("saved: {}", path.display())),
            );
          }
          crate::host_engine::services::RecordingAsyncEvent::Failed { task_id, error } => {
            services.log.warn_message(
              LogSource::Storage,
              HostLogMessage::new(
                "log_info.fallback.activated",
                "{domain} entered fallback mode: {reason}",
              )
              .param("domain", format!("recording-{task_id:?}"))
              .param("reason", error),
            );
          }
        }
      }
      EngineEvent::Video(event) => {
        services.video.handle_engine_event(&event);
        match &event {
          VideoAsyncEvent::Saved {
            task_id,
            source_path: _,
            mp4_path,
          } => services.log.info_message(
            LogSource::Storage,
            HostLogMessage::new(
              "log_info.export.video_finished",
              "Video export {id} finished: {path}",
            )
            .param("id", format!("{task_id:?}"))
            .param("path", mp4_path.display().to_string()),
          ),
          VideoAsyncEvent::Failed {
            task_id,
            stage,
            error,
            ..
          } => services.log.warn_message(
            LogSource::Storage,
            HostLogMessage::new(
              "log_info.export.video_failed",
              "Video export {id} failed during {stage}: {error}",
            )
            .param("id", format!("{task_id:?}"))
            .param("stage", format!("{stage:?}"))
            .param("error", error),
          ),
          VideoAsyncEvent::Preparing { .. }
          | VideoAsyncEvent::Encoder { .. }
          | VideoAsyncEvent::Progress { .. }
          | VideoAsyncEvent::Finalizing { .. } => {}
        }
        video_events.push(event);
      }
      EngineEvent::Network(event) => {
        services.network.handle_engine_event(&event);
        match &event {
          NetworkEvent::Started { .. } | NetworkEvent::Finished { .. } => {}
          NetworkEvent::Failed { task_id, error, .. } => services.log.warn_message(
            LogSource::Engine,
            HostLogMessage::new(
              "log_info.network.failed",
              "Host network request {id} failed with {code}.",
            )
            .param("id", format!("{task_id:?}"))
            .param("code", error.code.as_str()),
          ),
          NetworkEvent::Cancelled { .. } => {}
        }
      }
      EngineEvent::Audio(event) => {
        services.audio.handle_engine_event(&event);
        services
          .recording
          .handle_audio_event(&event, &services.async_runtime);
        match &event {
          AudioAsyncEvent::Failed { error, .. } => services.log.warn_message(
            LogSource::Audio,
            HostLogMessage::new(
              "log_info.fallback.activated",
              "{domain} entered fallback mode: {reason}",
            )
            .param("domain", "audio-object")
            .param("reason", error.code.as_str()),
          ),
          AudioAsyncEvent::BackendFailed { error } => services.log.warn_message(
            LogSource::Audio,
            HostLogMessage::new(
              "log_info.audio.unavailable",
              "Audio output is unavailable; playback was disabled: {error}",
            )
            .param("error", error.code.as_str()),
          ),
          AudioAsyncEvent::CaptureFailed { error, .. } => services.log.warn_message(
            LogSource::Audio,
            HostLogMessage::new(
              "log_info.fallback.activated",
              "{domain} entered fallback mode: {reason}",
            )
            .param("domain", "audio-capture")
            .param("reason", error.code.as_str()),
          ),
          AudioAsyncEvent::Ready { .. }
          | AudioAsyncEvent::Started { .. }
          | AudioAsyncEvent::Paused { .. }
          | AudioAsyncEvent::Resumed { .. }
          | AudioAsyncEvent::Stopped { .. }
          | AudioAsyncEvent::Finished { .. }
          | AudioAsyncEvent::CaptureSaved { .. } => {}
        }
      }
      EngineEvent::File(_)
      | EngineEvent::Image(_)
      | EngineEvent::Time(_)
      | EngineEvent::TaskFinished => {}
      // Service-specific completion already includes business context; do not log the generic
      // failure again.
      EngineEvent::TaskFailed { .. } => {}
      EngineEvent::Log { source, message } => {
        services
          .log
          .warn_operation_failed(source, "async_task", "engine_event", message);
      }
    }
  }

  RuntimeEngineEvents {
    package: package_events,
    export: export_events,
    screenshot: screenshot_events,
    video: video_events,
  }
}

/// Reconcile stored continue/best results with the scanned games' declared save capabilities.
pub(super) fn reconcile_game_save_profile(services: &mut EngineServices) {
  let games = services
    .package
    .games()
    .into_iter()
    .filter_map(|package| {
      let game = package.game.as_ref()?;
      Some(GameSaveCapabilities {
        package_id: package.id.clone(),
        save_enabled: game.save,
        score_enabled: game.score.as_ref().is_some_and(|score| score.enabled),
      })
    })
    .collect::<Vec<_>>();
  if let Err(error) = services
    .storage
    .reconcile_game_save_capabilities(&games, &mut services.log)
  {
    services.log.error_message(
      LogSource::Storage,
      HostLogMessage::new(
        "log_info.storage.operation_failed",
        "Storage operation {operation} failed for {path}: {error}",
      )
      .param("operation", "reconcile-game-save")
      .param("path", "data/profiles/game_save.json")
      .param("error", error.to_string()),
    );
  }
}
