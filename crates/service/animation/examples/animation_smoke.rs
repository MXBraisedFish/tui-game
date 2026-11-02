//! Independent animation smoke entry exercising the public API and checking its results.
//!
//! Run `cargo run -p tg-service-animation --example animation_smoke --locked`.
//! No terminal, assets, or external resources are required.

use std::{sync::Arc, time::Duration};

use tg_service_animation::{
  AnimationBinding, AnimationClock, AnimationEasing, AnimationInterpolation, AnimationObjects,
  AnimationOwner, AnimationPlaybackOptions, AnimationProperty, AnimationService, AnimationSource,
  AnimationTarget, AnimationValue, ParameterKeyframe, ParameterTrack, TweenDefinition,
};

fn main() {
  let track = ParameterTrack::new(
    Duration::from_secs(3),
    vec![
      ParameterKeyframe {
        timestamp: Duration::ZERO,
        values: [("x".into(), 0.0)].into(),
      },
      ParameterKeyframe {
        timestamp: Duration::from_secs(1),
        values: [("y".into(), 2.0)].into(),
      },
      ParameterKeyframe {
        timestamp: Duration::from_secs(3),
        values: [("x".into(), 6.0)].into(),
      },
    ],
  )
  .expect("valid parameter track");
  let segment = track.segment("x", Duration::from_millis(1500)).unwrap();
  assert_eq!(
    (segment.index, *segment.start_value, *segment.end_value),
    (1, 0.0, 6.0)
  );
  assert_eq!(track.frames()[0].values["y"], 2.0);
  assert_eq!(AnimationEasing::OutInCubic.sample(0.25), 0.4375);
  let service = AnimationService::new();
  let mut objects = AnimationObjects::new();
  let value = service.create_value(&mut objects, AnimationValue::Float(0.0));
  let tween = AnimationSource::Tween(Arc::new(TweenDefinition {
    from: AnimationValue::Float(0.0),
    to: AnimationValue::Float(10.0),
    duration: Duration::from_millis(1_000),
    easing: AnimationEasing::Linear,
    interpolation: AnimationInterpolation::Linear,
  }));
  let binding = AnimationBinding {
    track: 0,
    target: AnimationTarget::Value(value),
    property: AnimationProperty::Value,
    initial_value: AnimationValue::Float(0.0),
  };
  service
    .play(
      &mut objects,
      AnimationOwner::Host,
      tween,
      vec![binding],
      AnimationPlaybackOptions::default(),
    )
    .expect("valid tween");
  service.update(&mut objects, AnimationClock::Ui, Duration::from_millis(500));
  let sampled = service.value(&objects, value).cloned();
  assert_eq!(sampled, Some(AnimationValue::Float(5.0)));
  println!(
    "animation ok: {sampled:?}, playbacks {}",
    objects.animation_count()
  );
}
