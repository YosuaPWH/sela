use std::time::Duration;

/// An exact critically damped spring. Retargeting keeps both position and velocity,
/// and the solution is independent of the display's refresh rate.
pub struct Expansion {
    value: f64,
    velocity: f64,
    target: f64,
}

impl Expansion {
    pub fn new(expanded: bool) -> Self {
        let target = f64::from(expanded);
        Self {
            value: target,
            velocity: 0.0,
            target,
        }
    }

    pub fn value(&self) -> f32 {
        self.value as f32
    }

    pub fn retarget(&mut self, expanded: bool) {
        self.target = f64::from(expanded);
    }

    pub fn is_active(&self) -> bool {
        self.value != self.target || self.velocity != 0.0
    }

    pub fn advance(&mut self, expanded: bool, elapsed: Duration, reduced_motion: bool) {
        let target = f64::from(expanded);
        if reduced_motion {
            self.value = target;
            self.velocity = 0.0;
            self.target = target;
            return;
        }

        // Advance toward the previous target first: an input received after an idle
        // frame must not be charged the whole idle interval on its first frame.
        let omega = if self.target > 0.5 { 32.0 } else { 38.0 };
        let seconds = elapsed.as_secs_f64();
        let displacement = self.value - self.target;
        let coefficient = self.velocity + omega * displacement;
        let decay = (-omega * seconds).exp();
        self.value = self.target + (displacement + coefficient * seconds) * decay;
        self.velocity = (self.velocity - omega * coefficient * seconds) * decay;
        self.value = self.value.clamp(0.0, 1.0);
        if (self.value - self.target).abs() < 0.0003 && self.velocity.abs() < 0.01 {
            self.value = self.target;
            self.velocity = 0.0;
        }
        self.target = target;
    }

    pub fn compact_opacity(&self) -> f32 {
        1.0 - smoothstep(0.02, 0.30, self.value())
    }

    pub fn content_opacity(&self) -> f32 {
        smoothstep(0.20, 0.92, self.value())
    }
}

fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Independent pixel dimensions let the music pill grow without resizing the OS surface.
pub struct Dimension {
    value: f64,
    velocity: f64,
    target: f64,
}

impl Dimension {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.into(),
            velocity: 0.0,
            target: value.into(),
        }
    }
    pub fn value(&self) -> f32 {
        self.value as f32
    }
    pub fn retarget(&mut self, value: f32) {
        self.target = value.into();
    }
    pub fn is_active(&self) -> bool {
        self.value != self.target || self.velocity != 0.0
    }
    pub fn advance(&mut self, target: f32, elapsed: Duration, reduced: bool) {
        if reduced {
            *self = Self::new(target);
            return;
        }
        let omega = if self.target >= self.value {
            32.0
        } else {
            38.0
        };
        let seconds = elapsed.as_secs_f64();
        let displacement = self.value - self.target;
        let coefficient = self.velocity + omega * displacement;
        let decay = (-omega * seconds).exp();
        self.value = self.target + (displacement + coefficient * seconds) * decay;
        self.velocity = (self.velocity - omega * coefficient * seconds) * decay;
        if (self.value - self.target).abs() < 0.03 && self.velocity.abs() < 0.9 {
            self.value = self.target;
            self.velocity = 0.0;
        }
        self.target = target.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_width_retargets_continuously_and_reduced_motion_snaps() {
        let mut width = Dimension::new(324.0);
        width.advance(580.0, Duration::from_secs(15), false);
        assert_eq!(width.value(), 324.0);
        width.advance(580.0, Duration::from_millis(70), false);
        let before = (width.value, width.velocity);
        width.advance(464.0, Duration::ZERO, false);
        assert_eq!((width.value, width.velocity), before);
        width.advance(600.0, Duration::ZERO, true);
        assert_eq!(width.value(), 600.0);
        assert!(!width.is_active());
    }

    #[test]
    fn idle_interval_does_not_skip_the_opening() {
        let mut motion = Expansion::new(false);
        motion.advance(true, Duration::from_secs(30), false);
        assert_eq!(motion.value(), 0.0);
        assert!(motion.is_active());
        motion.advance(true, Duration::from_millis(16), false);
        assert!(motion.value() > 0.0 && motion.value() < 0.2);
    }

    #[test]
    fn trajectory_matches_at_30_60_and_144_hz() {
        fn sample(hz: u32) -> f32 {
            let mut motion = Expansion::new(false);
            motion.advance(true, Duration::ZERO, false);
            for _ in 0..hz / 6 {
                motion.advance(true, Duration::from_secs_f64(1.0 / f64::from(hz)), false);
            }
            motion.value()
        }
        let reference = sample(60);
        for hz in [30, 144] {
            assert!((sample(hz) - reference).abs() < 0.00001);
        }
    }

    #[test]
    fn reversal_preserves_position_and_velocity_and_stays_in_bounds() {
        let mut motion = Expansion::new(false);
        motion.advance(true, Duration::ZERO, false);
        motion.advance(true, Duration::from_millis(70), false);
        let (value, velocity) = (motion.value, motion.velocity);
        motion.advance(false, Duration::ZERO, false);
        assert_eq!(motion.value, value);
        assert_eq!(motion.velocity, velocity);
        for step in 0..500 {
            motion.advance(step % 21 < 9, Duration::from_secs_f64(1.0 / 144.0), false);
            assert!((0.0..=1.0).contains(&motion.value));
        }
        for _ in 0..60 {
            motion.advance(false, Duration::from_secs_f64(1.0 / 60.0), false);
        }
        assert_eq!(motion.value(), 0.0);
        assert!(!motion.is_active());
    }

    #[test]
    fn reduced_motion_snaps_all_layers_and_cancels_velocity() {
        let mut motion = Expansion::new(false);
        motion.advance(true, Duration::ZERO, false);
        motion.advance(true, Duration::from_millis(30), false);
        motion.advance(true, Duration::ZERO, true);
        assert_eq!(motion.value(), 1.0);
        assert_eq!(motion.content_opacity(), 1.0);
        assert_eq!(motion.compact_opacity(), 0.0);
        assert!(!motion.is_active());
        motion.advance(false, Duration::ZERO, true);
        assert_eq!(motion.compact_opacity(), 1.0);
        assert_eq!(motion.content_opacity(), 0.0);
        assert!(!motion.is_active());
    }
}
