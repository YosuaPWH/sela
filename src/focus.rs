use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Ready,
    Running,
    Paused,
    Complete,
}

pub struct FocusTimer {
    pub phase: Phase,
    total: Duration,
    remaining: Duration,
    pub completed: u32,
}

impl FocusTimer {
    pub fn new(minutes: u32) -> Self {
        let total = Duration::from_secs(u64::from(minutes.clamp(1, 120)) * 60);
        Self {
            phase: Phase::Ready,
            total,
            remaining: total,
            completed: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.phase = match self.phase {
            Phase::Running => Phase::Paused,
            Phase::Complete => {
                self.remaining = self.total;
                Phase::Running
            }
            _ => Phase::Running,
        };
    }

    pub fn reset(&mut self, minutes: u32) {
        self.total = Duration::from_secs(u64::from(minutes.clamp(1, 120)) * 60);
        self.remaining = self.total;
        self.phase = Phase::Ready;
    }

    /// Returns true exactly once at the running → complete transition.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if self.phase != Phase::Running {
            return false;
        }
        self.remaining = self.remaining.saturating_sub(elapsed);
        if self.remaining.is_zero() {
            self.phase = Phase::Complete;
            self.completed = self.completed.saturating_add(1);
            return true;
        }
        false
    }

    pub fn label(&self) -> String {
        let seconds = self.remaining.as_secs() + u64::from(self.remaining.subsec_nanos() > 0);
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }

    pub fn progress(&self) -> f32 {
        1.0 - self.remaining.as_secs_f32() / self.total.as_secs_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_resume_and_completion_are_monotonic() {
        let mut t = FocusTimer::new(1);
        t.toggle();
        assert!(!t.advance(Duration::from_millis(500)));
        assert_eq!(t.label(), "01:00");
        t.toggle();
        t.advance(Duration::from_secs(100));
        assert_eq!(t.label(), "01:00");
        t.toggle();
        assert!(t.advance(Duration::from_secs(60)));
        assert_eq!(t.completed, 1);
        assert_eq!(t.label(), "00:00");
        assert!(!t.advance(Duration::from_secs(60)));
        t.toggle();
        assert_eq!(t.label(), "01:00");
        assert_eq!(t.phase, Phase::Running);
    }

    #[test]
    fn reset_cancels_active_session() {
        let mut t = FocusTimer::new(25);
        t.toggle();
        t.advance(Duration::from_secs(30));
        t.reset(50);
        assert_eq!(t.phase, Phase::Ready);
        assert_eq!(t.label(), "50:00");
        assert_eq!(t.progress(), 0.0);
    }
}
