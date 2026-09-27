//! Minimal entry: plays a linear float tween on a standalone value and samples it halfway.

use std::{sync::Arc, time::Duration};

use tg_service_animation::{
  AnimationBinding, AnimationClock, AnimationEasing, AnimationInterpolation, AnimationObjects,
  AnimationOwner,
  AnimationPlaybackOptions, AnimationProperty, AnimationService, AnimationSource, AnimationTarget,
  AnimationValue, TweenDefinition,
};

fn main() {
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
    .play(&mut objects, AnimationOwner::Host, tween, vec![binding], AnimationPlaybackOptions::default())
    .expect("valid tween");
  service.update(&mut objects, AnimationClock::Ui, Duration::from_millis(500));
  let sampled = service.value(&objects, value).cloned();
  assert_eq!(sampled, Some(AnimationValue::Float(5.0)));
  println!("animation ok: {sampled:?}, playbacks {}", objects.animation_count());
}
