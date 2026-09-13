use std::f64::consts::PI;

const MAX_TILT_RAD: f64 = 75.0_f64.to_radians();

const ANGULAR_DAMPING_PER_SECOND: f64 = 1.6;
const RATE_SLEEP_THRESHOLD_RAD_S: f64 = 0.0005;

#[derive(Debug, Clone, Copy)]
pub struct PhysicsSnapshot {
    pub attitude_rad: [f64; 3],
    pub angular_velocity_rad_s: [f64; 3],
    pub linear_acceleration_enu_mps2: [f64; 3],
    pub velocity_enu_mps: [f64; 3],
    pub position_enu_m: [f64; 3],
}

impl PhysicsSnapshot {
    pub fn attitude_deg(self) -> [f64; 3] {
        self.attitude_rad.map(f64::to_degrees)
    }

    pub fn angular_velocity_deg_s(self) -> [f64; 3] {
        self.angular_velocity_rad_s.map(f64::to_degrees)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsModel {
    attitude_rad: [f64; 3],
    angular_velocity_rad_s: [f64; 3],
    linear_acceleration_enu_mps2: [f64; 3],
    velocity_enu_mps: [f64; 3],
    position_enu_m: [f64; 3],
}

impl Default for PhysicsModel {
    fn default() -> Self {
        Self {
            attitude_rad: [0.0; 3],
            angular_velocity_rad_s: [0.0; 3],
            linear_acceleration_enu_mps2: [0.0; 3],
            velocity_enu_mps: [0.0; 3],
            position_enu_m: [0.0; 3],
        }
    }
}

impl PhysicsModel {
    pub fn snapshot(&self) -> PhysicsSnapshot {
        PhysicsSnapshot {
            attitude_rad: self.attitude_rad,
            angular_velocity_rad_s: self.angular_velocity_rad_s,
            linear_acceleration_enu_mps2: self.linear_acceleration_enu_mps2,
            velocity_enu_mps: self.velocity_enu_mps,
            position_enu_m: self.position_enu_m,
        }
    }

    pub fn step(&mut self, dt_seconds: f64) {
        let dt_seconds = dt_seconds.clamp(0.0, 0.05);

        if dt_seconds <= 0.0 {
            return;
        }

        for axis in 0..3 {
            self.attitude_rad[axis] += self.angular_velocity_rad_s[axis] * dt_seconds;
        }

        self.attitude_rad[0] = self.attitude_rad[0].clamp(-MAX_TILT_RAD, MAX_TILT_RAD);

        self.attitude_rad[1] = self.attitude_rad[1].clamp(-MAX_TILT_RAD, MAX_TILT_RAD);

        self.attitude_rad[2] = wrap_angle(self.attitude_rad[2]);

        let damping = (-ANGULAR_DAMPING_PER_SECOND * dt_seconds).exp();

        for rate in &mut self.angular_velocity_rad_s {
            *rate *= damping;

            if rate.abs() < RATE_SLEEP_THRESHOLD_RAD_S {
                *rate = 0.0;
            }
        }
    }

    pub fn set_attitude_deg(&mut self, roll_deg: f64, pitch_deg: f64, yaw_deg: f64) {
        self.attitude_rad = [
            roll_deg.to_radians().clamp(-MAX_TILT_RAD, MAX_TILT_RAD),
            pitch_deg.to_radians().clamp(-MAX_TILT_RAD, MAX_TILT_RAD),
            wrap_angle(yaw_deg.to_radians()),
        ];
    }

    pub fn add_angular_velocity_deg_s(&mut self, delta_deg_s: [f64; 3]) {
        for (rate, delta) in self.angular_velocity_rad_s.iter_mut().zip(delta_deg_s) {
            *rate += delta.to_radians();
        }
    }

    pub fn reset_attitude(&mut self) {
        self.attitude_rad = [0.0; 3];
        self.angular_velocity_rad_s = [0.0; 3];
    }

    pub fn reset_all(&mut self) {
        *self = Self::default();
    }
}

fn wrap_angle(mut angle: f64) -> f64 {
    while angle > PI {
        angle -= 2.0 * PI;
    }

    while angle < -PI {
        angle += 2.0 * PI;
    }

    angle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_level() {
        let snapshot = PhysicsModel::default().snapshot();

        assert_eq!(snapshot.attitude_rad, [0.0; 3],);

        assert_eq!(snapshot.angular_velocity_rad_s, [0.0; 3],);
    }

    #[test]
    fn attitude_can_be_set_in_degrees() {
        let mut model = PhysicsModel::default();

        model.set_attitude_deg(20.0, -15.0, 90.0);

        let attitude = model.snapshot().attitude_deg();

        assert!((attitude[0] - 20.0).abs() < 0.001);

        assert!((attitude[1] + 15.0).abs() < 0.001);

        assert!((attitude[2] - 90.0).abs() < 0.001);
    }

    #[test]
    fn angular_impulse_changes_attitude() {
        let mut model = PhysicsModel::default();

        model.add_angular_velocity_deg_s([90.0, 0.0, 0.0]);

        model.step(0.1);

        assert!(model.snapshot().attitude_rad[0] > 0.0);
    }
}
