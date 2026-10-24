use std::time::{Duration, Instant};

/// Accumulate small trackpad deltas, but never turn one wheel burst into a
/// race through every page. Input is raw egui scroll distance in logical points.
#[derive(Default)]
pub struct HeaderWheel {
    accumulated: f32,
    last_input: Option<Instant>,
    last_step: Option<Instant>,
}

impl HeaderWheel {
    /// Returns true for the next page, false for the previous page.
    pub fn input(&mut self, delta: f32, now: Instant) -> Option<bool> {
        if !delta.is_finite() || delta == 0.0 {
            return None;
        }
        if self
            .last_step
            .is_some_and(|at| now.saturating_duration_since(at) < Duration::from_millis(220))
        {
            self.accumulated = 0.0;
            return None;
        }
        if self
            .last_input
            .is_none_or(|at| now.saturating_duration_since(at) > Duration::from_millis(250))
            || delta.signum() != self.accumulated.signum()
        {
            self.accumulated = 0.0;
        }
        self.last_input = Some(now);
        self.accumulated += delta;
        if self.accumulated.abs() < 32.0 {
            return None;
        }
        self.accumulated = 0.0;
        self.last_step = Some(now);
        Some(delta < 0.0)
    }

    pub fn leave(&mut self) {
        self.accumulated = 0.0;
        self.last_input = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trackpad_noise_accumulates_only_inside_a_single_gesture() {
        let mut wheel = HeaderWheel::default();
        let now = Instant::now();
        assert_eq!(wheel.input(-10.0, now), None);
        assert_eq!(wheel.input(-12.0, now + Duration::from_millis(15)), None);
        assert_eq!(
            wheel.input(-12.0, now + Duration::from_millis(30)),
            Some(true)
        );
        wheel.leave();
        assert_eq!(wheel.input(-20.0, now + Duration::from_millis(400)), None);
        assert_eq!(wheel.input(-20.0, now + Duration::from_millis(700)), None);
    }

    #[test]
    fn one_burst_does_not_race_through_pages() {
        let mut wheel = HeaderWheel::default();
        let now = Instant::now();
        assert_eq!(wheel.input(-50.0, now), Some(true));
        for milliseconds in [10, 30, 80, 150] {
            assert_eq!(
                wheel.input(-50.0, now + Duration::from_millis(milliseconds)),
                None
            );
        }
        assert_eq!(
            wheel.input(50.0, now + Duration::from_millis(250)),
            Some(false)
        );
    }

    #[test]
    fn reversing_direction_discards_the_previous_partial_gesture() {
        let mut wheel = HeaderWheel::default();
        let now = Instant::now();
        assert_eq!(wheel.input(-24.0, now), None);
        assert_eq!(wheel.input(16.0, now + Duration::from_millis(10)), None);
        assert_eq!(
            wheel.input(16.0, now + Duration::from_millis(20)),
            Some(false)
        );
        assert_eq!(wheel.input(f32::NAN, now), None);
    }
}
