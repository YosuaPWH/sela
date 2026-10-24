use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use sysinfo::System;

#[derive(Debug, Default, Clone)]
pub struct Snapshot {
    pub metrics_ready: bool,
    pub cpu: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub battery: Option<Battery>,
}

#[derive(Debug, Clone)]
pub struct Battery {
    pub percent: u8,
    pub plugged_in: bool,
}

pub struct Monitor {
    pub updates: mpsc::Receiver<Snapshot>,
    control: mpsc::Sender<bool>,
    active: bool,
}

impl Monitor {
    /// Only send state changes; repainting or rapidly switching tabs must not
    /// restart the CPU baseline on every frame.
    pub fn set_active(&mut self, active: bool) -> bool {
        if self.active == active {
            return false;
        }
        self.active = active;
        let _ = self.control.send(active);
        true
    }
}

pub fn spawn() -> Monitor {
    let (tx, rx) = mpsc::channel();
    let (control, commands) = mpsc::channel();
    thread::Builder::new()
        .name("luma-system".into())
        .spawn(move || {
            let mut system = System::new();
            let mut active = false;
            let mut next_metrics: Option<Instant> = None;
            let mut next_battery = Instant::now();
            let mut snapshot = Snapshot::default();
            loop {
                let deadline = next_metrics.map_or(next_battery, |next| next.min(next_battery));
                match commands.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(requested) => {
                        // Coalesce queued page changes before doing any OS work.
                        let requested = commands.try_iter().last().unwrap_or(requested);
                        if requested == active {
                            continue;
                        }
                        active = requested;
                        snapshot.metrics_ready = false;
                        next_metrics = if active {
                            system.refresh_cpu_usage();
                            Some(
                                Instant::now()
                                    + sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
                                        .max(Duration::from_millis(250)),
                            )
                        } else {
                            None
                        };
                        if tx.send(snapshot.clone()).is_err() {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let now = Instant::now();
                        if next_metrics.is_some_and(|due| now >= due) {
                            system.refresh_cpu_usage();
                            system.refresh_memory();
                            snapshot.cpu = system.global_cpu_usage();
                            snapshot.memory_used = system.used_memory();
                            snapshot.memory_total = system.total_memory();
                            snapshot.metrics_ready = true;
                            next_metrics = Some(now + Duration::from_secs(2));
                            next_battery = now;
                        }
                        if now >= next_battery {
                            snapshot.battery = crate::platform::battery();
                            next_battery = now + Duration::from_secs(30);
                        }
                        if tx.send(snapshot.clone()).is_err() {
                            break;
                        }
                    }
                }
            }
        })
        .expect("cannot start system worker");
    Monitor {
        updates: rx,
        control,
        active: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_warm_up_only_on_request_and_stop_after_the_page_closes() {
        let mut monitor = spawn();
        assert!(
            !monitor
                .updates
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .metrics_ready
        );
        assert!(monitor.set_active(true));
        assert!(!monitor.set_active(true));
        loop {
            let snapshot = monitor
                .updates
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            if snapshot.metrics_ready {
                assert!(snapshot.memory_total > 0);
                break;
            }
        }
        monitor.set_active(false);
        loop {
            if !monitor
                .updates
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .metrics_ready
            {
                break;
            }
        }
        assert!(matches!(
            monitor.updates.recv_timeout(Duration::from_millis(2100)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
    }
}
