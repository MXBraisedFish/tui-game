use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use crate::host_engine::services::widget::runtime_object::RuntimeObjectPool;

use super::*;

fn float_tween(from: f64, to: f64, milliseconds: u64) -> AnimationSource {
  AnimationSource::Tween(Arc::new(TweenDefinition {
    from: AnimationValue::Float(from),
    to: AnimationValue::Float(to),
    duration: Duration::from_millis(milliseconds),
    easing: AnimationEasing::Linear,
    interpolation: AnimationInterpolation::Linear,
  }))
}

fn value_binding(id: AnimationValueId, initial: f64) -> AnimationBinding {
  AnimationBinding {
    track: 0,
    target: AnimationTarget::Value(id),
    property: AnimationProperty::Value,
    initial_value: AnimationValue::Float(initial),
  }
}

fn float(value: Option<&AnimationValue>) -> f64 {
  match value {
    Some(AnimationValue::Float(value)) => *value,
    other => panic!("expected float, got {other:?}"),
  }
}

#[test]
fn tween_updates_and_commits_a_standalone_value() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 10.0, 1_000),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(500));
  assert_eq!(float(service.value(&pool, value)), 5.0);
  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(500));
  assert_eq!(float(service.value(&pool, value)), 10.0);
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Finished));
}

#[test]
fn delay_and_clock_select_only_the_matching_playback() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let ui_value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let game_value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let delayed = AnimationPlaybackOptions {
    delay: Duration::from_millis(200),
    ..AnimationPlaybackOptions::default()
  };
  service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 10.0, 100),
      vec![value_binding(ui_value, 0.0)],
      delayed,
    )
    .unwrap();
  service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 10.0, 100),
      vec![value_binding(game_value, 0.0)],
      AnimationPlaybackOptions {
        clock: AnimationClock::Game,
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(150));
  assert_eq!(float(service.value(&pool, ui_value)), 0.0);
  assert_eq!(float(service.value(&pool, game_value)), 0.0);
  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(100));
  assert_eq!(float(service.value(&pool, ui_value)), 5.0);
  service.update(&mut pool, AnimationClock::Game, Duration::from_millis(50));
  assert_eq!(float(service.value(&pool, game_value)), 5.0);
}

#[test]
fn ping_pong_repeat_returns_to_the_start_value() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 1.0, 100),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions {
        repeat: AnimationRepeatOptions {
          mode: AnimationRepeatMode::PingPong,
          count: AnimationRepeatCount::Finite(2),
        },
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(200));
  assert_eq!(float(service.value(&pool, value)), 0.0);
  assert_eq!(service.completed_cycles(&pool, handle), Some(2));
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Finished));
}

#[test]
fn restart_repeat_samples_the_next_cycle_start_at_the_boundary() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 1.0, 100),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions {
        repeat: AnimationRepeatOptions {
          mode: AnimationRepeatMode::Restart,
          count: AnimationRepeatCount::Finite(2),
        },
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(100));
  assert_eq!(float(service.value(&pool, value)), 0.0);
  assert_eq!(service.completed_cycles(&pool, handle), Some(1));
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Playing));
}

#[test]
fn cancel_commits_current_value_and_reset_restores_initial_value() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(2.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(2.0, 12.0, 1_000),
      vec![value_binding(value, 2.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(400));
  service.cancel(&mut pool, handle).unwrap();
  assert_eq!(float(service.value(&pool, value)), 6.0);
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Cancelled));
  service.reset(&mut pool, handle).unwrap();
  assert_eq!(float(service.value(&pool, value)), 2.0);
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Idle));
}

