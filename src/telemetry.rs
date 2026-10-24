use std::{sync::mpsc, thread, time::Duration};
use sysinfo::System;

#[derive(Debug, Default, Clone)]
pub struct Snapshot {
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

pub fn spawn() -> mpsc::Receiver<Snapshot> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("luma-system".into())
        .spawn(move || {
            let mut system = System::new();
            loop {
                system.refresh_cpu_usage();
                system.refresh_memory();
                if tx
                    .send(Snapshot {
                        cpu: system.global_cpu_usage(),
                        memory_used: system.used_memory(),
                        memory_total: system.total_memory(),
                        battery: crate::platform::battery(),
                    })
                    .is_err()
                {
                    break;
                }
                thread::sleep(Duration::from_secs(2));
            }
        })
        .expect("cannot start system worker");
    rx
}
