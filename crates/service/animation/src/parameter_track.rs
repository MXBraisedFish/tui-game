//! Named parameter tracks with independently closed interpolation segments.
//!
//! # Examples
//!
//! ```rust
//! use std::time::Duration;
//! use tg_service_animation::{ParameterKeyframe, ParameterTrack};
//! let track = ParameterTrack::new(Duration::from_secs(1), vec![
//!   ParameterKeyframe { timestamp: Duration::ZERO, values: [("x".into(), 0)].into() },
//!   ParameterKeyframe { timestamp: Duration::from_secs(1), values: [("x".into(), 10)].into() },
//! ]).unwrap();
//! let segment = track.segment("x", Duration::from_millis(500)).unwrap();
//! assert_eq!((segment.index, *segment.start_value, *segment.end_value), (1, 0, 10));
//! ```

use std::{collections::BTreeMap, fmt, time::Duration};

/// A timestamp and the parameters explicitly defined at that time.
///
/// # Fields
///
/// * `timestamp` - The position within the animation duration.
/// * `values` - The parameters explicitly recorded at that timestamp.
#[derive(Clone, Debug)]
pub struct ParameterKeyframe<V> {
  /// The timestamp within the animation duration.
  pub timestamp: Duration,
  /// The named values defined at this timestamp.
  pub values: BTreeMap<String, V>,
}

/// Sorted keyframes and independently closed tracks for arbitrary parameter values.
#[derive(Clone, Debug)]
pub struct ParameterTrack<V> {
  frames: Vec<ParameterKeyframe<V>>,
  parameters: BTreeMap<String, Vec<(Duration, V)>>,
}

/// A borrowed parameter segment and its one-based index within that parameter's track.
///
/// # Fields
///
/// * `index` - The one-based interpolation segment number for this parameter.
/// * `start_time` - The segment's start timestamp.
/// * `end_time` - The segment's end timestamp.
/// * `start_value` - The value at the start.
/// * `end_value` - The value at the end.
pub struct ParameterSegment<'a, V> {
  /// The one-based segment index used to select its easing function.
  pub index: usize,
  /// The start timestamp.
  pub start_time: Duration,
  /// The end timestamp, equal to the start for a single-node track.
  pub end_time: Duration,
  /// The start value.
  pub start_value: &'a V,
  /// The end value.
  pub end_value: &'a V,
}

/// An invalid parameter keyframe timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParameterTrackError;

impl fmt::Display for ParameterTrackError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("keyframe timestamp exceeds animation duration")
  }
}

impl std::error::Error for ParameterTrackError {}

impl<V: Clone> ParameterTrack<V> {
  /// Sort keyframes, keep the first duplicate timestamp, and close uncovered boundary nodes.
  ///
  /// Missing nodes between two explicit values remain one continuous segment. Nodes before
  /// the first or after the last explicit value repeat that boundary value.
  ///
  /// # Errors
  ///
  /// Return [`ParameterTrackError`] if any timestamp exceeds `duration`.
  pub fn new(
    duration: Duration,
    mut frames: Vec<ParameterKeyframe<V>>,
  ) -> Result<Self, ParameterTrackError> {
    if frames.iter().any(|frame| frame.timestamp > duration) {
      return Err(ParameterTrackError);
    }
    frames.sort_by_key(|frame| frame.timestamp);
    frames.dedup_by_key(|frame| frame.timestamp);
    let mut parameters: BTreeMap<String, Vec<(Duration, V)>> = BTreeMap::new();
    for frame in &frames {
      for (key, value) in &frame.values {
        parameters
          .entry(key.clone())
          .or_default()
          .push((frame.timestamp, value.clone()));
      }
    }
    for (key, nodes) in &mut parameters {
      // Only the original explicit boundaries determine which global nodes need padding.
      let (first_time, first_value) = nodes[0].clone();
      let (last_time, last_value) = nodes[nodes.len() - 1].clone();
      for frame in &mut frames {
        let boundary = if frame.timestamp < first_time {
          Some(&first_value)
        } else if frame.timestamp > last_time {
          Some(&last_value)
        } else {
          None
        };
        if let Some(value) = boundary {
          frame.values.insert(key.clone(), value.clone());
          nodes.push((frame.timestamp, value.clone()));
        }
      }
      nodes.sort_by_key(|(timestamp, _)| *timestamp);
    }
    Ok(Self { frames, parameters })
  }

  /// Return the sorted, deduplicated keyframes with boundary values filled in.
  pub fn frames(&self) -> &[ParameterKeyframe<V>] {
    &self.frames
  }

  /// Iterate over parameter names in deterministic order.
  pub fn keys(&self) -> impl Iterator<Item = &str> {
    self.parameters.keys().map(String::as_str)
  }

  /// Locate a parameter segment, retaining endpoint values outside the defined track.
  pub fn segment(&self, key: &str, time: Duration) -> Option<ParameterSegment<'_, V>> {
    let nodes = self.parameters.get(key)?;
    let index = nodes
      .partition_point(|(timestamp, _)| *timestamp < time)
      .saturating_sub(1)
      .min(nodes.len().saturating_sub(2));
    let (start_time, start_value) = &nodes[index];
    let (end_time, end_value) = &nodes[(index + 1).min(nodes.len() - 1)];
    Some(ParameterSegment {
      index: index + 1,
      start_time: *start_time,
      end_time: *end_time,
      start_value,
      end_value,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sparse_tracks_close_boundaries_without_splitting_internal_segments() {
    let frame = |timestamp, values: &[(&str, i32)]| ParameterKeyframe {
      timestamp: Duration::from_secs(timestamp),
      values: values
        .iter()
        .map(|(key, value)| ((*key).into(), *value))
        .collect(),
    };
    let track = ParameterTrack::new(
      Duration::from_secs(3),
      vec![
        frame(2, &[("a", 2), ("b", 1)]),
        frame(0, &[("b", 3)]),
        frame(1, &[("a", 1), ("c", 4)]),
        frame(3, &[]),
        frame(1, &[("a", 99), ("discarded", 1)]),
      ],
    )
    .unwrap();
    assert_eq!(track.frames().len(), 4);
    assert_eq!(track.frames()[0].values["a"], 1);
    assert!(!track.frames()[1].values.contains_key("b"));
    assert!(!track.keys().any(|key| key == "discarded"));
    let segment = track.segment("b", Duration::from_secs(1)).unwrap();
    assert_eq!(
      (segment.index, segment.start_time, segment.end_time),
      (1, Duration::ZERO, Duration::from_secs(2))
    );
    assert_eq!((*segment.start_value, *segment.end_value), (3, 1));
    let segment = track.segment("c", Duration::from_millis(2500)).unwrap();
    assert_eq!(segment.index, 3);
    assert_eq!((*segment.start_value, *segment.end_value), (4, 4));
    assert!(track.segment("missing", Duration::ZERO).is_none());
    assert!(ParameterTrack::new(Duration::ZERO, vec![frame(1, &[])]).is_err());
    assert!(
      ParameterTrack::<i32>::new(Duration::ZERO, vec![])
        .unwrap()
        .keys()
        .next()
        .is_none()
    );
  }
}