#[test]
fn restore_end_mode_removes_the_override_without_changing_base() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(3.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(3.0, 9.0, 100),
      vec![value_binding(value, 3.0)],
      AnimationPlaybackOptions {
        end_mode: AnimationEndMode::Restore,
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.finish(&mut pool, handle).unwrap();
  assert_eq!(float(service.value(&pool, value)), 3.0);
}

#[test]
fn controls_report_invalid_state_and_generation_ids_reject_stale_handles() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let options = AnimationPlaybackOptions {
    auto_play: false,
    ..AnimationPlaybackOptions::default()
  };
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 1.0, 100),
      vec![value_binding(value, 0.0)],
      options.clone(),
    )
    .unwrap();

  assert!(matches!(
    service.pause(&mut pool, handle),
    Err(AnimationError::InvalidPlaybackState { .. })
  ));
  service.start(&mut pool, handle).unwrap();
  service.pause(&mut pool, handle).unwrap();
  service.resume(&mut pool, handle).unwrap();
  service.remove(&mut pool, handle).unwrap();
  assert_eq!(service.state(&pool, handle), None);

  let replacement = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 1.0, 100),
      vec![value_binding(value, 0.0)],
      options,
    )
    .unwrap();
  assert_eq!(handle.id().index(), replacement.id().index());
  assert_ne!(handle.id().generation(), replacement.id().generation());
}

#[test]
fn markers_events_and_callbacks_preserve_occurrence_order() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let clip = AnimationClip {
    duration: Duration::from_millis(100),
    tracks: vec![AnimationTrack {
      property: AnimationProperty::Value,
      keyframes: vec![
        AnimationKeyframe {
          at: Duration::ZERO,
          value: AnimationValue::Float(0.0),
          easing: AnimationEasing::Linear,
          interpolation: AnimationInterpolation::Linear,
        },
        AnimationKeyframe {
          at: Duration::from_millis(100),
          value: AnimationValue::Float(1.0),
          easing: AnimationEasing::Linear,
          interpolation: AnimationInterpolation::Linear,
        },
      ],
    }],
    markers: vec![AnimationMarker {
      at: Duration::from_millis(50),
      name: "middle".into(),
    }],
  };
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      AnimationSource::Clip(Arc::new(clip)),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions {
        emit_events: true,
        callback: Some(AnimationCallbackId(7)),
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(100));
  let kinds = service
    .take_events(&mut pool, handle)
    .into_iter()
    .map(|event| event.kind)
    .collect::<Vec<_>>();
  assert_eq!(
    kinds,
    vec![
      AnimationEventKind::Started,
      AnimationEventKind::Marker {
        name: "middle".into()
      },
      AnimationEventKind::Finished,
    ]
  );
  let callbacks = service.take_callback_requests(&mut pool);
  assert_eq!(callbacks.len(), 3);
  assert!(
    callbacks
      .iter()
      .all(|request| request.callback == AnimationCallbackId(7))
  );
}

