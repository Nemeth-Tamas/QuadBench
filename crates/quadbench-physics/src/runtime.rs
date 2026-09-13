use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::model::{PhysicsModel, PhysicsSnapshot};

pub const DEFAULT_PHYSICS_RATE_HZ: u32 = 500;

#[derive(Debug, Clone, Copy)]
pub struct PhysicsRuntimeConfig {
    pub tick_rate_hz: u32,
}

impl Default for PhysicsRuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate_hz: DEFAULT_PHYSICS_RATE_HZ,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsRuntimeTelemetry {
    pub enabled: bool,
    pub configured_tick_rate_hz: u32,
    pub measured_tick_rate_hz: f64,
    pub tick_count: u64,
    pub overrun_count: u64,
    pub last_tick_age: Option<Duration>,
}

struct RuntimeTelemetry {
    tick_count: u64,
    overrun_count: u64,
    last_tick: Option<Instant>,
    measured_tick_rate_hz: f64,
    rate_window_started: Instant,
    rate_window_ticks: u64,
}

struct SharedState {
    model: Mutex<PhysicsModel>,
    enabled: AtomicBool,
    stop: AtomicBool,
    telemetry: Mutex<RuntimeTelemetry>,
}

#[derive(Clone)]
pub struct PhysicsHandle {
    shared: Arc<SharedState>,
    config: PhysicsRuntimeConfig,
}

pub struct PhysicsRuntime {
    handle: PhysicsHandle,
    worker: Option<JoinHandle<()>>,
}

impl PhysicsRuntime {
    pub fn spawn(config: PhysicsRuntimeConfig) -> io::Result<Self> {
        let shared = Arc::new(SharedState {
            model: Mutex::new(PhysicsModel::default()),
            enabled: AtomicBool::new(false),
            stop: AtomicBool::new(false),
            telemetry: Mutex::new(RuntimeTelemetry {
                tick_count: 0,
                overrun_count: 0,
                last_tick: None,
                measured_tick_rate_hz: 0.0,
                rate_window_started: Instant::now(),
                rate_window_ticks: 0,
            }),
        });

        let handle = PhysicsHandle {
            shared: Arc::clone(&shared),
            config,
        };

        let worker_shared = Arc::clone(&shared);

        let worker = thread::Builder::new()
            .name("quadbench-physics".to_owned())
            .spawn(move || {
                run_worker(worker_shared, config);
            })?;

        Ok(Self {
            handle,
            worker: Some(worker),
        })
    }

    pub fn handle(&self) -> PhysicsHandle {
        self.handle.clone()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.handle.set_enabled(enabled);
    }

    pub fn snapshot(&self) -> PhysicsSnapshot {
        self.handle.snapshot()
    }

    pub fn telemetry(&self) -> PhysicsRuntimeTelemetry {
        self.handle.telemetry()
    }

    pub fn set_motor_commands(&self, commands: [f32; 4]) {
        self.handle.set_motor_commands(commands);
    }

    pub fn set_attitude_deg(&self, roll_deg: f64, pitch_deg: f64, yaw_deg: f64) {
        self.handle.set_attitude_deg(roll_deg, pitch_deg, yaw_deg);
    }

    pub fn add_angular_velocity_deg_s(&self, delta_deg_s: [f64; 3]) {
        self.handle.add_angular_velocity_deg_s(delta_deg_s);
    }

    pub fn reset_attitude(&self) {
        self.handle.reset_attitude();
    }

    pub fn reset_translation(&self) {
        self.handle.reset_translation();
    }

    pub fn reset_all(&self) {
        self.handle.reset_all();
    }
}

impl PhysicsHandle {
    pub fn set_enabled(&self, enabled: bool) {
        self.shared.enabled.store(enabled, Ordering::Release);
    }

    pub fn snapshot(&self) -> PhysicsSnapshot {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
    }

    pub fn telemetry(&self) -> PhysicsRuntimeTelemetry {
        let telemetry = self
            .shared
            .telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        PhysicsRuntimeTelemetry {
            enabled: self.shared.enabled.load(Ordering::Acquire),
            configured_tick_rate_hz: self.config.tick_rate_hz.max(1),
            measured_tick_rate_hz: telemetry.measured_tick_rate_hz,
            tick_count: telemetry.tick_count,
            overrun_count: telemetry.overrun_count,
            last_tick_age: telemetry.last_tick.map(|instant| instant.elapsed()),
        }
    }

    pub fn set_motor_commands(&self, commands: [f32; 4]) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .set_motor_commands(commands);
    }

    pub fn set_attitude_deg(&self, roll_deg: f64, pitch_deg: f64, yaw_deg: f64) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .set_attitude_deg(roll_deg, pitch_deg, yaw_deg);
    }

    pub fn add_angular_velocity_deg_s(&self, delta_deg_s: [f64; 3]) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .add_angular_velocity_deg_s(delta_deg_s);
    }

    pub fn reset_attitude(&self) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset_attitude();
    }

    pub fn reset_translation(&self) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset_translation();
    }

    pub fn reset_all(&self) {
        self.shared
            .model
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset_all();
    }
}

impl Drop for PhysicsRuntime {
    fn drop(&mut self) {
        self.handle.shared.stop.store(true, Ordering::Release);

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_worker(shared: Arc<SharedState>, config: PhysicsRuntimeConfig) {
    let tick_rate_hz = config.tick_rate_hz.max(1);

    let dt_seconds = 1.0 / f64::from(tick_rate_hz);

    let tick_interval = Duration::from_secs_f64(dt_seconds);

    let spin_margin = Duration::from_micros(200);

    let mut next_tick = Instant::now() + tick_interval;

    while !shared.stop.load(Ordering::Acquire) {
        if !shared.enabled.load(Ordering::Acquire) {
            let now = Instant::now();

            next_tick = now + tick_interval;

            {
                let mut telemetry = shared
                    .telemetry
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                telemetry.measured_tick_rate_hz = 0.0;

                telemetry.rate_window_started = now;

                telemetry.rate_window_ticks = 0;
            }

            thread::sleep(Duration::from_millis(1));

            continue;
        }

        let now = Instant::now();

        if now < next_tick {
            let remaining = next_tick - now;

            let sleep_for = remaining.saturating_sub(spin_margin);

            if !sleep_for.is_zero() {
                thread::sleep(sleep_for);
            } else {
                thread::yield_now();
            }

            continue;
        }

        {
            let mut model = shared
                .model
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            model.step(dt_seconds);
        }

        let completed = Instant::now();

        {
            let mut telemetry = shared
                .telemetry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            telemetry.tick_count += 1;
            telemetry.last_tick = Some(completed);
            telemetry.rate_window_ticks += 1;

            let rate_elapsed = completed.duration_since(telemetry.rate_window_started);

            if rate_elapsed >= Duration::from_millis(500) {
                telemetry.measured_tick_rate_hz =
                    telemetry.rate_window_ticks as f64 / rate_elapsed.as_secs_f64();

                telemetry.rate_window_started = completed;

                telemetry.rate_window_ticks = 0;
            }
        }

        next_tick += tick_interval;

        if completed > next_tick {
            let mut telemetry = shared
                .telemetry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            telemetry.overrun_count += 1;

            next_tick = completed + tick_interval;
        }
    }
}
