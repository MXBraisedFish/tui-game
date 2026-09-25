use super::AnimationEasing;

pub(crate) fn sample(easing: AnimationEasing, t: f64) -> f64 {
  let t = t.clamp(0.0, 1.0);
  match easing {
    AnimationEasing::Linear => t,
    AnimationEasing::InQuad => t * t,
    AnimationEasing::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
    AnimationEasing::InOutQuad => {
      if t < 0.5 {
        2.0 * t * t
      } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
      }
    }
    AnimationEasing::InCubic => t * t * t,
    AnimationEasing::OutCubic => 1.0 - (1.0 - t).powi(3),
    AnimationEasing::InOutCubic => {
      if t < 0.5 {
        4.0 * t.powi(3)
      } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
      }
    }
    AnimationEasing::InSine => 1.0 - (t * std::f64::consts::FRAC_PI_2).cos(),
    AnimationEasing::OutSine => (t * std::f64::consts::FRAC_PI_2).sin(),
    AnimationEasing::InOutSine => (1.0 - (std::f64::consts::PI * t).cos()) / 2.0,
    AnimationEasing::InBack => {
      const C1: f64 = 1.70158;
      const C3: f64 = C1 + 1.0;
      C3 * t.powi(3) - C1 * t.powi(2)
    }
    AnimationEasing::OutBack => {
      const C1: f64 = 1.70158;
      const C3: f64 = C1 + 1.0;
      1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }
    AnimationEasing::InOutBack => {
      const C1: f64 = 1.70158;
      const C2: f64 = C1 * 1.525;
      if t < 0.5 {
        (2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2) / 2.0
      } else {
        ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
      }
    }
  }
}