#[test]
fn owner_cleanup_removes_only_owned_playbacks() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let first = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let second = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let first_handle = service
    .play(
      &mut pool,
      AnimationOwner::UiPool(UiPoolId(1)),
      float_tween(0.0, 1.0, 100),
      vec![value_binding(first, 0.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();
  let second_handle = service
    .play(
      &mut pool,
      AnimationOwner::UiPool(UiPoolId(2)),
      float_tween(0.0, 1.0, 100),
      vec![value_binding(second, 0.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.clear_owner(&mut pool, AnimationOwner::UiPool(UiPoolId(1)));
  assert_eq!(service.state(&pool, first_handle), None);
  assert!(service.state(&pool, second_handle).is_some());
}

#[test]
fn character_frames_normalize_newlines_and_allow_different_dimensions() {
  let clip = AnimationClip::character_frames_from_text(vec![
    (Duration::from_millis(80), "我\r\nA".into()),
    (Duration::from_millis(120), "🙂 wide\rsecond\nthird".into()),
  ])
  .unwrap();
  assert_eq!(clip.duration, Duration::from_millis(200));
  let AnimationValue::CharacterFrame(lines) = &clip.tracks[0].keyframes[0].value else {
    panic!("expected character frame")
  };
  assert_eq!(&**lines, &["我".to_string(), "A".to_string()]);
  assert_eq!(
    CharacterFrame::from_text(Duration::from_millis(1), "我A").display_size(),
    (3, 1)
  );

  let too_wide = CharacterFrame::from_text(Duration::from_millis(1), "x".repeat(513));
  assert!(matches!(
    AnimationClip::character_frames(vec![too_wide]),
    Err(AnimationError::CharacterFrameTooLarge { .. })
  ));
}

#[test]
fn character_frames_use_step_sampling_and_replace_the_whole_frame() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let initial = AnimationValue::CharacterFrame(Arc::from(["old".to_string()]));
  let value = service.create_value(&mut pool, initial.clone());
  let clip = AnimationClip::character_frames_from_text(vec![
    (Duration::from_millis(100), "long\nframe".into()),
    (Duration::from_millis(100), "短".into()),
  ])
  .unwrap();
  service
    .play(
      &mut pool,
      AnimationOwner::Host,
      AnimationSource::Clip(Arc::new(clip)),
      vec![AnimationBinding {
        track: 0,
        target: AnimationTarget::Value(value),
        property: AnimationProperty::Value,
        initial_value: initial,
      }],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(100));
  let Some(AnimationValue::CharacterFrame(lines)) = service.value(&pool, value) else {
    panic!("expected frame")
  };
  assert_eq!(&**lines, &["短".to_string()]);
}

#[test]
fn effect_parameters_are_animatable_without_per_cell_tweens() {
  let animation = AnimationService::new();
  let effects = CharacterEffectService::new();
  let mut pool = RuntimeObjectPool::new();
  let effect = effects.create(
    &mut pool,
    HashMap::from([(EffectParameterId::PHASE, AnimationValue::Float(0.0))]),
  );
  let handle = animation
    .play(
      &mut pool,
      AnimationOwner::Object(AnimationTarget::Effect(effect)),
      float_tween(0.0, 1.0, 100),
      vec![AnimationBinding {
        track: 0,
        target: AnimationTarget::Effect(effect),
        property: AnimationProperty::EffectPhase,
        initial_value: AnimationValue::Float(0.0),
      }],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  animation.update(&mut pool, AnimationClock::Ui, Duration::from_millis(50));
  assert_eq!(
    float(effects.parameter(&pool, effect, EffectParameterId::PHASE)),
    0.5
  );
  assert!(effects.remove(&mut pool, effect));
  assert_eq!(animation.state(&pool, handle), None);
}

#[derive(Default)]
struct RecordingRouter(Vec<AnimationWrite>);

impl AnimationTargetRouter for RecordingRouter {
  fn apply(&mut self, write: &AnimationWrite) -> Result<(), AnimationError> {
    self.0.push(write.clone());
    Ok(())
  }
}

struct MissingTargetRouter;

impl AnimationTargetRouter for MissingTargetRouter {
  fn apply(&mut self, write: &AnimationWrite) -> Result<(), AnimationError> {
    Err(AnimationError::TargetNotFound(write.target))
  }
}

#[test]
fn external_targets_are_applied_only_through_the_router() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let target = AnimationTarget::Ui(UiObjectRef {
    pool: UiPoolId(4),
    kind: UiObjectKind::ProgressBar,
    id: 9,
  });
  service
    .play(
      &mut pool,
      AnimationOwner::UiPool(UiPoolId(4)),
      float_tween(0.0, 1.0, 100),
      vec![AnimationBinding {
        track: 0,
        target,
        property: AnimationProperty::Progress,
        initial_value: AnimationValue::Float(0.0),
      }],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();
  let mut router = RecordingRouter::default();
  service
    .update_and_apply(
      &mut pool,
      AnimationClock::Ui,
      Duration::from_millis(50),
      &mut router,
    )
    .unwrap();
  assert_eq!(router.0.len(), 1);
  assert_eq!(router.0[0].target, target);
  assert_eq!(router.0[0].operation, AnimationWriteOperation::Override);
}

#[test]
fn missing_external_target_cancels_its_animation() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let target = AnimationTarget::Ui(UiObjectRef {
    pool: UiPoolId(4),
    kind: UiObjectKind::ProgressBar,
    id: 9,
  });
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::UiPool(UiPoolId(4)),
      float_tween(0.0, 1.0, 100),
      vec![AnimationBinding {
        track: 0,
        target,
        property: AnimationProperty::Progress,
        initial_value: AnimationValue::Float(0.0),
      }],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  let error = service
    .update_and_apply(
      &mut pool,
      AnimationClock::Ui,
      Duration::from_millis(50),
      &mut MissingTargetRouter,
    )
    .unwrap_err();
  assert_eq!(error, AnimationError::TargetNotFound(target));
  assert_eq!(service.state(&pool, handle), Some(PlaybackState::Cancelled));
}

#[test]
fn colors_interpolate_in_rgb_and_discrete_values_require_step() {
  let from = AnimationValue::Color(AnimationColor::rgb(0, 10, 20));
  let to = AnimationValue::Color(AnimationColor::rgb(100, 110, 120));
  assert_eq!(
    from
      .interpolate(&to, 0.5, AnimationInterpolation::Linear)
      .unwrap(),
    AnimationValue::Color(AnimationColor::rgb(50, 60, 70))
  );
  assert!(matches!(
    AnimationValue::Text("a".into()).interpolate(
      &AnimationValue::Text("b".into()),
      0.5,
      AnimationInterpolation::Linear
    ),
    Err(AnimationError::DiscreteInterpolationRequired(_))
  ));
}

#[test]
fn property_names_support_stable_aliases() {
  assert_eq!(
    AnimationProperty::from_str("style.foreground").unwrap(),
    AnimationProperty::Foreground
  );
  assert_eq!(
    AnimationProperty::from_str("text.visible_graphemes").unwrap(),
    AnimationProperty::VisibleGraphemes
  );
  assert!(AnimationProperty::from_str("future.magic").is_err());
}

#[test]
fn back_easing_can_overshoot_continuous_values() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let source = AnimationSource::Tween(Arc::new(TweenDefinition {
    from: AnimationValue::Float(0.0),
    to: AnimationValue::Float(10.0),
    duration: Duration::from_millis(100),
    easing: AnimationEasing::OutBack,
    interpolation: AnimationInterpolation::Linear,
  }));
  service
    .play(
      &mut pool,
      AnimationOwner::Host,
      source,
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(70));
  assert!(float(service.value(&pool, value)) > 10.0);
}

#[test]
fn step_values_switch_only_at_the_timeline_boundary() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Text("before".into()));
  let source = AnimationSource::Tween(Arc::new(TweenDefinition {
    from: AnimationValue::Text("before".into()),
    to: AnimationValue::Text("after".into()),
    duration: Duration::from_millis(100),
    easing: AnimationEasing::OutBack,
    interpolation: AnimationInterpolation::Step,
  }));
  service
    .play(
      &mut pool,
      AnimationOwner::Host,
      source,
      vec![AnimationBinding {
        track: 0,
        target: AnimationTarget::Value(value),
        property: AnimationProperty::Value,
        initial_value: AnimationValue::Text("before".into()),
      }],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(70));
  assert_eq!(
    service.value(&pool, value),
    Some(&AnimationValue::Text("before".into()))
  );
  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(30));
  assert_eq!(
    service.value(&pool, value),
    Some(&AnimationValue::Text("after".into()))
  );
}

#[test]
fn ping_pong_markers_follow_the_current_playback_direction() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let clip = AnimationClip {
    duration: Duration::from_millis(100),
    tracks: vec![AnimationTrack {
      property: AnimationProperty::Value,
      keyframes: vec![
        AnimationKeyframe {
          at: Duration::ZERO,
          value: AnimationValue::Float(0.0),
          easing: AnimationEasing::Linear,
          interpolation: AnimationInterpolation::Linear,
        },
        AnimationKeyframe {
          at: Duration::from_millis(100),
          value: AnimationValue::Float(1.0),
          easing: AnimationEasing::Linear,
          interpolation: AnimationInterpolation::Linear,
        },
      ],
    }],
    markers: vec![
      AnimationMarker {
        at: Duration::from_millis(25),
        name: "first".into(),
      },
      AnimationMarker {
        at: Duration::from_millis(75),
        name: "second".into(),
      },
    ],
  };
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      AnimationSource::Clip(Arc::new(clip)),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions {
        repeat: AnimationRepeatOptions {
          mode: AnimationRepeatMode::PingPong,
          count: AnimationRepeatCount::Finite(2),
        },
        emit_events: true,
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(200));
  let markers = service
    .take_events(&mut pool, handle)
    .into_iter()
    .filter_map(|event| match event.kind {
      AnimationEventKind::Marker { name } => Some(name),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(markers, ["first", "second", "second", "first"]);
}

#[test]
fn clip_samples_multiple_tracks_into_separate_writes() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let target = AnimationTarget::Ui(UiObjectRef {
    pool: UiPoolId(2),
    kind: UiObjectKind::Other,
    id: 3,
  });
  let clip = AnimationClip {
    duration: Duration::from_millis(100),
    tracks: vec![
      AnimationTrack {
        property: AnimationProperty::OffsetX,
        keyframes: vec![
          AnimationKeyframe {
            at: Duration::ZERO,
            value: AnimationValue::Float(-4.0),
            easing: AnimationEasing::Linear,
            interpolation: AnimationInterpolation::Linear,
          },
          AnimationKeyframe {
            at: Duration::from_millis(100),
            value: AnimationValue::Float(0.0),
            easing: AnimationEasing::Linear,
            interpolation: AnimationInterpolation::Linear,
          },
        ],
      },
      AnimationTrack {
        property: AnimationProperty::Foreground,
        keyframes: vec![
          AnimationKeyframe {
            at: Duration::ZERO,
            value: AnimationValue::Color(AnimationColor::rgb(0, 0, 0)),
            easing: AnimationEasing::Linear,
            interpolation: AnimationInterpolation::Linear,
          },
          AnimationKeyframe {
            at: Duration::from_millis(100),
            value: AnimationValue::Color(AnimationColor::rgb(100, 120, 140)),
            easing: AnimationEasing::Linear,
            interpolation: AnimationInterpolation::Linear,
          },
        ],
      },
    ],
    markers: Vec::new(),
  };
  service
    .play(
      &mut pool,
      AnimationOwner::UiPool(UiPoolId(2)),
      AnimationSource::Clip(Arc::new(clip)),
      vec![
        AnimationBinding {
          track: 0,
          target,
          property: AnimationProperty::OffsetX,
          initial_value: AnimationValue::Float(-4.0),
        },
        AnimationBinding {
          track: 1,
          target,
          property: AnimationProperty::Foreground,
          initial_value: AnimationValue::Color(AnimationColor::rgb(0, 0, 0)),
        },
      ],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  let update = service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(50));
  assert_eq!(update.writes.len(), 2);
  assert_eq!(update.writes[0].value, Some(AnimationValue::Float(-2.0)));
  assert_eq!(
    update.writes[1].value,
    Some(AnimationValue::Color(AnimationColor::rgb(50, 60, 70)))
  );
}

#[test]
fn unsupported_target_property_is_rejected_before_playback() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let result = service.play(
    &mut pool,
    AnimationOwner::Host,
    float_tween(0.0, 1.0, 100),
    vec![AnimationBinding {
      track: 0,
      target: AnimationTarget::Ui(UiObjectRef {
        pool: UiPoolId(1),
        kind: UiObjectKind::TextInput,
        id: 1,
      }),
      property: AnimationProperty::Progress,
      initial_value: AnimationValue::Float(0.0),
    }],
    AnimationPlaybackOptions::default(),
  );
  assert!(matches!(
    result,
    Err(AnimationError::PropertyNotSupported { .. })
  ));
}

#[test]
fn changing_base_value_during_restore_animation_keeps_the_override_layer() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 10.0, 100),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions {
        end_mode: AnimationEndMode::Restore,
        ..AnimationPlaybackOptions::default()
      },
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(50));
  service
    .set_value(&mut pool, value, AnimationValue::Float(20.0))
    .unwrap();
  assert_eq!(float(service.value(&pool, value)), 5.0);
  service.finish(&mut pool, handle).unwrap();
  assert_eq!(float(service.value(&pool, value)), 20.0);
}

#[test]
fn playback_speed_changes_only_future_time_advancement() {
  let service = AnimationService::new();
  let mut pool = RuntimeObjectPool::new();
  let value = service.create_value(&mut pool, AnimationValue::Float(0.0));
  let handle = service
    .play(
      &mut pool,
      AnimationOwner::Host,
      float_tween(0.0, 10.0, 100),
      vec![value_binding(value, 0.0)],
      AnimationPlaybackOptions::default(),
    )
    .unwrap();

  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(25));
  assert_eq!(float(service.value(&pool, value)), 2.5);
  assert!(service.set_speed(&mut pool, handle, 2.0));
  service.update(&mut pool, AnimationClock::Ui, Duration::from_millis(25));
  assert!((float(service.value(&pool, value)) - 7.5).abs() < f64::EPSILON * 8.0);
  assert!(!service.set_speed(&mut pool, handle, 0.0));
}
